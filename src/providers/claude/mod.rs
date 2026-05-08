pub mod types;

use chrono::Utc;
use reqwest::StatusCode;
use tracing::info;
use uuid::Uuid;

use crate::config::types::ApiKey;
use crate::proxy::types::{
    Choice, MessageContent, OpenAiChatRequest, OpenAiChatResponse, OpenAiMessage, OpenAiUsage,
    RawUsage,
};
use crate::providers::ProviderError;

use types::{
    AnthropicCacheControl, AnthropicCacheControlHint, AnthropicContentBlock, AnthropicMessage,
    AnthropicRequest, AnthropicResponse, AnthropicSystem, DEFAULT_MAX_TOKENS,
    ANTHROPIC_API_URL, ANTHROPIC_VERSION,
};

#[derive(Debug, Clone)]
pub struct ClaudeAdapter {
    pub(crate) api_key: ApiKey,
    client: reqwest::Client,
}

impl ClaudeAdapter {
    pub fn new(api_key: ApiKey) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
        }
    }

    // ── Request translation ──────────────────────────────────────────────────

    pub fn translate_request(req: &OpenAiChatRequest) -> Result<AnthropicRequest, ProviderError> {
        // Parse cache control hint (fail-fast on malformed JSON)
        let cache_hint: AnthropicCacheControlHint = match req.extra.get("anthropic_cache_control") {
            None => AnthropicCacheControlHint::default(),
            Some(v) => serde_json::from_value(v.clone()).map_err(|e| {
                ProviderError::ClientError(format!("malformed anthropic_cache_control: {e}"))
            })?,
        };

        // Extract and concatenate system messages (BR-U2-01)
        let system_text: String = req
            .messages
            .iter()
            .filter(|m| m.role == "system")
            .map(|m| match &m.content {
                MessageContent::Text(t) => t.as_str(),
                MessageContent::Parts(_) => "",
            })
            .collect::<Vec<_>>()
            .join("\n\n");

        // Resolve model alias (BR-U2-03)
        let model = types::resolve_model(&req.model).to_string();

        // Translate non-system messages (BR-U2-11)
        let mut anthropic_messages: Vec<AnthropicMessage> = Vec::new();
        for msg in req.messages.iter().filter(|m| m.role != "system") {
            let role = match msg.role.as_str() {
                "user" | "assistant" => msg.role.clone(),
                other => {
                    return Err(ProviderError::ClientError(format!(
                        "unsupported message role: \"{other}\"; only \"user\" and \"assistant\" are allowed"
                    )))
                }
            };

            let content: Vec<AnthropicContentBlock> = match &msg.content {
                MessageContent::Text(t) => vec![AnthropicContentBlock {
                    block_type: "text".to_string(),
                    text: Some(t.clone()),
                    cache_control: None,
                }],
                MessageContent::Parts(parts) => parts
                    .iter()
                    .map(|p| AnthropicContentBlock {
                        block_type: "text".to_string(),
                        text: p.text.clone(),
                        cache_control: None,
                    })
                    .collect(),
            };

            anthropic_messages.push(AnthropicMessage { role, content });
        }

        // Apply cache control hints (BR-U2-04)
        let ephemeral = || Some(AnthropicCacheControl {
            control_type: "ephemeral".to_string(),
        });

        let system: Option<AnthropicSystem> = if system_text.is_empty() {
            None
        } else if cache_hint.system {
            // Wrap system in block form so we can attach cache_control to the last block
            let mut blocks = vec![AnthropicContentBlock {
                block_type: "text".to_string(),
                text: Some(system_text),
                cache_control: None,
            }];
            if let Some(last) = blocks.last_mut() {
                last.cache_control = ephemeral();
            }
            Some(AnthropicSystem::Blocks(blocks))
        } else {
            Some(AnthropicSystem::Text(system_text))
        };

        for &idx in &cache_hint.messages {
            if let Some(msg) = anthropic_messages.get_mut(idx) {
                if let Some(last_block) = msg.content.last_mut() {
                    last_block.cache_control = ephemeral();
                }
            }
            // out-of-range indices silently ignored (BR-U2-04)
        }

        Ok(AnthropicRequest {
            model,
            messages: anthropic_messages,
            system,
            max_tokens: req.max_tokens.unwrap_or(DEFAULT_MAX_TOKENS),
            temperature: req.temperature,
            top_p: req.top_p,
            stop_sequences: req.stop.clone(),
            stream: req.stream,
        })
    }

    // ── Response translation ─────────────────────────────────────────────────

    pub fn translate_response(resp: AnthropicResponse) -> (OpenAiChatResponse, RawUsage) {
        // Concatenate text blocks
        let content: String = resp
            .content
            .iter()
            .filter(|b| b.block_type == "text")
            .map(|b| b.text.as_str())
            .collect::<Vec<_>>()
            .join("");

        // Map stop reason (BR-U2-12)
        let finish_reason = match resp.stop_reason.as_deref() {
            Some("end_turn") | Some("stop_sequence") | None => "stop",
            Some("max_tokens") => "length",
            Some("tool_use") => "tool_calls",
            Some(_) => "stop",
        }
        .to_string();

        let prompt_tokens = resp.usage.input_tokens;
        let completion_tokens = resp.usage.output_tokens;

        let oai_response = OpenAiChatResponse {
            id: format!("chatcmpl-{}", Uuid::new_v4()),
            object: "chat.completion".to_string(),
            created: Utc::now().timestamp(),
            model: resp.model,
            choices: vec![Choice {
                index: 0,
                message: OpenAiMessage {
                    role: "assistant".to_string(),
                    content: MessageContent::Text(content),
                    name: None,
                },
                finish_reason: Some(finish_reason),
            }],
            usage: OpenAiUsage {
                prompt_tokens,
                completion_tokens,
                total_tokens: prompt_tokens + completion_tokens,
            },
        };

        let raw_usage = RawUsage {
            prompt_tokens,
            completion_tokens,
            cache_read_tokens: resp.usage.cache_read_input_tokens.unwrap_or(0),
            cache_creation_tokens: resp.usage.cache_creation_input_tokens.unwrap_or(0),
        };

        (oai_response, raw_usage)
    }

    // ── HTTP call ────────────────────────────────────────────────────────────

    pub async fn send_stream(
        &self,
        req: AnthropicRequest,
        timeout_secs: u64,
    ) -> Result<reqwest::Response, ProviderError> {
        self.client
            .post(ANTHROPIC_API_URL)
            .header("x-api-key", self.api_key.0.as_str())
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("content-type", "application/json")
            .header("accept", "text/event-stream")
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .json(&req)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    ProviderError::Timeout
                } else {
                    ProviderError::ServerError(e.to_string())
                }
            })
    }

    pub async fn send(
        &self,
        req: AnthropicRequest,
        timeout_secs: u64,
    ) -> Result<AnthropicResponse, ProviderError> {
        let model = req.model.clone();

        let result = self
            .client
            .post(ANTHROPIC_API_URL)
            .header("x-api-key", self.api_key.0.as_str())
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("content-type", "application/json")
            .header("accept", "application/json")
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .json(&req)
            .send()
            .await;

        let response = match result {
            Ok(r) => r,
            Err(e) if e.is_timeout() => return Err(ProviderError::Timeout),
            Err(_) => return Err(ProviderError::Timeout),
        };

        let status = response.status();

        match status {
            StatusCode::OK => {
                let body: AnthropicResponse = response
                    .json()
                    .await
                    .map_err(|e| ProviderError::ServerError(format!("deserialize error: {e}")))?;

                info!(
                    model = %model,
                    http_status = %status.as_u16(),
                    input_tokens = %body.usage.input_tokens,
                    output_tokens = %body.usage.output_tokens,
                    cache_read = %body.usage.cache_read_input_tokens.unwrap_or(0),
                    cache_creation = %body.usage.cache_creation_input_tokens.unwrap_or(0),
                    "anthropic request succeeded"
                );

                Ok(body)
            }
            StatusCode::UNAUTHORIZED => Err(ProviderError::AuthFailed),
            StatusCode::TOO_MANY_REQUESTS => Err(ProviderError::RateLimited),
            s if s.as_u16() == 529 => Err(ProviderError::RateLimited),
            StatusCode::BAD_REQUEST => {
                let msg = response
                    .json::<types::AnthropicErrorResponse>()
                    .await
                    .map(|e| e.error.message)
                    .unwrap_or_else(|_| "bad request".to_string());
                Err(ProviderError::ClientError(msg))
            }
            s => {
                let msg = format!("upstream HTTP {}", s.as_u16());
                Err(ProviderError::ServerError(msg))
            }
        }
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proxy::types::OpenAiMessage;
    use serde_json::json;
    use std::collections::HashMap;

    fn text_msg(role: &str, content: &str) -> OpenAiMessage {
        OpenAiMessage {
            role: role.to_string(),
            content: MessageContent::Text(content.to_string()),
            name: None,
        }
    }

    fn base_request(messages: Vec<OpenAiMessage>) -> OpenAiChatRequest {
        OpenAiChatRequest {
            model: "claude-sonnet-4-6".to_string(),
            messages,
            stream: None,
            max_tokens: None,
            temperature: None,
            top_p: None,
            stop: None,
            user: None,
            extra: HashMap::new(),
        }
    }

    // resolve_model ─────────────────────────────────────────────────────────

    #[test]
    fn test_resolve_model_alias() {
        assert_eq!(types::resolve_model("claude-sonnet-4"), "claude-sonnet-4-6");
        assert_eq!(types::resolve_model("claude-opus-4"), "claude-opus-4-7");
        assert_eq!(types::resolve_model("claude-3-opus"), "claude-3-opus-20240229");
    }

    #[test]
    fn test_resolve_model_passthrough() {
        assert_eq!(types::resolve_model("claude-sonnet-4-6"), "claude-sonnet-4-6");
        assert_eq!(types::resolve_model("unknown-model"), "unknown-model");
    }

    // translate_request ─────────────────────────────────────────────────────

    #[test]
    fn test_translate_request_system_concat() {
        let mut req = base_request(vec![
            text_msg("system", "You are helpful."),
            text_msg("system", "Be concise."),
            text_msg("user", "Hello"),
        ]);
        req.model = "claude-sonnet-4-6".to_string();

        let result = ClaudeAdapter::translate_request(&req).unwrap();

        assert_eq!(result.messages.len(), 1);
        match result.system.unwrap() {
            AnthropicSystem::Text(t) => assert_eq!(t, "You are helpful.\n\nBe concise."),
            AnthropicSystem::Blocks(_) => panic!("expected Text variant"),
        }
    }

    #[test]
    fn test_translate_request_default_max_tokens() {
        let req = base_request(vec![text_msg("user", "Hi")]);
        let result = ClaudeAdapter::translate_request(&req).unwrap();
        assert_eq!(result.max_tokens, 4096);
    }

    #[test]
    fn test_translate_request_explicit_max_tokens() {
        let mut req = base_request(vec![text_msg("user", "Hi")]);
        req.max_tokens = Some(1024);
        let result = ClaudeAdapter::translate_request(&req).unwrap();
        assert_eq!(result.max_tokens, 1024);
    }

    #[test]
    fn test_translate_request_model_alias() {
        let mut req = base_request(vec![text_msg("user", "Hi")]);
        req.model = "claude-sonnet-4".to_string();
        let result = ClaudeAdapter::translate_request(&req).unwrap();
        assert_eq!(result.model, "claude-sonnet-4-6");
    }

    #[test]
    fn test_translate_request_unknown_role() {
        let req = base_request(vec![text_msg("tool", "some tool output")]);
        let err = ClaudeAdapter::translate_request(&req).unwrap_err();
        assert!(matches!(err, ProviderError::ClientError(_)));
    }

    #[test]
    fn test_translate_request_cache_control_system() {
        let mut req = base_request(vec![
            text_msg("system", "Be helpful."),
            text_msg("user", "Hello"),
        ]);
        req.extra.insert(
            "anthropic_cache_control".to_string(),
            json!({"system": true}),
        );

        let result = ClaudeAdapter::translate_request(&req).unwrap();

        match result.system.unwrap() {
            AnthropicSystem::Blocks(blocks) => {
                assert!(blocks.last().unwrap().cache_control.is_some());
            }
            AnthropicSystem::Text(_) => panic!("expected Blocks variant"),
        }
    }

    #[test]
    fn test_translate_request_cache_control_message() {
        let mut req = base_request(vec![
            text_msg("user", "Hello"),
            text_msg("assistant", "Hi there"),
        ]);
        req.extra.insert(
            "anthropic_cache_control".to_string(),
            json!({"messages": [0]}),
        );

        let result = ClaudeAdapter::translate_request(&req).unwrap();

        let last_block = result.messages[0].content.last().unwrap();
        assert!(last_block.cache_control.is_some());
        assert!(result.messages[1].content.last().unwrap().cache_control.is_none());
    }

    #[test]
    fn test_translate_request_cache_control_oob() {
        let mut req = base_request(vec![text_msg("user", "Hello")]);
        req.extra.insert(
            "anthropic_cache_control".to_string(),
            json!({"messages": [99]}),
        );

        // Should succeed silently (out-of-range index ignored)
        let result = ClaudeAdapter::translate_request(&req);
        assert!(result.is_ok());
    }

    #[test]
    fn test_translate_request_malformed_cache_control() {
        let mut req = base_request(vec![text_msg("user", "Hello")]);
        req.extra.insert(
            "anthropic_cache_control".to_string(),
            json!({"system": "not-a-bool"}),
        );

        let err = ClaudeAdapter::translate_request(&req).unwrap_err();
        assert!(matches!(err, ProviderError::ClientError(_)));
    }

    // translate_response ────────────────────────────────────────────────────

    fn make_response(stop_reason: Option<&str>, cache_read: Option<u32>, cache_creation: Option<u32>) -> AnthropicResponse {
        AnthropicResponse {
            id: "msg_test".to_string(),
            response_type: "message".to_string(),
            role: "assistant".to_string(),
            content: vec![types::AnthropicResponseBlock {
                block_type: "text".to_string(),
                text: "Hello!".to_string(),
            }],
            model: "claude-sonnet-4-6".to_string(),
            stop_reason: stop_reason.map(str::to_string),
            usage: types::AnthropicUsage {
                input_tokens: 10,
                output_tokens: 5,
                cache_read_input_tokens: cache_read,
                cache_creation_input_tokens: cache_creation,
            },
        }
    }

    #[test]
    fn test_translate_response_stop_reasons() {
        let cases = [
            (Some("end_turn"), "stop"),
            (Some("max_tokens"), "length"),
            (Some("stop_sequence"), "stop"),
            (Some("tool_use"), "tool_calls"),
            (None, "stop"),
            (Some("unknown"), "stop"),
        ];

        for (stop_reason, expected_finish) in cases {
            let (resp, _) = ClaudeAdapter::translate_response(make_response(stop_reason, None, None));
            assert_eq!(
                resp.choices[0].finish_reason.as_deref(),
                Some(expected_finish),
                "stop_reason={stop_reason:?}"
            );
        }
    }

    #[test]
    fn test_translate_response_id_format() {
        let (resp, _) = ClaudeAdapter::translate_response(make_response(Some("end_turn"), None, None));
        assert!(resp.id.starts_with("chatcmpl-"), "id was: {}", resp.id);
    }

    #[test]
    fn test_translate_response_cache_tokens_absent() {
        let (_, raw) = ClaudeAdapter::translate_response(make_response(Some("end_turn"), None, None));
        assert_eq!(raw.cache_read_tokens, 0);
        assert_eq!(raw.cache_creation_tokens, 0);
    }

    #[test]
    fn test_translate_response_cache_tokens_present() {
        let (_, raw) = ClaudeAdapter::translate_response(make_response(Some("end_turn"), Some(100), Some(200)));
        assert_eq!(raw.cache_read_tokens, 100);
        assert_eq!(raw.cache_creation_tokens, 200);
    }

    #[test]
    fn test_translate_response_usage() {
        let (resp, raw) = ClaudeAdapter::translate_response(make_response(Some("end_turn"), None, None));
        assert_eq!(resp.usage.prompt_tokens, 10);
        assert_eq!(resp.usage.completion_tokens, 5);
        assert_eq!(resp.usage.total_tokens, 15);
        assert_eq!(raw.prompt_tokens, 10);
        assert_eq!(raw.completion_tokens, 5);
    }
}
