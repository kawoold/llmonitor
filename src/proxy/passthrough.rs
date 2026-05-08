use axum::{
    body::Body,
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::Response,
};
use bytes::Bytes;
use chrono::Utc;
use futures::StreamExt;
use serde::Deserialize;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::proxy::error::ProxyError;
use crate::providers::claude::types::{ANTHROPIC_API_URL, ANTHROPIC_VERSION};
use crate::tracking::session::SessionEngine;
use crate::tracking::types::UsageRecord;

#[derive(Deserialize)]
struct PassthroughMeta {
    model: String,
    #[serde(default)]
    messages: Vec<serde_json::Value>,
    stream: Option<bool>,
}

/// Transparent proxy for the native Anthropic Messages API.
/// Forwards requests to api.anthropic.com, substituting the server's configured
/// API key, and records usage on the way back.
pub async fn messages_passthrough(
    State(state): State<AppState>,
    in_headers: HeaderMap,
    body: Bytes,
) -> Result<Response<Body>, ProxyError> {
    let meta: PassthroughMeta = serde_json::from_slice(&body)
        .map_err(|e| ProxyError::RequestParseError(format!("invalid request body: {e}")))?;

    let api_key = state
        .config_service
        .get_anthropic_api_key()
        .await
        .map_err(|e| ProxyError::InternalError(e.to_string()))?;

    let anthropic_version = in_headers
        .get("anthropic-version")
        .and_then(|v| v.to_str().ok())
        .unwrap_or(ANTHROPIC_VERSION);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(600))
        .build()
        .map_err(|e| ProxyError::InternalError(e.to_string()))?;

    let mut req_builder = client
        .post(ANTHROPIC_API_URL)
        .header("x-api-key", api_key.0.as_str())
        .header("anthropic-version", anthropic_version)
        .header("content-type", "application/json")
        .body(body.to_vec());

    if let Some(beta) = in_headers.get("anthropic-beta").and_then(|v| v.to_str().ok()) {
        req_builder = req_builder.header("anthropic-beta", beta);
    }

    let upstream = req_builder
        .send()
        .await
        .map_err(|e| ProxyError::InternalError(format!("upstream request failed: {e}")))?;

    // Forward non-2xx responses from Anthropic transparently so the client
    // sees the real error (e.g. 400 invalid_request, 429 rate_limit, etc.)
    if !upstream.status().is_success() {
        let status = StatusCode::from_u16(upstream.status().as_u16())
            .unwrap_or(StatusCode::BAD_GATEWAY);
        let err_bytes = upstream.bytes().await.unwrap_or_default();
        return Ok(Response::builder()
            .status(status)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(err_bytes))
            .unwrap());
    }

    if meta.stream == Some(true) {
        passthrough_stream(upstream, meta.model, meta.messages, state).await
    } else {
        passthrough_non_stream(upstream, meta.model, meta.messages, state).await
    }
}

async fn passthrough_non_stream(
    upstream: reqwest::Response,
    model: String,
    messages: Vec<serde_json::Value>,
    state: AppState,
) -> Result<Response<Body>, ProxyError> {
    let body_bytes = upstream
        .bytes()
        .await
        .map_err(|e| ProxyError::InternalError(format!("reading upstream body: {e}")))?;

    // Parse usage best-effort — don't fail the response if parsing fails
    if let Ok(v) = serde_json::from_slice::<serde_json::Value>(&body_bytes) {
        let resolved_model = v
            .get("model")
            .and_then(|m| m.as_str())
            .unwrap_or(&model)
            .to_string();

        let usage = v.get("usage");
        let input_tokens = usage
            .and_then(|u| u.get("input_tokens"))
            .and_then(|n| n.as_u64())
            .unwrap_or(0) as u32;
        let output_tokens = usage
            .and_then(|u| u.get("output_tokens"))
            .and_then(|n| n.as_u64())
            .unwrap_or(0) as u32;
        let cache_read = usage
            .and_then(|u| u.get("cache_read_input_tokens"))
            .and_then(|n| n.as_u64())
            .unwrap_or(0) as u32;
        let cache_creation = usage
            .and_then(|u| u.get("cache_creation_input_tokens"))
            .and_then(|n| n.as_u64())
            .unwrap_or(0) as u32;

        if input_tokens > 0 || output_tokens > 0 {
            let record = UsageRecord {
                id: Uuid::new_v4().to_string(),
                created_at: Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                provider: "anthropic".to_string(),
                model: resolved_model.clone(),
                session_id: SessionEngine::derive_session_id_from_values(&messages),
                prompt_tokens: input_tokens,
                completion_tokens: output_tokens,
                total_tokens: input_tokens + output_tokens,
                cache_read_tokens: cache_read,
                cache_creation_tokens: cache_creation,
            };
            state.tracking_service.record(record);
            state.metrics.record_cache(cache_read, cache_creation);
            state.metrics.record_tokens("anthropic", &resolved_model, input_tokens, output_tokens);
            state.metrics.record_request("anthropic", &resolved_model, 200);
        }
    }

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body_bytes))
        .unwrap())
}

async fn passthrough_stream(
    upstream: reqwest::Response,
    model: String,
    messages: Vec<serde_json::Value>,
    state: AppState,
) -> Result<Response<Body>, ProxyError> {
    let (tx, rx) = mpsc::channel::<Result<Bytes, std::io::Error>>(64);

    tokio::spawn(async move {
        let mut byte_stream = upstream.bytes_stream();
        let mut sse_buf = String::new();
        let mut input_tokens: u32 = 0;
        let mut output_tokens: u32 = 0;
        let mut cache_read: u32 = 0;
        let mut cache_creation: u32 = 0;
        let mut resolved_model = model.clone();

        while let Some(chunk) = byte_stream.next().await {
            match chunk {
                Ok(bytes) => {
                    if let Ok(text) = std::str::from_utf8(&bytes) {
                        sse_buf.push_str(text);
                        parse_sse_usage(
                            &sse_buf,
                            &mut input_tokens,
                            &mut output_tokens,
                            &mut cache_read,
                            &mut cache_creation,
                            &mut resolved_model,
                        );
                        // Discard fully-parsed lines; keep only the trailing partial line.
                        if let Some(pos) = sse_buf.rfind('\n') {
                            sse_buf.drain(..=pos);
                        }
                    }
                    let _ = tx.send(Ok(bytes)).await;
                }
                Err(e) => {
                    let _ = tx
                        .send(Err(std::io::Error::new(std::io::ErrorKind::Other, e)))
                        .await;
                    return;
                }
            }
        }

        if input_tokens > 0 || output_tokens > 0 {
            let record = UsageRecord {
                id: Uuid::new_v4().to_string(),
                created_at: Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                provider: "anthropic".to_string(),
                model: resolved_model.clone(),
                session_id: SessionEngine::derive_session_id_from_values(&messages),
                prompt_tokens: input_tokens,
                completion_tokens: output_tokens,
                total_tokens: input_tokens + output_tokens,
                cache_read_tokens: cache_read,
                cache_creation_tokens: cache_creation,
            };
            state.tracking_service.record(record);
            state.metrics.record_cache(cache_read, cache_creation);
            state.metrics.record_tokens("anthropic", &resolved_model, input_tokens, output_tokens);
            state.metrics.record_request("anthropic", &resolved_model, 200);
        }
    });

    let stream = tokio_stream::wrappers::ReceiverStream::new(rx);
    let body = Body::from_stream(stream);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/event-stream")
        .header(header::CACHE_CONTROL, "no-cache")
        .header("x-accel-buffering", "no")
        .body(body)
        .unwrap())
}

fn parse_sse_usage(
    buf: &str,
    input_tokens: &mut u32,
    output_tokens: &mut u32,
    cache_read: &mut u32,
    cache_creation: &mut u32,
    resolved_model: &mut String,
) {
    for line in buf.lines() {
        let Some(json_str) = line.strip_prefix("data: ") else {
            continue;
        };
        let Ok(v) = serde_json::from_str::<serde_json::Value>(json_str) else {
            continue;
        };
        match v.get("type").and_then(|t| t.as_str()) {
            Some("message_start") => {
                if let Some(msg) = v.get("message") {
                    if let Some(m) = msg.get("model").and_then(|m| m.as_str()) {
                        *resolved_model = m.to_string();
                    }
                    if let Some(u) = msg.get("usage") {
                        if let Some(n) = u.get("input_tokens").and_then(|n| n.as_u64()) {
                            *input_tokens = n as u32;
                        }
                        if let Some(n) = u.get("cache_read_input_tokens").and_then(|n| n.as_u64()) {
                            *cache_read = n as u32;
                        }
                        if let Some(n) = u.get("cache_creation_input_tokens").and_then(|n| n.as_u64()) {
                            *cache_creation = n as u32;
                        }
                    }
                }
            }
            Some("message_delta") => {
                if let Some(u) = v.get("usage") {
                    if let Some(n) = u.get("output_tokens").and_then(|n| n.as_u64()) {
                        *output_tokens = n as u32;
                    }
                }
            }
            _ => {}
        }
    }
}
