//! I/O specific error types

use std::time::Duration;
use thiserror::Error;

pub type IoResult<T> = Result<T, IoError>;

#[derive(Error, Debug)]
pub enum IoError {
    // Socket errors
    #[error("Failed to bind socket to {addr}: {reason}")]
    BindFailed {
        addr: std::net::SocketAddr,
        reason: String,
    },

    #[error("Failed to send {bytes} bytes: {reason}")]
    SendFailed { bytes: usize, reason: String },

    #[error("Failed to receive data: {reason}")]
    ReceiveFailed { reason: String },

    #[error("Socket is closed")]
    SocketClosed,

    // I/O operation errors
    #[error("Operation timeout after {duration:?}")]
    Timeout { duration: Duration },

    #[error("Buffer too small: need {needed} bytes, have {available}")]
    BufferTooSmall { needed: usize, available: usize },

    #[error("Invalid address: {addr}")]
    InvalidAddress { addr: String },

    // Runtime errors
    #[error("Runtime not available: {runtime}")]
    RuntimeUnavailable { runtime: String },

    #[error("Feature not supported: {feature}")]
    FeatureNotSupported { feature: String },

    // io_uring specific errors
    #[error("io_uring error: {code}")]
    IoUringError { code: i32 },

    #[error("io_uring queue full")]
    IoUringQueueFull,

    // System errors
    #[error("System error")]
    SystemError(#[from] std::io::Error),

    #[error("Address parse error")]
    AddrParseError(#[from] std::net::AddrParseError),

    // Internal errors
    #[error("Internal error: {message}")]
    Internal { message: String },
}

impl IoError {
    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Timeout { .. } => true,
            Self::SendFailed { .. } => true,
            Self::ReceiveFailed { .. } => true,
            Self::IoUringQueueFull => true,
            Self::SystemError(e) => e.kind() == std::io::ErrorKind::WouldBlock,
            _ => false,
        }
    }

    /// Check if this is a resource exhaustion error
    pub fn is_resource_exhausted(&self) -> bool {
        match self {
            Self::IoUringQueueFull => true,
            Self::BufferTooSmall { .. } => true,
            Self::SystemError(e) => matches!(
                e.kind(),
                std::io::ErrorKind::OutOfMemory | std::io::ErrorKind::ResourceBusy
            ),
            _ => false,
        }
    }
}
