use std::time::Duration;

pub const CHANNEL_CAPACITY: usize = 1_000;
pub const FLUSH_INTERVAL: Duration = Duration::from_millis(500);
pub const BATCH_SIZE: usize = 50;

#[derive(Debug, Clone)]
pub struct UsageRecord {
    pub id: String,
    pub created_at: String,
    pub provider: String,
    pub model: String,
    pub session_id: String,
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
    pub cache_read_tokens: u32,
    pub cache_creation_tokens: u32,
}
