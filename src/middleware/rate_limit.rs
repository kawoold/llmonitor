use axum::{
    body::Body,
    http::{Request, StatusCode},
    response::{IntoResponse, Response},
};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tower::{Layer, Service};
use std::task::{Context, Poll};
use std::future::Future;
use std::pin::Pin;

#[derive(Debug)]
pub struct TokenBucket {
    capacity: f64,
    tokens: f64,
    refill_per_sec: f64,
    last_refill: Instant,
}

impl TokenBucket {
    pub fn new(capacity: u32, refill_per_sec: u32) -> Self {
        Self {
            capacity: capacity as f64,
            tokens: capacity as f64,
            refill_per_sec: refill_per_sec as f64,
            last_refill: Instant::now(),
        }
    }

    pub fn try_acquire(&mut self) -> bool {
        self.refill();
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_per_sec).min(self.capacity);
        self.last_refill = now;
    }

    pub fn retry_after_secs(&self) -> u64 {
        let deficit = 1.0 - self.tokens;
        (deficit / self.refill_per_sec).ceil() as u64
    }
}

#[derive(Clone)]
pub struct RateLimitLayer {
    bucket: Arc<Mutex<TokenBucket>>,
}

impl RateLimitLayer {
    pub fn new(capacity: u32, refill_per_sec: u32) -> Self {
        Self {
            bucket: Arc::new(Mutex::new(TokenBucket::new(capacity, refill_per_sec))),
        }
    }
}

impl<S> Layer<S> for RateLimitLayer {
    type Service = RateLimitMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RateLimitMiddleware {
            inner,
            bucket: self.bucket.clone(),
        }
    }
}

#[derive(Clone)]
pub struct RateLimitMiddleware<S> {
    inner: S,
    bucket: Arc<Mutex<TokenBucket>>,
}

impl<S> Service<Request<Body>> for RateLimitMiddleware<S>
where
    S: Service<Request<Body>, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let bucket = self.bucket.clone();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            let (allowed, retry_after) = {
                let mut b = bucket.lock().unwrap();
                let allowed = b.try_acquire();
                let retry_after = if !allowed {
                    b.retry_after_secs()
                } else {
                    0
                };
                (allowed, retry_after)
            };

            if !allowed {
                let mut resp = (
                    StatusCode::TOO_MANY_REQUESTS,
                    axum::Json(serde_json::json!({
                        "error": {
                            "message": "Rate limit exceeded",
                            "type": "rate_limit_error",
                            "code": "rate_limit_exceeded"
                        }
                    })),
                )
                    .into_response();
                resp.headers_mut().insert(
                    "Retry-After",
                    retry_after.to_string().parse().unwrap(),
                );
                return Ok(resp);
            }

            inner.call(req).await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_bucket_allows_request() {
        let mut bucket = TokenBucket::new(10, 1);
        assert!(bucket.try_acquire());
    }

    #[test]
    fn empty_bucket_denies_request() {
        let mut bucket = TokenBucket::new(2, 1);
        bucket.try_acquire();
        bucket.try_acquire();
        assert!(!bucket.try_acquire());
    }

    #[test]
    fn retry_after_is_positive_when_empty() {
        let mut bucket = TokenBucket::new(1, 1);
        bucket.try_acquire();
        assert!(bucket.retry_after_secs() >= 1);
    }
}
