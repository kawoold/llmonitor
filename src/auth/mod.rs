use axum::{
    body::Body,
    extract::State,
    http::{header, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Json, Response},
};
use base64::{engine::general_purpose::STANDARD, Engine};
use tracing::warn;

use crate::app_state::AppState;

pub async fn basic_auth_middleware(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());

    let credentials = auth_header.and_then(|h| h.strip_prefix("Basic ")).and_then(|encoded| {
        STANDARD.decode(encoded).ok().and_then(|bytes| {
            String::from_utf8(bytes).ok().and_then(|s| {
                let mut parts = s.splitn(2, ':');
                let username = parts.next()?.to_string();
                let password = parts.next()?.to_string();
                Some((username, password))
            })
        })
    });

    let (username, password) = match credentials {
        Some(c) => c,
        None => return unauthorized_response(),
    };

    match state.admin_service.verify(&username, &password).await {
        Ok(()) => next.run(request).await,
        Err(e) => {
            warn!(username, error = %e, "Authentication failed");
            unauthorized_response()
        }
    }
}

fn unauthorized_response() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        [(
            header::WWW_AUTHENTICATE,
            "Basic realm=\"llmonitor\"",
        )],
        Json(serde_json::json!({
            "error": {
                "message": "Invalid credentials",
                "type": "authentication_error",
                "code": "unauthorized"
            }
        })),
    )
        .into_response()
}
