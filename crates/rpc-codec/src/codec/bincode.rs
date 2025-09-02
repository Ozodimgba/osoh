use crate::codec::{
    config::{BincodeConfig, CodecConfig},
    traits::*,
};
use crate::error::{RpcError, RpcErrorCode, RpcResult};
use serde::{Serialize, de::DeserializeOwned};

#[derive(Clone)]
pub struct BincodeCodec {
    config: BincodeConfig,
    // bincode_config: bincode::config::Configuration,
    enable_compression: bool,     // compress large messages?
    compression_threshold: usize, // how large before compression
}

impl std::fmt::Debug for BincodeCodec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BincodeCodec")
            .field("config", &self.config)
            .field("bincode_config", &"<bincode::Configuration>") // Skip the problematic field
            .field("enable_compression", &self.enable_compression)
            .field("compression_threshold", &self.compression_threshold)
            .finish()
    }
}

impl BincodeCodec {
    pub fn new() -> Self {
        let config = BincodeConfig::default();

        Self {
            config,
            enable_compression: false,
            compression_threshold: 1024,
        }
    }

    pub fn with_config(codec_config: CodecConfig) -> Self {
        Self {
            config: codec_config.bincode,
            enable_compression: codec_config.enable_compression,
            compression_threshold: codec_config.compression_threshold_bytes,
        }
    }

    pub fn high_performance() -> Self {
        let mut config = BincodeConfig::default();
        config.little_endian = true; // Use your field name
        config.limit = Some(100 * 1024 * 1024); // Fix the math: 100MB

        Self {
            config,
            enable_compression: false,
            compression_threshold: usize::MAX,
        }
    }

    pub fn compact() -> Self {
        let mut config = BincodeConfig::default();
        config.little_endian = true; // Use your field name
        config.limit = Some(5 * 1024 * 1024); // Fix the math: 5MB

        Self {
            config,
            enable_compression: true,
            compression_threshold: 256, // Fix: you had usize::MAX but want compression
        }
    }

    // Add helper method to get bincode config when needed
    fn get_bincode_config(&self) -> impl bincode::config::Config {
        self.config.to_bincode_config()
    }
}

impl RpcCodec for BincodeCodec {
    fn encode<T>(&self, value: &T) -> RpcResult<Vec<u8>>
    where
        T: Serialize,
    {
        let bincode_config = self.config.to_bincode_config();

        let serialized_bytes =
            bincode::serde::encode_to_vec(value, bincode_config).map_err(|e| {
                RpcError::new(
                    RpcErrorCode::InternalError,
                    format!("Failed to serialize with bincode: {}", e),
                )
            })?;

        self.config
            .check_size_limit(serialized_bytes.len())
            .map_err(|e| RpcError::new(RpcErrorCode::InvalidParams, e))?;

        Ok(serialized_bytes)
    }

    fn decode<T: DeserializeOwned>(&self, bytes: &[u8]) -> RpcResult<T>
    where
        T: DeserializeOwned,
    {
        // Check size limit before decoding
        self.config
            .check_size_limit(bytes.len())
            .map_err(|e| RpcError::new(RpcErrorCode::InvalidParams, e))?;

        // eprintln!(
        //     "Deserializing {} bytes: {:?}",
        //     bytes.len(),
        //     &bytes[..std::cmp::min(20, bytes.len())]
        // );

        // Try to decode as UTF-8 to check if it's an error message
        if let Ok(text) = std::str::from_utf8(bytes) {
            if text.contains("Failed to deserialize") {
                eprintln!(
                    "⚠️ WARNING: Trying to deserialize an error message: {}",
                    text
                );
                return Err(RpcError::new(
                    RpcErrorCode::ParseError,
                    format!("Received error message instead of data: {}", text),
                ));
            }
        }

        let bincode_config = self.config.to_bincode_config();

        let (deserialized_value, _bytes_read) =
            bincode::serde::decode_from_slice(bytes, bincode_config).map_err(|e| {
                RpcError::new(
                    RpcErrorCode::ParseError,
                    format!("Failed to deserialize with bincode: {}", e),
                )
            })?;

        Ok(deserialized_value)
    }

    fn name(&self) -> &'static str {
        "bincode"
    }

    fn mime_type(&self) -> &'static str {
        "application/octet-stream" // Standard MIME type for binary data
    }
}
