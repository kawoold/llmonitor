use axum::{
    body::Body,
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Json, Response},
};
use bytes::Bytes;
use chrono::Utc;
use futures::StreamExt;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::proxy::error::ProxyError;
use crate::proxy::routing::{extract_provider_name, validate_provider_name};
use crate::proxy::types::OpenAiChatRequest;
use crate::providers::Provider;
use crate::providers::claude::ClaudeAdapter;
use crate::tracking::session::SessionEngine;
use crate::tracking::types::UsageRecord;

pub async fn chat_completion(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<OpenAiChatRequest>,
) -> Result<impl IntoResponse, ProxyError> {
    if request.stream == Some(true) {
        return Ok(
            chat_completion_stream_inner(state, headers, request)
                .await?
                .into_response(),
        );
    }

    let provider_name = extract_provider_name(&headers)?;
    validate_provider_name(&provider_name)?;

    let api_key = state
        .config_service
        .get_anthropic_api_key()
        .await
        .map_err(|e| ProxyError::InternalError(e.to_string()))?;

    let provider = Provider::from_name(&provider_name, api_key)?;

    if !provider.accepts_model(&request.model) {
        let accepted = provider.accepted_models().join(", ");
        return Err(ProxyError::ModelNotAccepted {
            model: request.model.clone(),
            provider: provider.name().to_string(),
            accepted,
        });
    }

    let messages = request.messages.clone();
    let (response, raw_usage) = provider
        .chat_completion(request)
        .await
        .map_err(ProxyError::from)?;

    let usage_record = UsageRecord {
        id: Uuid::new_v4().to_string(),
        created_at: Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        provider: provider.name().to_string(),
        model: response.model.clone(),
        session_id: SessionEngine::derive_session_id(&messages),
        prompt_tokens: raw_usage.prompt_tokens,
        completion_tokens: raw_usage.completion_tokens,
        total_tokens: raw_usage.prompt_tokens + raw_usage.completion_tokens,
        cache_read_tokens: raw_usage.cache_read_tokens,
        cache_creation_tokens: raw_usage.cache_creation_tokens,
    };
    state.tracking_service.record(usage_record);

    state.metrics.record_cache(raw_usage.cache_read_tokens, raw_usage.cache_creation_tokens);
    state.metrics.record_tokens(
        provider.name(),
        &response.model,
        raw_usage.prompt_tokens,
        raw_usage.completion_tokens,
    );
    state.metrics.record_request(provider.name(), &response.model, 200);

    Ok(Json(response).into_response())
}

async fn chat_completion_stream_inner(
    state: AppState,
    headers: HeaderMap,
    request: OpenAiChatRequest,
) -> Result<Response<Body>, ProxyError> {
    let provider_name = extract_provider_name(&headers)?;
    validate_provider_name(&provider_name)?;

    let api_key = state
        .config_service
        .get_anthropic_api_key()
        .await
        .map_err(|e| ProxyError::InternalError(e.to_string()))?;

    let provider = Provider::from_name(&provider_name, api_key.clone())?;

    if !provider.accepts_model(&request.model) {
        let accepted = provider.accepted_models().join(", ");
        return Err(ProxyError::ModelNotAccepted {
            model: request.model.clone(),
            provider: provider.name().to_string(),
            accepted,
        });
    }

    let messages = request.messages.clone();
    let model_name = request.model.clone();
    let provider_name_str = provider.name().to_string();

    let anthropic_req = ClaudeAdapter::translate_request(&request)?;
    let claude = ClaudeAdapter::new(api_key);
    let upstream = claude
        .send_stream(anthropic_req, 120)
        .await
        .map_err(ProxyError::from)?;

    if upstream.status() == reqwest::StatusCode::UNAUTHORIZED {
        return Err(ProxyError::UpstreamAuthError);
    }
    if !upstream.status().is_success() {
        return Err(ProxyError::UpstreamServerError(format!(
            "upstream HTTP {}",
            upstream.status().as_u16()
        )));
    }

    let (tx, rx) = mpsc::channel::<Result<Bytes, std::io::Error>>(64);

    let tracking_service = state.tracking_service.clone();
    let metrics = state.metrics.clone();

    tokio::spawn(async move {
        let mut byte_stream = upstream.bytes_stream();
        let mut sse_buf = String::new();
        let mut input_tokens: u32 = 0;
        let mut output_tokens: u32 = 0;
        let mut cache_read_tokens: u32 = 0;
        let mut cache_creation_tokens: u32 = 0;
        let mut resolved_model = model_name.clone();

        while let Some(chunk) = byte_stream.next().await {
            match chunk {
                Ok(bytes) => {
                    if let Ok(text) = std::str::from_utf8(&bytes) {
                        sse_buf.push_str(text);
                        parse_sse_usage(
                            &sse_buf,
                            &mut input_tokens,
                            &mut output_tokens,
                            &mut cache_read_tokens,
                            &mut cache_creation_tokens,
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
                provider: provider_name_str.clone(),
                model: resolved_model.clone(),
                session_id: SessionEngine::derive_session_id(&messages),
                prompt_tokens: input_tokens,
                completion_tokens: output_tokens,
                total_tokens: input_tokens + output_tokens,
                cache_read_tokens,
                cache_creation_tokens,
            };
            tracking_service.record(record);
            metrics.record_cache(cache_read_tokens, cache_creation_tokens);
            metrics.record_tokens(&provider_name_str, &resolved_model, input_tokens, output_tokens);
            metrics.record_request(&provider_name_str, &resolved_model, 200);
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

