use prometheus::{Encoder, IntCounterVec, Opts, Registry, TextEncoder};

pub struct MetricsRegistry {
    registry: Registry,
    requests_total: IntCounterVec,
    tokens_total: IntCounterVec,
    cache_tokens_total: IntCounterVec,
}

impl MetricsRegistry {
    pub fn new() -> Self {
        let registry = Registry::new();

        let requests_total = IntCounterVec::new(
            Opts::new("llmonitor_requests_total", "Total proxy requests by provider, model, and status"),
            &["provider", "model", "status"],
        )
        .expect("metric definition is valid");

        let tokens_total = IntCounterVec::new(
            Opts::new("llmonitor_tokens_total", "Total tokens processed by provider, model, and kind"),
            &["provider", "model", "kind"],
        )
        .expect("metric definition is valid");

        let cache_tokens_total = IntCounterVec::new(
            Opts::new("llmonitor_cache_tokens_total", "Anthropic prompt cache tokens by kind"),
            &["kind"],
        )
        .expect("metric definition is valid");

        registry.register(Box::new(requests_total.clone())).expect("register requests_total");
        registry.register(Box::new(tokens_total.clone())).expect("register tokens_total");
        registry.register(Box::new(cache_tokens_total.clone())).expect("register cache_tokens_total");

        Self { registry, requests_total, tokens_total, cache_tokens_total }
    }

    pub fn record_request(&self, provider: &str, model: &str, status: u16) {
        self.requests_total
            .with_label_values(&[provider, model, &status.to_string()])
            .inc();
    }

    pub fn record_tokens(&self, provider: &str, model: &str, prompt: u32, completion: u32) {
        self.tokens_total
            .with_label_values(&[provider, model, "prompt"])
            .inc_by(prompt as u64);
        self.tokens_total
            .with_label_values(&[provider, model, "completion"])
            .inc_by(completion as u64);
    }

    pub fn record_cache(&self, cache_read: u32, cache_creation: u32) {
        self.cache_tokens_total
            .with_label_values(&["read"])
            .inc_by(cache_read as u64);
        self.cache_tokens_total
            .with_label_values(&["creation"])
            .inc_by(cache_creation as u64);
    }

    pub fn render(&self) -> String {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer).expect("prometheus encoding failed");
        String::from_utf8(buffer).expect("prometheus output is valid utf8")
    }
}

impl Default for MetricsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_instances_do_not_panic() {
        let _a = MetricsRegistry::new();
        let _b = MetricsRegistry::new();
    }

    #[test]
    fn render_returns_prometheus_format() {
        let m = MetricsRegistry::new();
        m.record_request("claude", "claude-sonnet-4-6", 200);
        let output = m.render();
        assert!(output.contains("# HELP llmonitor_requests_total"));
        assert!(output.contains("# TYPE llmonitor_requests_total counter"));
    }

    #[test]
    fn record_request_increments_counter() {
        let m = MetricsRegistry::new();
        m.record_request("claude", "claude-sonnet-4-6", 200);
        let output = m.render();
        assert!(output.contains(r#"provider="claude""#));
    }

    #[test]
    fn record_tokens_increments_both_kinds() {
        let m = MetricsRegistry::new();
        m.record_tokens("claude", "claude-sonnet-4-6", 100, 50);
        let output = m.render();
        assert!(output.contains(r#"kind="prompt""#));
        assert!(output.contains(r#"kind="completion""#));
    }

    #[test]
    fn record_cache_increments_both_kinds() {
        let m = MetricsRegistry::new();
        m.record_cache(200, 50);
        let output = m.render();
        assert!(output.contains(r#"kind="read""#));
        assert!(output.contains(r#"kind="creation""#));
    }
}
