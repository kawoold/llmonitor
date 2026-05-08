use std::sync::Arc;

use tokio::sync::mpsc;
use tracing::{error, warn};

use super::types::UsageRecord;

#[derive(Clone)]
pub struct TrackingService {
    sender: Arc<mpsc::Sender<UsageRecord>>,
}

impl TrackingService {
    pub fn new(sender: Arc<mpsc::Sender<UsageRecord>>) -> Self {
        Self { sender }
    }

    pub fn record(&self, record: UsageRecord) {
        match self.sender.try_send(record) {
            Ok(()) => {}
            Err(mpsc::error::TrySendError::Full(_)) => {
                warn!("tracking channel full, dropping usage record");
            }
            Err(mpsc::error::TrySendError::Closed(_)) => {
                error!("tracking channel closed unexpectedly");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tracking::types::CHANNEL_CAPACITY;

    fn dummy_record() -> UsageRecord {
        UsageRecord {
            id: "test-id".to_string(),
            created_at: "2026-01-01T00:00:00Z".to_string(),
            provider: "claude".to_string(),
            model: "claude-sonnet-4-6".to_string(),
            session_id: "a".repeat(64),
            prompt_tokens: 10,
            completion_tokens: 5,
            total_tokens: 15,
            cache_read_tokens: 0,
            cache_creation_tokens: 0,
        }
    }

    #[tokio::test]
    async fn record_sends_to_channel() {
        let (tx, mut rx) = mpsc::channel::<UsageRecord>(10);
        let svc = TrackingService::new(Arc::new(tx));
        svc.record(dummy_record());
        let received = rx.recv().await;
        assert!(received.is_some());
    }

    #[tokio::test]
    async fn record_on_full_channel_does_not_panic() {
        // capacity=0 is not allowed; use capacity=1, fill it, then try again
        let (tx, _rx) = mpsc::channel::<UsageRecord>(1);
        let svc = TrackingService::new(Arc::new(tx));
        svc.record(dummy_record()); // fills the channel
        svc.record(dummy_record()); // should warn and drop, not panic
    }

    #[test]
    fn channel_capacity_constant() {
        assert_eq!(CHANNEL_CAPACITY, 1_000);
    }
}
