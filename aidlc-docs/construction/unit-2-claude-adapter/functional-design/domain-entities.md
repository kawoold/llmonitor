# Domain Entities — Unit 2: Claude Provider Adapter

## Anthropic API Request Types

```rust
// src/providers/claude/types.rs

#[derive(Debug, Serialize)]
pub struct AnthropicRequest {
    pub model: String,
    pub messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<AnthropicSystem>,
    pub max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

// Anthropic system field accepts either a plain string or a list of content blocks
// (list form is needed for cache_control on the system prompt)
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum AnthropicSystem {
    Text(String),
    Blocks(Vec<AnthropicContentBlock>),
}

#[derive(Debug, Serialize, Clone)]
pub struct AnthropicMessage {
    pub role: String,  // "user" | "assistant"
    pub content: Vec<AnthropicContentBlock>,
}

#[derive(Debug, Serialize, Clone)]
pub struct AnthropicContentBlock {
    #[serde(rename = "type")]
    pub block_type: String,  // "text" | "image"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<AnthropicCacheControl>,
}

#[derive(Debug, Serialize, Clone)]
pub struct AnthropicCacheControl {
    #[serde(rename = "type")]
    pub control_type: String,  // always "ephemeral"
}
```

## Anthropic API Response Types

```rust
#[derive(Debug, Deserialize)]
pub struct AnthropicResponse {
    pub id: String,
    #[serde(rename = "type")]
    pub response_type: String,  // "message"
    pub role: String,           // "assistant"
    pub content: Vec<AnthropicResponseBlock>,
    pub model: String,
    pub stop_reason: Option<String>,
    pub usage: AnthropicUsage,
}

#[derive(Debug, Deserialize)]
pub struct AnthropicResponseBlock {
    #[serde(rename = "type")]
    pub block_type: String,
    #[serde(default)]
    pub text: String,
}

#[derive(Debug, Deserialize)]
pub struct AnthropicUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cache_read_input_tokens: Option<u32>,
    pub cache_creation_input_tokens: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct AnthropicErrorResponse {
    #[serde(rename = "type")]
    pub error_type: String,
    pub error: AnthropicErrorDetail,
}

#[derive(Debug, Deserialize)]
pub struct AnthropicErrorDetail {
    #[serde(rename = "type")]
    pub detail_type: String,
    pub message: String,
}
```

## Cache Control Extension (Client-facing)

```rust
// Parsed from OpenAiChatRequest.extra["anthropic_cache_control"]
#[derive(Debug, Deserialize, Default)]
pub struct AnthropicCacheControlHint {
    /// Cache the system prompt
    #[serde(default)]
    pub system: bool,
    /// Indices into the OpenAI messages array (after system messages removed)
    /// to apply cache_control on
    #[serde(default)]
    pub messages: Vec<usize>,
}
```

**Client example:**
```json
{
  "model": "claude-sonnet-4-6",
  "messages": [
    {"role": "user", "content": "Tell me about Rust."}
  ],
  "anthropic_cache_control": {"system": true, "messages": [0]}
}
```

## Model Alias Map

```rust
// src/providers/claude/types.rs
pub const MODEL_ALIAS_MAP: &[(&str, &str)] = &[
    ("claude-3-opus",       "claude-3-opus-20240229"),
    ("claude-3-sonnet",     "claude-3-sonnet-20240229"),
    ("claude-3-haiku",      "claude-3-haiku-20240307"),
    ("claude-3-5-sonnet",   "claude-3-5-sonnet-20241022"),
    ("claude-3-5-haiku",    "claude-3-5-haiku-20241022"),
    ("claude-sonnet-4",     "claude-sonnet-4-6"),
    ("claude-opus-4",       "claude-opus-4-7"),
    ("claude-haiku-4",      "claude-haiku-4-5-20251001"),
];

pub fn resolve_model(name: &str) -> &str {
    MODEL_ALIAS_MAP
        .iter()
        .find(|(alias, _)| *alias == name)
        .map(|(_, canonical)| *canonical)
        .unwrap_or(name)
}
```

## ClaudeAdapter Struct

```rust
// src/providers/claude/mod.rs
pub struct ClaudeAdapter {
    pub(crate) api_key: ApiKey,
    client: reqwest::Client,
}

impl ClaudeAdapter {
    pub fn new(api_key: ApiKey) -> Self;
    pub async fn send(&self, req: AnthropicRequest) -> Result<AnthropicResponse, ProviderError>;

    // Translation functions (pure — no I/O)
    pub fn translate_request(req: &OpenAiChatRequest) -> AnthropicRequest;
    pub fn translate_response(resp: AnthropicResponse) -> (OpenAiChatResponse, RawUsage);
}
```

## Constants

```rust
pub const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
pub const ANTHROPIC_VERSION: &str = "2023-06-01";
pub const DEFAULT_MAX_TOKENS: u32 = 4096;
```
