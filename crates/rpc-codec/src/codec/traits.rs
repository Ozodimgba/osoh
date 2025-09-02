use crate::error::RpcResult;
use serde::{Serialize, de::DeserializeOwned};
use std::fmt::Debug;

/// Core trait that all codecs must implement
pub trait RpcCodec: Send + Sync + Clone + Debug + 'static {
    /// Serialize a value to bytes
    fn encode<T>(&self, value: &T) -> RpcResult<Vec<u8>>
    where
        T: Serialize;

    /// Deserialize bytes to a value
    fn decode<T>(&self, bytes: &[u8]) -> RpcResult<T>
    where
        T: DeserializeOwned;

    /// Get the name of this codec (for logging/debugging)
    fn name(&self) -> &'static str;

    /// Get the MIME type for this codec
    fn mime_type(&self) -> &'static str;
}

/// Extended codec trait for zero-copy operations
pub trait ZeroCopyCodec: RpcCodec {
    /// Deserialize without copying data where possible
    fn decode_borrowed<'a, T>(&self, bytes: &'a [u8]) -> RpcResult<T>
    where
        T: serde::Deserialize<'a>;

    /// Check if a type supports zero-copy deserialization
    fn supports_zero_copy<T>(&self) -> bool
    where
        T: DeserializeOwned;
}

/// Codec with size estimation capabilities
pub trait SizingCodec: RpcCodec {
    /// Estimate serialized size without actually serializing
    fn estimate_size<T>(&self, value: &T) -> Option<usize>
    where
        T: Serialize;

    /// Get maximum possible size for a type
    fn max_size<T>(&self) -> Option<usize>
    where
        T: Serialize;
}

/// Codec with streaming capabilities
pub trait StreamingCodec: RpcCodec {
    /// Serialize directly to a writer
    fn encode_to_writer<T, W>(&self, value: &T, writer: W) -> RpcResult<()>
    where
        T: Serialize,
        W: std::io::Write;

    /// Deserialize directly from a reader
    fn decode_from_reader<T, R>(&self, reader: R) -> RpcResult<T>
    where
        T: DeserializeOwned,
        R: std::io::Read;
}

/// Helper trait for codec selection
pub trait CodecFactory {
    type Codec: RpcCodec;

    fn create_codec(&self) -> Self::Codec;
    fn codec_name(&self) -> &'static str;
}

/// Codec performance metrics
#[derive(Debug, Clone, Default)]
pub struct CodecMetrics {
    pub total_encodes: u64,
    pub total_decodes: u64,
    pub total_bytes_encoded: u64,
    pub total_bytes_decoded: u64,
    pub encode_time_nanos: u64,
    pub decode_time_nanos: u64,
    pub errors: u64,
}

impl CodecMetrics {
    pub fn avg_encode_time_nanos(&self) -> f64 {
        if self.total_encodes == 0 {
            0.0
        } else {
            self.encode_time_nanos as f64 / self.total_encodes as f64
        }
    }

    pub fn avg_decode_time_nanos(&self) -> f64 {
        if self.total_decodes == 0 {
            0.0
        } else {
            self.decode_time_nanos as f64 / self.total_decodes as f64
        }
    }

    pub fn throughput_mbps_encode(&self) -> f64 {
        if self.encode_time_nanos == 0 {
            0.0
        } else {
            (self.total_bytes_encoded as f64 * 1_000_000_000.0 * 8.0)
                / (self.encode_time_nanos as f64 * 1_000_000.0)
        }
    }

    pub fn throughput_mbps_decode(&self) -> f64 {
        if self.decode_time_nanos == 0 {
            0.0
        } else {
            (self.total_bytes_decoded as f64 * 1_000_000_000.0 * 8.0)
                / (self.decode_time_nanos as f64 * 1_000_000.0)
        }
    }
}

/// Codec wrapper that tracks performance metrics
#[derive(Debug, Clone)]
pub struct MetricsCodec<C: RpcCodec> {
    inner: C,
    metrics: std::sync::Arc<std::sync::Mutex<CodecMetrics>>,
}

impl<C: RpcCodec> MetricsCodec<C> {
    pub fn new(codec: C) -> Self {
        Self {
            inner: codec,
            metrics: std::sync::Arc::new(std::sync::Mutex::new(CodecMetrics::default())),
        }
    }

    pub fn get_metrics(&self) -> CodecMetrics {
        self.metrics.lock().unwrap().clone()
    }

    pub fn reset_metrics(&self) {
        *self.metrics.lock().unwrap() = CodecMetrics::default();
    }
}

impl<C: RpcCodec> RpcCodec for MetricsCodec<C> {
    fn encode<T>(&self, value: &T) -> RpcResult<Vec<u8>>
    where
        T: Serialize,
    {
        let start = std::time::Instant::now();
        let result = self.inner.encode(value);
        let elapsed = start.elapsed().as_nanos() as u64;

        let mut metrics = self.metrics.lock().unwrap();
        metrics.total_encodes += 1;
        metrics.encode_time_nanos += elapsed;

        match &result {
            Ok(bytes) => {
                metrics.total_bytes_encoded += bytes.len() as u64;
            }
            Err(_) => {
                metrics.errors += 1;
            }
        }

        result
    }

    fn decode<T>(&self, bytes: &[u8]) -> RpcResult<T>
    where
        T: DeserializeOwned,
    {
        let start = std::time::Instant::now();
        let result = self.inner.decode(bytes);
        let elapsed = start.elapsed().as_nanos() as u64;

        let mut metrics = self.metrics.lock().unwrap();
        metrics.total_decodes += 1;
        metrics.decode_time_nanos += elapsed;
        metrics.total_bytes_decoded += bytes.len() as u64;

        if result.is_err() {
            metrics.errors += 1;
        }

        result
    }

    fn name(&self) -> &'static str {
        self.inner.name()
    }

    fn mime_type(&self) -> &'static str {
        self.inner.mime_type()
    }
}
