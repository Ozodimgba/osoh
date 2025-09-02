use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::SystemTime;

pub type RpcResult<T> = Result<T, RpcError>;

/// Main RPC error type
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct RpcError {
    /// Standardized error code
    pub code: RpcErrorCode,

    /// Human-readable error message
    pub message: String,

    /// Additional error context/details
    pub data: Option<serde_json::Value>,

    /// Stack trace (only in debug builds)
    #[cfg(debug_assertions)]
    pub stack_trace: Option<String>,

    /// Error occurred at this timestamp
    pub timestamp: SystemTime,
}

/// Standardized error codes
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum RpcErrorCode {
    // Client errors (1000-1999)
    InvalidRequest = 1000,
    MethodNotFound = 1001,
    InvalidParams = 1002,
    ParseError = 1003,
    InvalidMethod = 1004,
    RateLimited = 1005,
    Unauthorized = 1006,
    Forbidden = 1007,

    // Server errors (2000-2999)
    InternalError = 2000,
    ServiceUnavailable = 2001,
    Timeout = 2002,
    NotImplemented = 2003,

    // Solana-specific errors (3000-3999)
    AccountNotFound = 3000,
    TransactionNotFound = 3001,
    BlockNotFound = 3002,
    InvalidSignature = 3003,
    InsufficientFunds = 3004,
    TransactionExpired = 3005,
    ProgramError = 3006,
    InvalidAccountData = 3007,
    SlotNotFound = 3008,

    // Network errors (4000-4999)
    ConnectionError = 4000,
    NetworkTimeout = 4001,
    HostUnreachable = 4002,

    // Validation errors (5000-5999)
    InvalidPubkey = 5000,
    InvalidTransaction = 5001,
    InvalidBlockhash = 5002,
    InvalidCommitment = 5003,
    InvalidEncoding = 5004,
}

impl RpcError {
    /// Create a new RPC error
    pub fn new(code: RpcErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
            #[cfg(debug_assertions)]
            stack_trace: Some(format!("{:?}", std::backtrace::Backtrace::capture())),
            timestamp: SystemTime::now(),
        }
    }

    /// Create error with additional data
    pub fn with_data(
        code: RpcErrorCode,
        message: impl Into<String>,
        data: serde_json::Value,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            data: Some(data),
            #[cfg(debug_assertions)]
            stack_trace: Some(format!("{:?}", std::backtrace::Backtrace::capture())),
            timestamp: SystemTime::now(),
        }
    }

    // Convenience constructors for common errors

    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self::new(RpcErrorCode::InvalidRequest, message)
    }

    pub fn method_not_found(message: impl Into<String>) -> Self {
        Self::new(RpcErrorCode::MethodNotFound, message)
    }

    pub fn invalid_params(message: impl Into<String>) -> Self {
        Self::new(RpcErrorCode::InvalidParams, message)
    }

    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::new(RpcErrorCode::InternalError, message)
    }

    pub fn account_not_found(pubkey: impl Into<String>) -> Self {
        Self::with_data(
            RpcErrorCode::AccountNotFound,
            "Account not found",
            serde_json::json!({ "pubkey": pubkey.into() }),
        )
    }

    pub fn transaction_not_found(signature: impl Into<String>) -> Self {
        Self::with_data(
            RpcErrorCode::TransactionNotFound,
            "Transaction not found",
            serde_json::json!({ "signature": signature.into() }),
        )
    }

    pub fn rate_limited(retry_after: Option<u64>) -> Self {
        let data = if let Some(seconds) = retry_after {
            Some(serde_json::json!({ "retry_after_seconds": seconds }))
        } else {
            None
        };

        Self {
            code: RpcErrorCode::RateLimited,
            message: "Rate limit exceeded".to_string(),
            data,
            #[cfg(debug_assertions)]
            stack_trace: Some(format!("{:?}", std::backtrace::Backtrace::capture())),
            timestamp: SystemTime::now(),
        }
    }

    /// Check if this is a client error (4xx equivalent)
    pub fn is_client_error(&self) -> bool {
        let code_num = self.code as u32;
        (1000..2000).contains(&code_num) || (5000..6000).contains(&code_num)
    }

    /// Check if this is a server error (5xx equivalent)
    pub fn is_server_error(&self) -> bool {
        let code_num = self.code as u32;
        (2000..3000).contains(&code_num) || (4000..5000).contains(&code_num)
    }

    /// Check if this is a Solana-specific error
    pub fn is_solana_error(&self) -> bool {
        let code_num = self.code as u32;
        (3000..4000).contains(&code_num)
    }
}

impl fmt::Display for RpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code as u32, self.message)
    }
}

impl std::error::Error for RpcError {}

impl RpcErrorCode {
    /// Get human-readable description of the error code
    pub fn description(&self) -> &'static str {
        match self {
            Self::InvalidRequest => "The JSON sent is not a valid request object",
            Self::MethodNotFound => "The method does not exist or is not available",
            Self::InvalidParams => "Invalid method parameters",
            Self::ParseError => "Invalid JSON was received",
            Self::InvalidMethod => "The method is not a valid RPC method name",
            Self::RateLimited => "Too many requests",
            Self::Unauthorized => "Authentication required",
            Self::Forbidden => "Access denied",

            Self::InternalError => "Internal RPC error",
            Self::ServiceUnavailable => "Service temporarily unavailable",
            Self::Timeout => "Request timed out",
            Self::NotImplemented => "Method not implemented",

            Self::AccountNotFound => "Account does not exist",
            Self::TransactionNotFound => "Transaction not found",
            Self::BlockNotFound => "Block not found",
            Self::InvalidSignature => "Invalid transaction signature",
            Self::InsufficientFunds => "Insufficient funds for transaction",
            Self::TransactionExpired => "Transaction has expired",
            Self::ProgramError => "Program execution error",
            Self::InvalidAccountData => "Account data is invalid",
            Self::SlotNotFound => "Slot not found",

            Self::ConnectionError => "Connection error",
            Self::NetworkTimeout => "Network timeout",
            Self::HostUnreachable => "Host unreachable",

            Self::InvalidPubkey => "Invalid public key format",
            Self::InvalidTransaction => "Invalid transaction format",
            Self::InvalidBlockhash => "Invalid blockhash format",
            Self::InvalidCommitment => "Invalid commitment level",
            Self::InvalidEncoding => "Invalid encoding specified",
        }
    }

    /// Get HTTP status code equivalent
    pub fn http_status_code(&self) -> u16 {
        match self {
            Self::InvalidRequest | Self::InvalidParams | Self::ParseError | Self::InvalidMethod => {
                400
            } // Bad Request

            Self::Unauthorized => 401,   // Unauthorized
            Self::Forbidden => 403,      // Forbidden
            Self::MethodNotFound => 404, // Not Found
            Self::RateLimited => 429,    // Too Many Requests

            Self::InternalError | Self::ProgramError => 500, // Internal Server Error
            Self::NotImplemented => 501,                     // Not Implemented
            Self::ServiceUnavailable => 503,                 // Service Unavailable
            Self::Timeout | Self::NetworkTimeout => 504,     // Gateway Timeout

            // Solana-specific errors map to 400 (client error) or 404 (not found)
            Self::AccountNotFound
            | Self::TransactionNotFound
            | Self::BlockNotFound
            | Self::SlotNotFound => 404,

            Self::InvalidSignature
            | Self::InsufficientFunds
            | Self::TransactionExpired
            | Self::InvalidAccountData
            | Self::InvalidPubkey
            | Self::InvalidTransaction
            | Self::InvalidBlockhash
            | Self::InvalidCommitment
            | Self::InvalidEncoding => 400,

            Self::ConnectionError | Self::HostUnreachable => 502, // Bad Gateway
        }
    }
}

impl fmt::Display for RpcErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", *self as u32)
    }
}
