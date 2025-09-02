use crate::codec::traits::*;
use crate::error::{RpcError, RpcErrorCode, RpcResult};
use serde::{Serialize, de::DeserializeOwned};

/// JSON codec for debugging and development
#[derive(Debug, Clone)]
pub struct JsonCodec {
    pretty: bool,
}

impl JsonCodec {
    pub fn new() -> Self {
        Self { pretty: false }
    }

    pub fn pretty() -> Self {
        Self { pretty: true }
    }
}

impl Default for JsonCodec {
    fn default() -> Self {
        Self::new()
    }
}

impl RpcCodec for JsonCodec {
    fn encode<T>(&self, value: &T) -> RpcResult<Vec<u8>>
    where
        T: Serialize,
    {
        let json_str = if self.pretty {
            serde_json::to_string_pretty(value)
        } else {
            serde_json::to_string(value)
        }
        .map_err(|e| {
            RpcError::new(
                RpcErrorCode::InternalError,
                format!("JSON serialization failed: {}", e),
            )
        })?;

        Ok(json_str.into_bytes())
    }

    fn decode<T>(&self, bytes: &[u8]) -> RpcResult<T>
    where
        T: DeserializeOwned,
    {
        let json_str = std::str::from_utf8(bytes).map_err(|e| {
            RpcError::new(RpcErrorCode::ParseError, format!("Invalid UTF-8: {}", e))
        })?;

        serde_json::from_str(json_str).map_err(|e| {
            RpcError::new(
                RpcErrorCode::ParseError,
                format!("JSON parsing failed: {}", e),
            )
        })
    }

    fn name(&self) -> &'static str {
        "json"
    }

    fn mime_type(&self) -> &'static str {
        "application/json"
    }
}
