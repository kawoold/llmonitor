use axum::http::{HeaderMap, HeaderValue};
use axum::response::IntoResponse;

use crate::config::types::ApiKey;
use crate::proxy::error::ProxyError;
use crate::proxy::routing::{extract_provider_name, validate_provider_name};
use crate::providers::Provider;

fn make_headers(provider: Option<&str>) -> HeaderMap {
    let mut headers = HeaderMap::new();
    if let Some(p) = provider {
        headers.insert("x-provider", HeaderValue::from_str(p).unwrap());
    }
    headers
}

#[test]
fn extract_provider_missing_header() {
    let headers = make_headers(None);
    assert!(matches!(
        extract_provider_name(&headers),
        Err(ProxyError::ProviderHeaderMissing)
    ));
}

#[test]
fn extract_provider_present() {
    let headers = make_headers(Some("claude"));
    assert_eq!(extract_provider_name(&headers).unwrap(), "claude");
}

#[test]
fn validate_provider_unknown() {
    assert!(matches!(
        validate_provider_name("openai"),
        Err(ProxyError::ProviderUnknown(_))
    ));
}

#[test]
fn validate_provider_known() {
    assert!(validate_provider_name("claude").is_ok());
    assert!(validate_provider_name("CLAUDE").is_ok());
    assert!(validate_provider_name("anthropic").is_ok());
}

#[test]
fn provider_from_name_case_insensitive() {
    let key = ApiKey::new("test-key");
    assert!(Provider::from_name("claude", key.clone()).is_ok());
    assert!(Provider::from_name("Claude", key.clone()).is_ok());
    assert!(Provider::from_name("CLAUDE", key.clone()).is_ok());
    assert!(Provider::from_name("anthropic", key).is_ok());
}

#[test]
fn provider_from_name_unknown() {
    let key = ApiKey::new("test-key");
    let result = Provider::from_name("openai", key);
    assert!(matches!(result, Err(ProxyError::ProviderUnknown(_))));
}

#[test]
fn provider_from_name_not_configured() {
    let key = ApiKey::new("");
    let result = Provider::from_name("claude", key);
    assert!(matches!(result, Err(ProxyError::ProviderNotConfigured(_))));
}

#[test]
fn provider_accepted_models_non_empty() {
    let key = ApiKey::new("test-key");
    let provider = Provider::from_name("claude", key).unwrap();
    assert!(!provider.accepted_models().is_empty());
}

#[test]
fn provider_accepts_known_model() {
    let key = ApiKey::new("test-key");
    let provider = Provider::from_name("claude", key).unwrap();
    assert!(provider.accepts_model("claude-sonnet-4-6"));
    assert!(!provider.accepts_model("gpt-4"));
}

#[test]
fn proxy_error_provider_header_missing_status() {
    let response = ProxyError::ProviderHeaderMissing.into_response();
    assert_eq!(response.status(), 400);
}

#[test]
fn proxy_error_provider_unknown_status() {
    let response = ProxyError::ProviderUnknown("bogus".to_string()).into_response();
    assert_eq!(response.status(), 400);
}

#[test]
fn proxy_error_provider_not_configured_status() {
    let response = ProxyError::ProviderNotConfigured("claude".to_string()).into_response();
    assert_eq!(response.status(), 503);
}

#[test]
fn proxy_error_rate_limit_status() {
    let response = ProxyError::RateLimitExceeded.into_response();
    assert_eq!(response.status(), 429);
}

#[test]
fn proxy_error_internal_error_status() {
    let response = ProxyError::InternalError("boom".to_string()).into_response();
    assert_eq!(response.status(), 500);
}

#[test]
fn proxy_error_model_not_accepted_status() {
    let response = ProxyError::ModelNotAccepted {
        model: "gpt-4".to_string(),
        provider: "claude".to_string(),
        accepted: "claude-sonnet-4-6".to_string(),
    }
    .into_response();
    assert_eq!(response.status(), 400);
}
