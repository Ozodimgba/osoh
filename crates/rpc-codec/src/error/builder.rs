use crate::error::types::{RpcError, RpcErrorCode};
use serde_json::Value;
use std::collections::HashMap;

/// Builder for constructing RPC errors with rich context
#[derive(Debug, Clone)]
pub struct RpcErrorBuilder {
    code: RpcErrorCode,
    message: String,
    data: HashMap<String, Value>,
}

impl RpcErrorBuilder {
    /// Start building an error with the given code
    pub fn new(code: RpcErrorCode) -> Self {
        Self {
            code,
            message: code.description().to_string(),
            data: HashMap::new(),
        }
    }

    /// Set the error message
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    /// Add a data field
    pub fn with_data(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        self.data.insert(key.into(), value.into());
        self
    }

    /// Add multiple data fields
    pub fn with_context(mut self, context: impl Into<HashMap<String, Value>>) -> Self {
        self.data.extend(context.into());
        self
    }

    /// Add pubkey to error context (common for Solana errors)
    pub fn with_pubkey(mut self, pubkey: impl Into<String>) -> Self {
        self.data
            .insert("pubkey".to_string(), Value::String(pubkey.into()));
        self
    }

    /// Add signature to error context
    pub fn with_signature(mut self, signature: impl Into<String>) -> Self {
        self.data
            .insert("signature".to_string(), Value::String(signature.into()));
        self
    }

    /// Add slot to error context
    pub fn with_slot(mut self, slot: u64) -> Self {
        self.data
            .insert("slot".to_string(), Value::Number(slot.into()));
        self
    }

    /// Add validation details
    pub fn with_validation_error(
        mut self,
        field: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        let validation = serde_json::json!({
            "field": field.into(),
            "reason": reason.into()
        });
        self.data.insert("validation_error".to_string(), validation);
        self
    }

    /// Build the final error
    pub fn build(self) -> RpcError {
        if self.data.is_empty() {
            RpcError::new(self.code, self.message)
        } else {
            RpcError::with_data(
                self.code,
                self.message,
                Value::Object(self.data.into_iter().collect()),
            )
        }
    }
}

/// Convenient macros for error construction
#[macro_export]
macro_rules! rpc_error {
    ($code:expr, $message:expr) => {
        $crate::error::RpcError::new($code, $message)
    };

    ($code:expr, $message:expr, $($key:expr => $value:expr),+ $(,)?) => {
        $crate::error::RpcErrorBuilder::new($code)
            .message($message)
            $(
                .with_data($key, $value)
            )+
            .build()
    };
}

#[macro_export]
macro_rules! account_not_found {
    ($pubkey:expr) => {
        $crate::error::RpcErrorBuilder::new($crate::error::RpcErrorCode::AccountNotFound)
            .with_pubkey($pubkey)
            .build()
    };
}

#[macro_export]
macro_rules! invalid_params {
    ($field:expr, $reason:expr) => {
        $crate::error::RpcErrorBuilder::new($crate::error::RpcErrorCode::InvalidParams)
            .with_validation_error($field, $reason)
            .build()
    };
}
