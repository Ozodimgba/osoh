use serde::{Deserialize, Serialize};

/// Configuration for bincode serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BincodeConfig {
    /// Maximum size limit for deserializing collections
    pub limit: Option<u64>,
    /// Whether to use little endian (most common)
    pub little_endian: bool,
}

impl Default for BincodeConfig {
    fn default() -> Self {
        Self {
            limit: Some(10 * 1024 * 1024), // 10MB max
            little_endian: true,
        }
    }
}

impl BincodeConfig {
    pub fn high_performance() -> Self {
        Self {
            limit: Some(50 * 1024 * 1024), // Larger limit
            little_endian: true,
        }
    }

    pub fn compact() -> Self {
        Self {
            limit: Some(5 * 1024 * 1024), // Smaller limit
            little_endian: true,
        }
    }
}

impl BincodeConfig {
    pub fn to_bincode_config(&self) -> impl bincode::config::Config {
        bincode::config::standard().with_little_endian()
    }

    /// Check if data size exceeds our limit
    pub fn check_size_limit(&self, size: usize) -> Result<(), String> {
        if let Some(limit) = self.limit {
            if size as u64 > limit {
                return Err(format!("Data size {} exceeds limit {}", size, limit));
            }
        }
        Ok(())
    }
}

/// Global codec configuration
#[derive(Debug, Clone)]
pub struct CodecConfig {
    pub bincode: BincodeConfig,
    pub enable_compression: bool,
    pub compression_threshold_bytes: usize,
}

impl Default for CodecConfig {
    fn default() -> Self {
        Self {
            bincode: BincodeConfig::default(),
            enable_compression: false,
            compression_threshold_bytes: 1024,
        }
    }
}

impl CodecConfig {
    pub fn trading_optimized() -> Self {
        Self {
            bincode: BincodeConfig::high_performance(),
            enable_compression: false, // No compression for speed
            compression_threshold_bytes: usize::MAX,
        }
    }

    pub fn wan_optimized() -> Self {
        Self {
            bincode: BincodeConfig::compact(),
            enable_compression: true,
            compression_threshold_bytes: 256,
        }
    }
}
