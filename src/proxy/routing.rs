use axum::http::HeaderMap;

use crate::proxy::error::ProxyError;
use crate::providers::Provider;

pub fn extract_provider_name(headers: &HeaderMap) -> Result<String, ProxyError> {
    headers
        .get("x-provider")
        .ok_or(ProxyError::ProviderHeaderMissing)?
        .to_str()
        .map(|s| s.to_lowercase())
        .map_err(|_| ProxyError::ProviderHeaderMissing)
}

pub fn validate_provider_name(name: &str) -> Result<(), ProxyError> {
    if !Provider::is_known_name(name) {
        return Err(ProxyError::ProviderUnknown(name.to_string()));
    }
    Ok(())
}
