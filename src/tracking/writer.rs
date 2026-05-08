use std::time::Duration;

use sqlx::SqlitePool;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use super::types::{UsageRecord, BATCH_SIZE, FLUSH_INTERVAL};

pub struct TrackingWriterService {
    receiver: mpsc::Receiver<UsageRecord>,
    db: SqlitePool,
}

impl TrackingWriterService {
    pub fn new(receiver: mpsc::Receiver<UsageRecord>, db: SqlitePool) -> Self {
        Self { receiver, db }
    }

    pub fn spawn(mut self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            info!("tracking writer started");
            let mut buffer: Vec<UsageRecord> = Vec::with_capacity(BATCH_SIZE);
            let mut ticker = tokio::time::interval(FLUSH_INTERVAL);
            // Skip the immediate first tick
            ticker.tick().await;

            loop {
                tokio::select! {
                    record = self.receiver.recv() => {
                        match record {
                            Some(r) => buffer.push(r),
                            None => {
                                // Channel closed — final drain
                                if !buffer.is_empty() {
                                    flush(&buffer, &self.db).await;
                                }
                                info!(
                                    remaining = buffer.len(),
                                    "tracking writer draining and stopping"
                                );
                                break;
                            }
                        }
                    }
                    _ = ticker.tick() => {
                        if !buffer.is_empty() {
                            debug!(count = buffer.len(), "flushing tracking records");
                            // Process at most BATCH_SIZE records per tick
                            let drain_end = buffer.len().min(BATCH_SIZE);
                            let batch: Vec<UsageRecord> = buffer.drain(..drain_end).collect();
                            flush(&batch, &self.db).await;
                        }
                    }
                }
            }

            info!("tracking writer stopped");
        })
    }
}

async fn flush(records: &[UsageRecord], db: &SqlitePool) {
    for record in records {
        let result = insert_record(record, db).await;
        if let Err(e) = result {
            warn!(id = %record.id, error = %e, "tracking insert failed, retrying in 50ms");
            tokio::time::sleep(Duration::from_millis(50)).await;
            if let Err(e2) = insert_record(record, db).await {
                error!(id = %record.id, error = %e2, "tracking insert failed after retry, dropping record");
            }
        }
    }
}

async fn insert_record(record: &UsageRecord, db: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT OR IGNORE INTO request_logs \
         (id, created_at, provider, model, session_id, \
          prompt_tokens, completion_tokens, total_tokens, \
          cache_read_tokens, cache_creation_tokens) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&record.id)
    .bind(&record.created_at)
    .bind(&record.provider)
    .bind(&record.model)
    .bind(&record.session_id)
    .bind(record.prompt_tokens as i64)
    .bind(record.completion_tokens as i64)
    .bind(record.total_tokens as i64)
    .bind(record.cache_read_tokens as i64)
    .bind(record.cache_creation_tokens as i64)
    .execute(db)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tracking::types::CHANNEL_CAPACITY;
    use tokio::sync::mpsc;

    async fn test_db() -> SqlitePool {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        pool
    }

    fn make_record(id: &str) -> UsageRecord {
        UsageRecord {
            id: id.to_string(),
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
    async fn writer_flushes_record_to_db() {
        let db = test_db().await;
        let (tx, rx) = mpsc::channel::<UsageRecord>(CHANNEL_CAPACITY);
        let writer = TrackingWriterService::new(rx, db.clone());
        let handle = writer.spawn();

        tx.send(make_record("record-1")).await.unwrap();

        // Wait slightly longer than flush interval
        tokio::time::sleep(Duration::from_millis(700)).await;

        // Drop sender to trigger graceful drain
        drop(tx);
        handle.await.unwrap();

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM request_logs")
            .fetch_one(&db)
            .await
            .unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn idempotent_insert_ignores_duplicate() {
        let db = test_db().await;
        let record = make_record("dup-id");
        insert_record(&record, &db).await.unwrap();
        // Second insert should not error (INSERT OR IGNORE)
        insert_record(&record, &db).await.unwrap();

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM request_logs")
            .fetch_one(&db)
            .await
            .unwrap();
        assert_eq!(count, 1);
    }
}
