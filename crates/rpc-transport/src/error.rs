//! Transport layer error types

use std::time::Duration;
use thiserror::Error;

pub type TransportResult<T> = Result<T, TransportError>;

#[derive(Error, Debug)]
pub enum TransportError {
    // Connection errors
    #[error("Failed to establish connection: {reason}")]
    ConnectionFailed { reason: String },

    #[error("Connection lost")]
    ConnectionLost,

    #[error("Connection timeout after {timeout:?}")]
    ConnectionTimeout { timeout: Duration },

    // Stream errors
    #[error("Stream closed unexpectedly")]
    StreamClosed,

    #[error("Stream reset by peer")]
    StreamReset,

    #[error("Stream error")]
    StreamError { reason: String },

    // I/O errors
    #[error("Failed to send data: {reason}")]
    SendFailed { reason: String },

    #[error("Failed to receive data: {reason}")]
    ReceiveFailed { reason: String },

    #[error("Incomplete read: expected {expected} bytes, got {actual}")]
    IncompleteRead { expected: usize, actual: usize },

    // Protocol errors
    #[error("Invalid message frame: {reason}")]
    InvalidFrame { reason: String },

    #[error("Message too large: {size} bytes exceeds limit {limit}")]
    MessageTooLarge { size: usize, limit: usize },

    #[error("I/O error: {reason}")]
    IOError { reason: String },

    #[error("Protocol error: {reason}")]
    ProtocolError { reason: String },

    // Security errors
    #[error("TLS handshake failed")]
    TlsError(#[from] rustls::Error),

    #[error("Certificate verification failed: {reason}")]
    CertificateError { reason: String },

    // Configuration errors
    #[error("Invalid configuration: {reason}")]
    ConfigError { reason: String },

    // Timeout errors
    #[error("Operation timeout after {duration:?}")]
    Timeout {
        duration: Duration,
        operation: &'static str,
    },

    // Quinn-specific errors
    #[error("QUIC connection error")]
    QuicConnectionError(#[from] quinn::ConnectionError),

    #[error("QUIC write error")]
    QuicWriteError(#[from] quinn::WriteError),

    #[error("QUIC read error")]
    QuicReadError(#[from] quinn::ReadError),

    // Integration errors
    #[error("Codec error")]
    CodecError(#[from] rpc_codec::RpcError),

    #[error("Codec error")]
    NoAvailableStreams,

    // Generic errors
    #[error("Internal error: {message}")]
    Internal { message: String },
}

impl TransportError {
    /// Check if this error is recoverable (can retry)
    pub fn is_recoverable(&self) -> bool {
        match self {
            Self::ConnectionTimeout { .. } => true,
            Self::SendFailed { .. } => true,
            Self::ReceiveFailed { .. } => true,
            Self::ConnectionLost => true,
            Self::StreamClosed => false, // Don't retry on stream close
            Self::StreamReset => false,
            Self::TlsError(_) => false,
            Self::CertificateError { .. } => false,
            Self::ConfigError { .. } => false,
            Self::MessageTooLarge { .. } => false,
            Self::InvalidFrame { .. } => false,
            _ => false, // Conservative: don't retry unknown errors
        }
    }

    /// Check if this is a connection-level error (need new connection)
    pub fn is_connection_error(&self) -> bool {
        match self {
            Self::ConnectionFailed { .. } => true,
            Self::ConnectionLost => true,
            Self::ConnectionTimeout { .. } => true,
            Self::TlsError(_) => true,
            Self::CertificateError { .. } => true,
            Self::QuicConnectionError(_) => true,
            _ => false,
        }
    }

    /// Check if this is a stream-level error (can retry on new stream)
    pub fn is_stream_error(&self) -> bool {
        match self {
            Self::StreamClosed => true,
            Self::StreamReset => true,
            Self::QuicWriteError(_) => true,
            Self::QuicReadError(_) => true,
            Self::SendFailed { .. } => true,
            Self::ReceiveFailed { .. } => true,
            _ => false,
        }
    }
}
