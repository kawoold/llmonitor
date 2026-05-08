use sqlx::SqlitePool;
use tracing::{error, info};

use crate::config::admin::AdminService;
use crate::config::domain::BootstrapError;

pub async fn bootstrap(db: &SqlitePool) -> Result<(), BootstrapError> {
    let admin_service = AdminService::new(db.clone());

    if admin_service.any_exists().await? {
        info!("Admin users found, skipping bootstrap");
        return Ok(());
    }

    let username = std::env::var("ADMIN_USERNAME").ok();
    let password = std::env::var("ADMIN_PASSWORD").ok();

    match (username, password) {
        (Some(u), Some(p)) => {
            admin_service.create(&u, &p).await?;
            info!("Admin user created from environment variables");
            Ok(())
        }
        _ => {
            error!(
                "No admin users found and ADMIN_USERNAME/ADMIN_PASSWORD are not set. \
                 Set these environment variables to create the initial admin user."
            );
            Err(BootstrapError::MissingEnvVars)
        }
    }
}
