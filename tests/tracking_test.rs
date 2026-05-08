use std::sync::Arc;
use std::time::Duration;

use llmonitor::tracking::{TrackingService, TrackingWriterService, UsageRecord, CHANNEL_CAPACITY};

async fn test_db() -> sqlx::SqlitePool {
    llmonitor::test_helpers::test_db().await
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
async fn tracking_service_record_persists_to_db() {
    let db = test_db().await;
    let (tx, rx) = tokio::sync::mpsc::channel::<UsageRecord>(CHANNEL_CAPACITY);
    let writer = TrackingWriterService::new(rx, db.clone());
    let handle = writer.spawn();

    let service = TrackingService::new(Arc::new(tx));
    service.record(make_record("integration-1"));

    // Wait longer than the 500ms flush interval
    tokio::time::sleep(Duration::from_millis(700)).await;

    drop(service);
    handle.await.unwrap();

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM request_logs")
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(count, 1);
}

#[tokio::test]
async fn tracking_service_multiple_records_all_persisted() {
    let db = test_db().await;
    let (tx, rx) = tokio::sync::mpsc::channel::<UsageRecord>(CHANNEL_CAPACITY);
    let writer = TrackingWriterService::new(rx, db.clone());
    let handle = writer.spawn();

    let service = TrackingService::new(Arc::new(tx));
    for i in 0..5 {
        service.record(make_record(&format!("multi-{i}")));
    }

    tokio::time::sleep(Duration::from_millis(700)).await;

    drop(service);
    handle.await.unwrap();

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM request_logs")
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(count, 5);
}
