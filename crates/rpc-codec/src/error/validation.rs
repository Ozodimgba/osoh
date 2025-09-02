use crate::error::builder::RpcErrorBuilder;
use crate::error::types::{RpcError, RpcErrorCode};

/// Validation helpers for common RPC inputs
pub struct RpcValidator;

impl RpcValidator {
    /// Validate a Solana public key string
    pub fn validate_pubkey(pubkey: &str) -> Result<(), RpcError> {
        if pubkey.is_empty() {
            return Err(RpcErrorBuilder::new(RpcErrorCode::InvalidPubkey)
                .message("Public key cannot be empty")
                .with_validation_error("pubkey", "empty")
                .build());
        }

        if pubkey.len() != 44 && pubkey.len() != 43 {
            return Err(RpcErrorBuilder::new(RpcErrorCode::InvalidPubkey)
                .message("Public key must be 43-44 characters in base58")
                .with_validation_error("pubkey", format!("invalid_length: {}", pubkey.len()))
                .with_pubkey(pubkey)
                .build());
        }

        // Additional base58 validation could go here
        Ok(())
    }

    /// Validate a transaction signature
    pub fn validate_signature(signature: &str) -> Result<(), RpcError> {
        if signature.is_empty() {
            return Err(RpcErrorBuilder::new(RpcErrorCode::InvalidSignature)
                .message("Signature cannot be empty")
                .with_validation_error("signature", "empty")
                .build());
        }

        if signature.len() != 88 {
            return Err(RpcErrorBuilder::new(RpcErrorCode::InvalidSignature)
                .message("Signature must be 88 characters in base58")
                .with_validation_error("signature", format!("invalid_length: {}", signature.len()))
                .with_signature(signature)
                .build());
        }

        Ok(())
    }

    /// Validate slot number
    pub fn validate_slot(slot: u64) -> Result<(), RpcError> {
        if slot == 0 {
            return Err(RpcErrorBuilder::new(RpcErrorCode::InvalidParams)
                .message("Slot must be greater than 0")
                .with_validation_error("slot", "zero")
                .with_slot(slot)
                .build());
        }

        // Could add max slot validation here
        Ok(())
    }

    /// Validate commitment level
    pub fn validate_commitment(commitment: &str) -> Result<(), RpcError> {
        match commitment.to_lowercase().as_str() {
            "processed" | "confirmed" | "finalized" => Ok(()),
            _ => Err(RpcErrorBuilder::new(RpcErrorCode::InvalidCommitment)
                .message("Commitment must be 'processed', 'confirmed', or 'finalized'")
                .with_validation_error("commitment", format!("invalid_value: {}", commitment))
                .build()),
        }
    }

    /// Validate encoding
    pub fn validate_encoding(encoding: &str) -> Result<(), RpcError> {
        match encoding.to_lowercase().as_str() {
            "base58" | "base64" | "jsonparsed" => Ok(()),
            _ => Err(RpcErrorBuilder::new(RpcErrorCode::InvalidEncoding)
                .message("Encoding must be 'base58', 'base64', or 'jsonParsed'")
                .with_validation_error("encoding", format!("invalid_value: {}", encoding))
                .build()),
        }
    }
}

/// Validation macros
#[macro_export]
macro_rules! validate {
    ($validator_fn:expr) => {
        match $validator_fn {
            Ok(_) => {}
            Err(e) => return Err(e),
        }
    };
}

#[macro_export]
macro_rules! validate_required {
    ($field:expr, $name:expr) => {
        if $field.is_none() {
            return Err($crate::error::RpcErrorBuilder::new(
                $crate::error::RpcErrorCode::InvalidParams,
            )
            .message(format!("Required parameter '{}' is missing", $name))
            .with_validation_error($name, "missing")
            .build());
        }
    };
}
