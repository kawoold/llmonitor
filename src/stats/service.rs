use sqlx::{Row, SqlitePool};

use super::types::{
    CacheStats, ProviderSummary, SessionStats, StatsError, StatsSummary, TimeWindow, UsageDataPoint,
};

#[derive(Clone)]
pub struct StatsService {
    db: SqlitePool,
}

impl StatsService {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }

    pub async fn summary(&self, window: TimeWindow) -> Result<StatsSummary, StatsError> {
        let cutoff = window_cutoff(window);
        let sql = format!(
            "SELECT provider, model, \
             COUNT(*) as request_count, \
             COALESCE(SUM(prompt_tokens), 0) as prompt_tokens, \
             COALESCE(SUM(completion_tokens), 0) as completion_tokens, \
             COALESCE(SUM(total_tokens), 0) as total_tokens \
             FROM request_logs WHERE created_at >= {cutoff} \
             GROUP BY provider, model ORDER BY total_tokens DESC"
        );
        let rows = sqlx::query(&sql).fetch_all(&self.db).await?;

        let by_provider: Vec<ProviderSummary> = rows
            .iter()
            .map(|r| ProviderSummary {
                provider: r.get("provider"),
                model: r.get("model"),
                request_count: r.get("request_count"),
                prompt_tokens: r.get("prompt_tokens"),
                completion_tokens: r.get("completion_tokens"),
                total_tokens: r.get("total_tokens"),
            })
            .collect();

        let total_requests = by_provider.iter().map(|p| p.request_count).sum();
        let prompt_tokens = by_provider.iter().map(|p| p.prompt_tokens).sum();
        let completion_tokens = by_provider.iter().map(|p| p.completion_tokens).sum();
        let total_tokens = by_provider.iter().map(|p| p.total_tokens).sum();

        Ok(StatsSummary {
            window: window.label().to_string(),
            total_requests,
            total_tokens,
            prompt_tokens,
            completion_tokens,
            by_provider,
        })
    }

    pub async fn usage_timeseries(
        &self,
        window: TimeWindow,
    ) -> Result<Vec<UsageDataPoint>, StatsError> {
        let cutoff = window_cutoff(window);
        let bucket = bucket_expr(window);
        let sql = format!(
            "SELECT {bucket} as bucket, \
             COUNT(*) as request_count, \
             COALESCE(SUM(prompt_tokens), 0) as prompt_tokens, \
             COALESCE(SUM(completion_tokens), 0) as completion_tokens, \
             COALESCE(SUM(total_tokens), 0) as total_tokens \
             FROM request_logs WHERE created_at >= {cutoff} \
             GROUP BY bucket ORDER BY bucket ASC"
        );
        let rows = sqlx::query(&sql).fetch_all(&self.db).await?;

        Ok(rows
            .iter()
            .map(|r| UsageDataPoint {
                bucket: r.get("bucket"),
                request_count: r.get("request_count"),
                prompt_tokens: r.get("prompt_tokens"),
                completion_tokens: r.get("completion_tokens"),
                total_tokens: r.get("total_tokens"),
            })
            .collect())
    }

    pub async fn session_stats(
        &self,
        window: TimeWindow,
    ) -> Result<Vec<SessionStats>, StatsError> {
        let cutoff = window_cutoff(window);
        let sql = format!(
            "SELECT session_id, \
             COUNT(*) as request_count, \
             COALESCE(SUM(total_tokens), 0) as total_tokens, \
             MIN(created_at) as first_seen, \
             MAX(created_at) as last_seen \
             FROM request_logs WHERE created_at >= {cutoff} \
             GROUP BY session_id ORDER BY total_tokens DESC LIMIT 100"
        );
        let rows = sqlx::query(&sql).fetch_all(&self.db).await?;

        Ok(rows
            .iter()
            .map(|r| SessionStats {
                session_id: r.get("session_id"),
                request_count: r.get("request_count"),
                total_tokens: r.get("total_tokens"),
                first_seen: r.get("first_seen"),
                last_seen: r.get("last_seen"),
            })
            .collect())
    }

    pub async fn cache_stats(&self, window: TimeWindow) -> Result<CacheStats, StatsError> {
        let cutoff = window_cutoff(window);
        let sql = format!(
            "SELECT \
             COALESCE(SUM(cache_read_tokens), 0) as cache_read_tokens, \
             COALESCE(SUM(cache_creation_tokens), 0) as cache_creation_tokens, \
             COALESCE(SUM(CASE WHEN cache_read_tokens > 0 OR cache_creation_tokens > 0 THEN 1 ELSE 0 END), 0) as total_requests \
             FROM request_logs WHERE created_at >= {cutoff}"
        );
        let row = sqlx::query(&sql).fetch_one(&self.db).await?;

        let cache_read_tokens: i64 = row.get("cache_read_tokens");
        let cache_creation_tokens: i64 = row.get("cache_creation_tokens");
        let total_requests: i64 = row.get("total_requests");
        let total = cache_read_tokens + cache_creation_tokens;
        let cache_hit_rate = if total == 0 {
            0.0
        } else {
            cache_read_tokens as f64 / total as f64
        };

        Ok(CacheStats {
            window: window.label().to_string(),
            cache_read_tokens,
            cache_creation_tokens,
            total_requests,
            cache_hit_rate,
        })
    }
}

fn window_cutoff(window: TimeWindow) -> &'static str {
    match window {
        TimeWindow::OneHour => "strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '-1 hours')",
        TimeWindow::OneDay => "strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '-24 hours')",
        TimeWindow::SevenDays => "strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '-7 days')",
        TimeWindow::ThirtyDays => "strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '-30 days')",
    }
}

fn bucket_expr(window: TimeWindow) -> &'static str {
    match window {
        TimeWindow::OneHour => "strftime('%Y-%m-%dT%H:%M:00Z', created_at)",
        TimeWindow::OneDay => "strftime('%Y-%m-%dT%H:00:00Z', created_at)",
        TimeWindow::SevenDays => "strftime('%Y-%m-%d', created_at)",
        TimeWindow::ThirtyDays => "strftime('%Y-%m-%d', created_at)",
    }
}

#[cfg(test)]
mod tests {
    use chrono::{SecondsFormat, Utc};

    use super::*;

    async fn test_db() -> SqlitePool {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        pool
    }

    async fn insert_record(
        db: &SqlitePool,
        id: &str,
        provider: &str,
        model: &str,
        prompt: i64,
        completion: i64,
        cache_read: i64,
        cache_creation: i64,
        session_id: &str,
    ) {
        let created_at = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
        sqlx::query(
            "INSERT INTO request_logs \
             (id, created_at, provider, model, session_id, \
              prompt_tokens, completion_tokens, total_tokens, \
              cache_read_tokens, cache_creation_tokens) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(id)
        .bind(&created_at)
        .bind(provider)
        .bind(model)
        .bind(session_id)
        .bind(prompt)
        .bind(completion)
        .bind(prompt + completion)
        .bind(cache_read)
        .bind(cache_creation)
        .execute(db)
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn summary_empty_db_returns_zero_totals() {
        let db = test_db().await;
        let svc = StatsService::new(db);
        let result = svc.summary(TimeWindow::OneDay).await.unwrap();
        assert_eq!(result.total_requests, 0);
        assert_eq!(result.total_tokens, 0);
        assert!(result.by_provider.is_empty());
    }

    #[tokio::test]
    async fn summary_aggregates_correctly() {
        let db = test_db().await;
        insert_record(&db, "r1", "claude", "claude-sonnet-4-6", 10, 5, 0, 0, "s1").await;
        insert_record(&db, "r2", "claude", "claude-sonnet-4-6", 20, 8, 0, 0, "s2").await;
        let svc = StatsService::new(db);
        let result = svc.summary(TimeWindow::OneDay).await.unwrap();
        assert_eq!(result.total_requests, 2);
        assert_eq!(result.prompt_tokens, 30);
        assert_eq!(result.completion_tokens, 13);
        assert_eq!(result.by_provider.len(), 1);
        assert_eq!(result.by_provider[0].request_count, 2);
    }

    #[tokio::test]
    async fn usage_timeseries_empty_db() {
        let db = test_db().await;
        let svc = StatsService::new(db);
        let result = svc.usage_timeseries(TimeWindow::OneHour).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn session_stats_groups_by_session_id() {
        let db = test_db().await;
        insert_record(&db, "r1", "claude", "claude-sonnet-4-6", 10, 5, 0, 0, "sess-a").await;
        insert_record(&db, "r2", "claude", "claude-sonnet-4-6", 20, 8, 0, 0, "sess-a").await;
        insert_record(&db, "r3", "claude", "claude-sonnet-4-6", 5, 3, 0, 0, "sess-b").await;
        let svc = StatsService::new(db);
        let result = svc.session_stats(TimeWindow::OneDay).await.unwrap();
        assert_eq!(result.len(), 2);
        let sess_a = result.iter().find(|s| s.session_id == "sess-a").unwrap();
        assert_eq!(sess_a.request_count, 2);
        assert_eq!(sess_a.total_tokens, 43);
    }

    #[tokio::test]
    async fn cache_stats_computes_hit_rate() {
        let db = test_db().await;
        insert_record(&db, "r1", "claude", "claude-sonnet-4-6", 10, 5, 100, 0, "s1").await;
        insert_record(&db, "r2", "claude", "claude-sonnet-4-6", 10, 5, 0, 200, "s2").await;
        let svc = StatsService::new(db);
        let result = svc.cache_stats(TimeWindow::OneDay).await.unwrap();
        assert_eq!(result.cache_read_tokens, 100);
        assert_eq!(result.cache_creation_tokens, 200);
        assert_eq!(result.total_requests, 2);
        assert!((result.cache_hit_rate - 1.0 / 3.0).abs() < 0.001);
    }

    #[tokio::test]
    async fn cache_stats_zero_safe_on_empty_db() {
        let db = test_db().await;
        let svc = StatsService::new(db);
        let result = svc.cache_stats(TimeWindow::OneDay).await.unwrap();
        assert_eq!(result.cache_hit_rate, 0.0);
        assert_eq!(result.total_requests, 0);
    }
}
