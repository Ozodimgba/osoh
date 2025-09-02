use crate::error::types::{RpcError, RpcErrorCode};
use std::convert::From;

/// Convert from common error types
impl From<std::io::Error> for RpcError {
    fn from(err: std::io::Error) -> Self {
        use std::io::ErrorKind;

        let (code, message) = match err.kind() {
            ErrorKind::NotFound => (RpcErrorCode::AccountNotFound, "Resource not found"),
            ErrorKind::PermissionDenied => (RpcErrorCode::Forbidden, "Permission denied"),
            ErrorKind::ConnectionRefused => (RpcErrorCode::ConnectionError, "Connection refused"),
            ErrorKind::TimedOut => (RpcErrorCode::Timeout, "Operation timed out"),
            _ => (RpcErrorCode::InternalError, "I/O error"),
        };

        RpcError::with_data(
            code,
            format!("{}: {}", message, err),
            serde_json::json!({ "io_error": err.to_string() }),
        )
    }
}

impl From<serde_json::Error> for RpcError {
    fn from(err: serde_json::Error) -> Self {
        RpcError::with_data(
            RpcErrorCode::ParseError,
            format!("JSON parsing error: {}", err),
            serde_json::json!({ "json_error": err.to_string() }),
        )
    }
}

impl From<bincode::error::EncodeError> for RpcError {
    fn from(err: bincode::error::EncodeError) -> Self {
        RpcError::with_data(
            RpcErrorCode::ParseError,
            format!("Binary serialization error: {}", err),
            serde_json::json!({ "bincode_error": err.to_string() }),
        )
    }
}

impl From<std::num::ParseIntError> for RpcError {
    fn from(err: std::num::ParseIntError) -> Self {
        RpcError::with_data(
            RpcErrorCode::InvalidParams,
            format!("Invalid number format: {}", err),
            serde_json::json!({ "parse_error": err.to_string() }),
        )
    }
}

impl From<url::ParseError> for RpcError {
    fn from(err: url::ParseError) -> Self {
        RpcError::with_data(
            RpcErrorCode::InvalidParams,
            format!("Invalid URL format: {}", err),
            serde_json::json!({ "url_error": err.to_string() }),
        )
    }
}

/// Convert from solana-sdk errors (if using solana-sdk)
#[cfg(feature = "solana-sdk")]
impl From<solana_sdk::pubkey::ParsePubkeyError> for RpcError {
    fn from(err: solana_sdk::pubkey::ParsePubkeyError) -> Self {
        RpcError::with_data(
            RpcErrorCode::InvalidPubkey,
            format!("Invalid public key: {}", err),
            serde_json::json!({ "pubkey_error": err.to_string() }),
        )
    }
}

#[cfg(feature = "solana-sdk")]
impl From<solana_sdk::signature::ParseSignatureError> for RpcError {
    fn from(err: solana_sdk::signature::ParseSignatureError) -> Self {
        RpcError::with_data(
            RpcErrorCode::InvalidSignature,
            format!("Invalid signature: {}", err),
            serde_json::json!({ "signature_error": err.to_string() }),
        )
    }
}

/// Helper trait for converting Results
pub trait MapRpcError<T> {
    fn map_rpc_error(self, code: RpcErrorCode, message: &str) -> Result<T, RpcError>;
}

impl<T, E> MapRpcError<T> for Result<T, E>
where
    E: std::fmt::Display,
{
    fn map_rpc_error(self, code: RpcErrorCode, message: &str) -> Result<T, RpcError> {
        self.map_err(|e| {
            RpcError::with_data(
                code,
                format!("{}: {}", message, e),
                serde_json::json!({ "original_error": e.to_string() }),
            )
        })
    }
}
