use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct RequestContext {
    /// When the request was received.
    pub received_at: Instant,

    /// Client's IP address.
    pub client_ip: Option<String>,

    /// Request metadata.
    pub metadata: HashMap<String, String>,

    /// Tracing correlation ID.
    pub trace_id: String,

    pub rate_limit_remaining: Option<u32>,
}

impl RequestContext {
    pub fn new() -> Self {
        Self {
            received_at: Instant::now(),
            client_ip: None,
            metadata: HashMap::new(),
            trace_id: uuid::Uuid::new_v4().to_string(),
            rate_limit_remaining: None,
        }
    }

    pub fn with_client_ip(mut self, ip: String) -> Self {
        self.client_ip = Some(ip);
        self
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    pub fn elapsed(&self) -> std::time::Duration {
        self.received_at.elapsed()
    }
}

impl Default for RequestContext {
    fn default() -> Self {
        Self::new()
    }
}
