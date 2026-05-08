use proptest::prelude::*;

use crate::proxy::types::{MessageContent, OpenAiChatRequest, OpenAiMessage};

proptest! {
    #[test]
    fn chat_request_roundtrip(
        model in "[a-z]{3,20}",
        content in "[a-zA-Z0-9 ]{1,100}",
    ) {
        let request = OpenAiChatRequest {
            model,
            messages: vec![OpenAiMessage {
                role: "user".to_string(),
                content: MessageContent::Text(content),
                name: None,
            }],
            stream: None,
            max_tokens: None,
            temperature: None,
            top_p: None,
            stop: None,
            user: None,
            extra: Default::default(),
        };
        let json = serde_json::to_string(&request).unwrap();
        let decoded: OpenAiChatRequest = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&request.model, &decoded.model);
    }

    #[test]
    fn message_content_text_roundtrip(text in "[a-zA-Z0-9 !?.]{1,200}") {
        let content = MessageContent::Text(text.clone());
        let json = serde_json::to_string(&content).unwrap();
        let decoded: MessageContent = serde_json::from_str(&json).unwrap();
        match decoded {
            MessageContent::Text(t) => prop_assert_eq!(t, text),
            _ => return Err(TestCaseError::fail("Expected Text variant")),
        }
    }
}
