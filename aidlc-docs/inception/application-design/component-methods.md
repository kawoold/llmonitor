# Component Methods — llmonitor LLM Proxy

Method signatures are expressed in Rust. Detailed business logic is deferred to Functional Design (Construction phase).

---

## C-01: HttpServer

```rust
// src/main.rs
pub async fn run(config: ServerConfig) -> anyhow::Result<()>;
pub fn build_router(state: Arc<AppState>) -> Router;
pub async fn shutdown_signal();
```

```rust
// src/middleware/
pub fn security_headers_layer() -> impl Layer<...>;
pub fn request_id_layer() -> impl Layer<...>;
pub fn rate_limit_layer(config: &RateLimitConfig) -> impl Layer<...>;
```

---

## C-02: ProxyHandler

```rust
// src/proxy/handlers.rs
pub async fn chat_completion(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<OpenAiChatRequest>,
) -> Result<Json<OpenAiChatResponse>, ProxyError>;

pub async fn chat_completion_stream(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<OpenAiChatRequest>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, ProxyError>;
```

```rust
// src/proxy/routing.rs
pub fn resolve_provider(
    headers: &HeaderMap,
    config: &Config,
) -> Result<Provider, ProxyError>;
```

```rust
// src/proxy/error.rs
pub fn into_openai_error_response(err: ProxyError) -> (StatusCode, Json<OpenAiErrorBody>);
```

---

## C-03: Provider (Enum)

```rust
// src/providers/mod.rs
pub enum Provider {
    Claude(ClaudeAdapter),
}

impl Provider {
    pub fn from_name(name: &str, config: &Config) -> Result<Self, ProxyError>;
    pub fn name(&self) -> &str;

    pub async fn chat_completion(
        &self,
        request: OpenAiChatRequest,
    ) -> Result<(OpenAiChatResponse, RawUsage), ProviderError>;

    pub async fn chat_completion_stream(
        &self,
        request: OpenAiChatRequest,
    ) -> Result<impl Stream<Item = Result<OpenAiStreamChunk, ProviderError>>, ProviderError>;
}
```

---

## C-04: ClaudeAdapter

```rust
// src/providers/claude/mod.rs
pub struct ClaudeAdapter {
    api_key: String,
    client: reqwest::Client,
    base_url: String,
}

impl ClaudeAdapter {
    pub fn new(api_key: String) -> Self;

    // Translation
    pub fn translate_request(req: &OpenAiChatRequest) -> AnthropicRequest;
    pub fn translate_response(resp: AnthropicResponse) -> (OpenAiChatResponse, RawUsage);
    pub fn translate_stream_chunk(chunk: AnthropicStreamEvent) -> Option<OpenAiStreamChunk>;

    // HTTP
    pub async fn send(
        &self,
        req: AnthropicRequest,
    ) -> Result<AnthropicResponse, ProviderError>;

    pub async fn send_stream(
        &self,
        req: AnthropicRequest,
    ) -> Result<impl Stream<Item = Result<AnthropicStreamEvent, ProviderError>>, ProviderError>;
}
```

---

## C-05: SessionEngine

```rust
// src/tracking/session.rs
pub struct SessionId(pub String);  // hex-encoded SHA-256

pub fn derive_session_id(messages: &[OpenAiMessage]) -> SessionId;
// Returns zero/nil session ID string if messages has 0 or 1 entries (no prior context to hash)
```

---

## C-06: TrackingService

```rust
// src/tracking/service.rs
pub struct TrackingService {
    sender: mpsc::Sender<UsageRecord>,
}

impl TrackingService {
    pub fn new(sender: mpsc::Sender<UsageRecord>) -> Self;
    pub fn record(&self, record: UsageRecord);
    // Non-blocking: logs and drops the record if channel is full
}
```

---

## C-07: TrackingWriterService

```rust
// src/tracking/writer.rs
pub struct TrackingWriterService {
    receiver: mpsc::Receiver<UsageRecord>,
    db: SqlitePool,
    batch_size: usize,
    flush_interval: Duration,
}

impl TrackingWriterService {
    pub fn new(receiver: mpsc::Receiver<UsageRecord>, db: SqlitePool) -> Self;
    pub async fn run(self);
    // Loops: recv batch or timeout → sqlx insert → repeat until channel closed
}
```

---

## C-08: StatsService

```rust
// src/stats/service.rs
pub struct StatsService {
    db: SqlitePool,
}

impl StatsService {
    pub fn new(db: SqlitePool) -> Self;

    pub async fn summary(&self) -> Result<StatsSummary, StatsError>;
    pub async fn usage_timeseries(&self, window: TimeWindow) -> Result<Vec<UsageDataPoint>, StatsError>;
    pub async fn session_stats(&self, window: TimeWindow) -> Result<Vec<SessionStats>, StatsError>;
    pub async fn cache_stats(&self, window: TimeWindow) -> Result<CacheStats, StatsError>;
}

pub enum TimeWindow { OneHour, OneDay, SevenDays, ThirtyDays }
```

---

## C-09: MetricsRegistry

```rust
// src/metrics/registry.rs
pub struct MetricsRegistry {
    requests_total: IntCounterVec,        // labels: provider, model, status
    tokens_total: IntCounterVec,          // labels: provider, model, type (input|output)
    request_duration: HistogramVec,       // labels: provider, model
    cache_tokens_total: IntCounterVec,    // labels: type (hit|miss)
    active_sessions: IntGauge,
}

impl MetricsRegistry {
    pub fn new() -> Result<Self, prometheus::Error>;

    pub fn record_request(
        &self,
        provider: &str,
        model: &str,
        status: u16,
        duration: Duration,
    );

    pub fn record_tokens(
        &self,
        provider: &str,
        model: &str,
        input_tokens: u64,
        output_tokens: u64,
    );

    pub fn record_cache(&self, hit_tokens: u64, miss_tokens: u64);
    pub fn set_active_sessions(&self, count: i64);
    pub fn render(&self) -> Result<String, prometheus::Error>;
}
```

---

## C-10: ConfigService

```rust
// src/config/service.rs
pub struct ConfigService {
    db: SqlitePool,
}

impl ConfigService {
    pub fn new(db: SqlitePool) -> Self;

    pub async fn get_config(&self) -> Result<Config, ConfigError>;
    pub async fn get_config_redacted(&self) -> Result<RedactedConfig, ConfigError>;
    pub async fn update_config(&self, update: ConfigUpdate) -> Result<(), ConfigError>;
    pub async fn update_admin_credentials(
        &self,
        update: CredentialsUpdate,
    ) -> Result<(), ConfigError>;
    pub async fn verify_admin_password(
        &self,
        username: &str,
        password: &str,
    ) -> Result<bool, ConfigError>;
    pub async fn admin_exists(&self) -> Result<bool, ConfigError>;
}
```

---

## C-11: AuthMiddleware

```rust
// src/auth/middleware.rs
pub fn auth_middleware_layer(
    config_service: Arc<ConfigService>,
) -> impl Layer<...>;

// Internal per-request function
async fn verify_basic_auth(
    req: Request,
    config_service: Arc<ConfigService>,
    next: Next,
) -> Result<Response, AuthError>;
```

---

## C-12: FrontendAssets

```rust
// src/proxy/frontend.rs
#[derive(RustEmbed)]
#[folder = "frontend/dist/"]
struct FrontendAsset;

pub async fn serve_asset(uri: Uri) -> impl IntoResponse;
pub async fn serve_index() -> impl IntoResponse;
// index.html fallback for SPA client-side routing
```

---

## C-13: AdminBootstrapService

```rust
// src/config/bootstrap.rs
pub async fn bootstrap(db: &SqlitePool) -> anyhow::Result<()>;
// 1. Check admin_exists() via ConfigService
// 2. If not: read LLMONITOR_ADMIN_USER + LLMONITOR_ADMIN_PASSWORD env vars
// 3. Hash password with Argon2id, persist via ConfigService
// 4. Error if no admin and env vars absent
```

---

## Key Shared Types

```rust
// src/proxy/types.rs — OpenAI API schema
pub struct OpenAiChatRequest { pub model: String, pub messages: Vec<OpenAiMessage>, ... }
pub struct OpenAiChatResponse { pub choices: Vec<Choice>, pub usage: OpenAiUsage, ... }
pub struct OpenAiMessage { pub role: String, pub content: String }
pub struct OpenAiUsage { pub prompt_tokens: u64, pub completion_tokens: u64, pub total_tokens: u64 }

// src/providers/claude/types.rs — Anthropic API schema
pub struct AnthropicRequest { pub model: String, pub messages: Vec<AnthropicMessage>, ... }
pub struct AnthropicResponse { pub content: Vec<ContentBlock>, pub usage: AnthropicUsage, ... }
pub struct AnthropicUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_input_tokens: Option<u64>,
    pub cache_creation_input_tokens: Option<u64>,
}

// src/tracking/types.rs — persisted usage record
pub struct UsageRecord {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub provider: String,
    pub model: String,
    pub session_id: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub total_tokens: i64,
    pub cache_hit_tokens: i64,
    pub cache_miss_tokens: i64,
    pub duration_ms: i64,
    pub http_status: i16,
}

// src/providers/mod.rs — raw usage from provider response (pre-session-id)
pub struct RawUsage {
    pub model: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_hit_tokens: u64,
    pub cache_miss_tokens: u64,
    pub duration_ms: u64,
    pub http_status: u16,
}
```
