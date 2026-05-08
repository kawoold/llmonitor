use std::sync::Arc;
use std::time::Instant;

use sqlx::SqlitePool;

use crate::config::Config;
use crate::config::admin::AdminService;
use crate::config::service::ConfigService;
use crate::metrics::MetricsRegistry;
use crate::stats::StatsService;
use crate::tracking::TrackingService;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub tracking_service: TrackingService,
    pub metrics: Arc<MetricsRegistry>,
    pub config: Arc<Config>,
    pub config_service: ConfigService,
    pub admin_service: AdminService,
    pub stats: StatsService,
    pub started_at: Instant,
}

impl AppState {
    pub fn new(db: SqlitePool, config: Config, tracking_service: TrackingService) -> Self {
        Self {
            config_service: ConfigService::new(db.clone()),
            admin_service: AdminService::new(db.clone()),
            stats: StatsService::new(db.clone()),
            metrics: Arc::new(MetricsRegistry::new()),
            config: Arc::new(config),
            tracking_service,
            started_at: Instant::now(),
            db,
        }
    }
}
