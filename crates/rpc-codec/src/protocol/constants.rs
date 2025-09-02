/// Current protocol version
pub const PROTOCOL_VERSION: u8 = 1;

/// Maximum message size (10MB)
pub const MAX_MESSAGE_SIZE: usize = 10 * 1024 * 1024;

/// Maximum request ID (for wraparound)
pub const MAX_REQUEST_ID: u64 = u64::MAX;

/// Method IDs for all RPC methods
pub mod method_ids {
    // Account methods
    pub const GET_ACCOUNT_INFO: u16 = 1001;
    pub const GET_MULTIPLE_ACCOUNTS: u16 = 1002;
    pub const GET_BALANCE: u16 = 1003;

    // Transaction methods
    pub const SEND_TRANSACTION: u16 = 2001;
    pub const GET_TRANSACTION: u16 = 2002;
    pub const SIMULATE_TRANSACTION: u16 = 2003;

    // Block methods
    pub const GET_BLOCK: u16 = 3001;
    pub const GET_BLOCK_HEIGHT: u16 = 3002;
    pub const GET_RECENT_BLOCKHASH: u16 = 3003;

    // Subscription methods
    pub const ACCOUNT_SUBSCRIBE: u16 = 4001;
    pub const ACCOUNT_UNSUBSCRIBE: u16 = 4002;
    pub const PROGRAM_SUBSCRIBE: u16 = 4003;
    pub const PROGRAM_UNSUBSCRIBE: u16 = 4004;
}

/// Error codes
pub mod error_codes {
    pub const INVALID_REQUEST: u32 = 1000;
    pub const METHOD_NOT_FOUND: u32 = 1001;
    pub const INVALID_PARAMS: u32 = 1002;
    pub const INTERNAL_ERROR: u32 = 1003;
    pub const ACCOUNT_NOT_FOUND: u32 = 2001;
    pub const TRANSACTION_NOT_FOUND: u32 = 2002;
    pub const BLOCK_NOT_FOUND: u32 = 3001;
}
