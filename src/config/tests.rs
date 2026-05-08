use crate::config::service::mask_api_key;
use crate::config::domain::{ConfigError, UpdateSettingsRequest};

#[test]
fn mask_empty_key() {
    assert_eq!(mask_api_key(""), "");
}

#[test]
fn mask_short_key() {
    assert_eq!(mask_api_key("short"), "***...***");
    assert_eq!(mask_api_key("exactly12ch!"), "***...***");
}

#[test]
fn mask_normal_key() {
    let key = "sk-ant-api03-abcdefghijklmn1234";
    let masked = mask_api_key(key);
    assert!(masked.starts_with("sk-ant-a"));
    assert!(masked.ends_with("1234"));
    assert!(masked.contains("***...***"));
    assert!(!masked.contains("abcdefghijklmn"));
}

#[tokio::test]
async fn admin_verify_wrong_password() {
    use crate::config::admin::AdminService;

    let pool = crate::test_helpers::test_db().await;
    let svc = AdminService::new(pool.clone());
    svc.create("admin", "correct-password").await.unwrap();

    let result = svc.verify("admin", "wrong-password").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn admin_verify_unknown_user() {
    use crate::config::admin::AdminService;

    let pool = crate::test_helpers::test_db().await;
    let svc = AdminService::new(pool);
    let result = svc.verify("nonexistent", "password").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn admin_verify_correct_password() {
    use crate::config::admin::AdminService;

    let pool = crate::test_helpers::test_db().await;
    let svc = AdminService::new(pool);
    svc.create("admin", "correct-password").await.unwrap();
    assert!(svc.verify("admin", "correct-password").await.is_ok());
}

#[tokio::test]
async fn config_update_rejects_empty_api_key() {
    use crate::config::service::ConfigService;

    let pool = crate::test_helpers::test_db().await;
    let svc = ConfigService::new(pool);
    let req = UpdateSettingsRequest {
        anthropic_api_key: Some(String::new()),
        request_timeout_secs: None,
        rate_limit_capacity: None,
        rate_limit_refill_per_sec: None,
    };
    let result = svc.update_settings(req).await;
    assert!(matches!(result, Err(ConfigError::EmptyApiKey)));
}

#[tokio::test]
async fn config_get_all_returns_defaults_when_empty() {
    use crate::config::service::ConfigService;

    let pool = crate::test_helpers::test_db().await;
    let svc = ConfigService::new(pool);
    let snapshot = svc.get_all().await.unwrap();
    assert!(snapshot.anthropic_api_key.is_empty());
    assert_eq!(snapshot.request_timeout_secs, 30);
    assert_eq!(snapshot.rate_limit_capacity, 100);
}
