# Domain Entities — Unit 1: Proxy Core

## OpenAI API Schema Types (`src/proxy/types.rs`)

### OpenAiChatRequest
```
OpenAiChatRequest {
  model:             String                    -- required, non-empty
  messages:          Vec<OpenAiMessage>        -- required, non-empty
  temperature:       Option<f32>              -- 0.0–2.0, passed through
  max_tokens:        Option<u32>              -- passed through
  stream:            Option<bool>             -- default false
  top_p:             Option<f32>              -- passed through
  stop:              Option<Vec<String>>      -- passed through
  system:            Option<String>           -- OpenAI doesn't have this but Anthropic does; accepted as extension
  extra_fields:      serde_json::Value        -- all unknown fields captured for passthrough
}
```

### OpenAiMessage
```
OpenAiMessage {
  role:     String                            -- "user" | "assistant" | "system"
  content:  MessageContent                   -- String or Vec<ContentPart>
}

MessageContent = String | Vec<ContentPart>

ContentPart {
  type:  String                              -- "text" | "image_url"
  text:  Option<String>
  image_url: Option<ImageUrl>
}
```

### OpenAiChatResponse
```
OpenAiChatResponse {
  id:      String                            -- "chatcmpl-<uuid>"
  object:  String                            -- "chat.completion"
  created: i64                               -- Unix timestamp
  model:   String                            -- model name as returned by provider
  choices: Vec<Choice>
  usage:   OpenAiUsage
}

Choice {
  index:         u32
  message:       OpenAiMessage
  finish_reason: String                      -- "stop" | "length" | "tool_calls"
  logprobs:      Option<serde_json::Value>   -- null
}

OpenAiUsage {
  prompt_tokens:     u64
  completion_tokens: u64
  total_tokens:      u64
}
```

### OpenAiStreamChunk
```
OpenAiStreamChunk {
  id:      String
  object:  String                            -- "chat.completion.chunk"
  created: i64
  model:   String
  choices: Vec<StreamChoice>
  usage:   Option<OpenAiUsage>               -- only present in final chunk
}

StreamChoice {
  index:         u32
  delta:         MessageDelta
  finish_reason: Option<String>
}

MessageDelta {
  role:    Option<String>                    -- only in first chunk
  content: Option<String>                    -- incremental content
}
```

---

## Error Types (`src/proxy/error.rs`)

### ProxyError (internal enum)
```
ProxyError {
  ProviderHeaderMissing,
  ProviderUnknown { name: String },
  ProviderNotConfigured { name: String },
  ModelNotAccepted { model: String, provider: String, accepted: Vec<String> },
  RequestParseError { detail: String },
  RateLimitExceeded { retry_after_secs: u64 },
  UpstreamClientError { status: u16, message: String },
  UpstreamAuthError,
  UpstreamRateLimit,
  UpstreamServerError { status: u16 },
  UpstreamTimeout,
  InternalError { source: String },  -- source is NEVER sent to client
}
```

### OpenAiErrorBody (sent to clients)
```
OpenAiErrorBody {
  error: OpenAiErrorDetail
}

OpenAiErrorDetail {
  type:    String
  message: String
  param:   Option<String>
  code:    Option<String>
}
```

---

## Health Types (`src/proxy/health.rs`)

```
HealthResponse {
  status:      HealthStatus        -- "ok" | "degraded"
  db:          DbStatus            -- "connected" | "error"
  uptime_secs: u64
}

HealthStatus = Ok | Degraded
DbStatus     = Connected | Error
```

---

## Application State (`src/app_state.rs`)

```
AppState {
  db:       SqlitePool
  tracking: Arc<TrackingService>
  metrics:  Arc<MetricsRegistry>
  config:   Arc<ConfigService>
  stats:    Arc<StatsService>
  started_at: Instant              -- for uptime calculation
}
```

---

## Provider Abstraction (`src/providers/mod.rs`)

```
Provider (enum) {
  Claude(ClaudeAdapter)
}

Provider methods:
  from_name(name: &str, config: &Config) -> Result<Provider, ProxyError>
  name(&self) -> &str
  accepted_models(&self) -> &[&str]
  chat_completion(request: OpenAiChatRequest) -> Result<(OpenAiChatResponse, RawUsage), ProviderError>
  chat_completion_stream(request: OpenAiChatRequest) -> Result<BoxStream<...>, ProviderError>
```

---

## Middleware State

```
RequestId(Uuid)                    -- injected by request-id middleware, available as request extension

RateLimitState {
  tokens:       AtomicU64          -- current token bucket level
  last_refill:  Instant
  capacity:     u64                -- max tokens (= rate limit)
  refill_rate:  u64                -- tokens added per second
}
```
