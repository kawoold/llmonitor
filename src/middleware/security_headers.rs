use axum::{
    body::Body,
    http::Request,
    middleware::Next,
    response::Response,
};

pub async fn security_headers_middleware(request: Request<Body>, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert("x-content-type-options", "nosniff".parse().unwrap());
    headers.insert("x-frame-options", "DENY".parse().unwrap());
    headers.insert("referrer-policy", "strict-origin-when-cross-origin".parse().unwrap());
    headers.insert("cache-control", "no-store".parse().unwrap());
    headers.insert(
        "content-security-policy",
        "default-src 'self'; script-src 'self' https://cdn.tailwindcss.com; style-src 'self' 'unsafe-inline'"
            .parse()
            .unwrap(),
    );
    response
}
