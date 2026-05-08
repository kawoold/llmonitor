use sha2::{Digest, Sha256};

use crate::proxy::types::{MessageContent, OpenAiMessage};

pub struct SessionEngine;

impl SessionEngine {
    pub fn derive_session_id(messages: &[OpenAiMessage]) -> String {
        let canonical = messages
            .iter()
            .map(|m| format!("{}:{}", m.role, content_text(&m.content)))
            .collect::<Vec<_>>()
            .join("\n");
        let hash = Sha256::digest(canonical.as_bytes());
        format!("{hash:x}")
    }

    /// Derive a session ID from raw Anthropic-format message values.
    /// Handles both `"content": "string"` and `"content": [{"type":"text","text":"..."}]`.
    pub fn derive_session_id_from_values(messages: &[serde_json::Value]) -> String {
        let canonical = messages
            .iter()
            .map(|m| {
                let role = m.get("role").and_then(|r| r.as_str()).unwrap_or("");
                let text = extract_value_content(m);
                format!("{role}:{text}")
            })
            .collect::<Vec<_>>()
            .join("\n");
        let hash = Sha256::digest(canonical.as_bytes());
        format!("{hash:x}")
    }
}

fn content_text(content: &MessageContent) -> String {
    match content {
        MessageContent::Text(s) => s.clone(),
        MessageContent::Parts(parts) => parts
            .iter()
            .filter_map(|p| p.text.as_deref())
            .collect::<Vec<_>>()
            .join(" "),
    }
}

fn extract_value_content(msg: &serde_json::Value) -> String {
    match msg.get("content") {
        Some(serde_json::Value::String(s)) => s.clone(),
        Some(serde_json::Value::Array(parts)) => parts
            .iter()
            .filter_map(|p| {
                if p.get("type").and_then(|t| t.as_str()) == Some("text") {
                    p.get("text").and_then(|t| t.as_str()).map(String::from)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join(" "),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proxy::types::OpenAiMessage;
    use proptest::prelude::*;

    fn msg(role: &str, content: &str) -> OpenAiMessage {
        OpenAiMessage {
            role: role.to_string(),
            content: MessageContent::Text(content.to_string()),
            name: None,
        }
    }

    #[test]
    fn known_single_message() {
        let messages = vec![msg("user", "hello")];
        let id = SessionEngine::derive_session_id(&messages);
        // SHA-256 of "user:hello"
        assert_eq!(id.len(), 64);
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
        // Same input → same output
        assert_eq!(id, SessionEngine::derive_session_id(&messages));
    }

    #[test]
    fn different_messages_different_id() {
        let a = vec![msg("user", "hello")];
        let b = vec![msg("user", "goodbye")];
        assert_ne!(
            SessionEngine::derive_session_id(&a),
            SessionEngine::derive_session_id(&b)
        );
    }

    #[test]
    fn order_matters() {
        let a = vec![msg("user", "first"), msg("assistant", "second")];
        let b = vec![msg("assistant", "second"), msg("user", "first")];
        assert_ne!(
            SessionEngine::derive_session_id(&a),
            SessionEngine::derive_session_id(&b)
        );
    }

    #[test]
    fn empty_messages() {
        let id = SessionEngine::derive_session_id(&[]);
        assert_eq!(id.len(), 64);
    }

    proptest! {
        #[test]
        fn prop_deterministic(role in "[a-z]{1,10}", content in ".*") {
            let messages = vec![OpenAiMessage {
                role: role.clone(),
                content: MessageContent::Text(content.clone()),
                name: None,
            }];
            let id1 = SessionEngine::derive_session_id(&messages);
            let id2 = SessionEngine::derive_session_id(&messages);
            prop_assert_eq!(id1, id2);
        }

        #[test]
        fn prop_always_64_hex_chars(role in "[a-z]{1,10}", content in ".*") {
            let messages = vec![OpenAiMessage {
                role,
                content: MessageContent::Text(content),
                name: None,
            }];
            let id = SessionEngine::derive_session_id(&messages);
            prop_assert_eq!(id.len(), 64);
            prop_assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
        }
    }
}
