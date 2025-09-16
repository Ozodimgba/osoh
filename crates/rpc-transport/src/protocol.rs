//! Application Protocol layer for RPC transport over QUIC streams
//!
//! This module defines how RpcMessage<T> (from rpc-codec) flows through QUIC streams.
//! Design principle: Start simple, leave room for sophisticated features later.

use bincode::{config, serde::encode_to_vec};
pub use rpc_codec::protocol::{MessageType, RpcMessage, constants::method_ids};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StreamedMessage<T> {
    pub message: RpcMessage<T>,
    pub priority: MessagePriority,
    pub timeout: Duration,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, Copy, Hash, Eq, PartialEq)]
pub enum MessagePriority {
    Low = 0, // Background tasks, analytics

    #[default]
    Normal = 1, // Regular RPC calls

    High = 2,     // User-facing requests
    Critical = 3, // Error handling, heartbeats
}

impl<T: Serialize> StreamedMessage<T> {
    pub fn new(message: RpcMessage<T>) -> Self {
        Self {
            message,
            priority: MessagePriority::default(),
            timeout: Duration::from_secs(30), // default timeout for normal priority
        }
    }

    pub fn get_stream_payload(&self) -> &T {
        &self.message.payload
    }

    pub fn with_priority(mut self, priority: MessagePriority) -> Self {
        self.priority = priority;
        self
    }

    ///Validate the message against the given stream configuration.
    pub fn validate(&self, config: &crate::config::QuicConfig) -> Result<(), ProtocolError> {
        self.validate_method_id()?;
        self.validate_size(config.max_message_size)?;
        self.validate_timeout(config.max_idle_timeout)?;
        Ok(())
    }

    fn validate_method_id(&self) -> Result<(), ProtocolError> {
        if !method_ids::is_valid(self.message.method_id) {
            return Err(ProtocolError::MethodNotFound {
                method: self.message.method_id,
            });
        }
        Ok(())
    }

    fn validate_timeout(&self, max_timeout: Duration) -> Result<(), ProtocolError> {
        if self.timeout > max_timeout {
            return Err(ProtocolError::Timeout {
                duration: self.timeout,
                operation: "Timeout exceeds maximum allowed",
            });
        }
        Ok(())
    }
}

// tech debt: serialized generic message size method should be defined in codec crate
impl<T: Serialize> StreamedMessage<T> {
    fn validate_size(&self, max_size: usize) -> Result<(), ProtocolError> {
        let config = config::standard();
        let encoded = encode_to_vec(self, config).map_err(|_| ProtocolError::CodecSizeError {
            info: "Error: Bincode message length error",
        })?;

        if encoded.len() > max_size {
            return Err(ProtocolError::message_too_large(encoded.len(), max_size));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProtocolError {
    MethodNotFound {
        method: u16,
    },
    Timeout {
        duration: Duration,
        operation: &'static str,
    },
    MessageTooLarge {
        size: usize,
        limit: usize,
    },
    CodecSizeError {
        info: &'static str,
    },
}
impl ProtocolError {
    pub fn method_not_found(method: u16) -> Self {
        Self::MethodNotFound { method }
    }

    pub fn timeout(duration: Duration, operation: &'static str) -> Self {
        Self::Timeout {
            duration,
            operation,
        }
    }

    pub fn message_too_large(size: usize, limit: usize) -> Self {
        Self::MessageTooLarge { size, limit }
    }
}
