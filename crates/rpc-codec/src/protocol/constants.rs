/// Current protocol version
pub const PROTOCOL_VERSION: u8 = 1;

/// Max message size defined in config on transport
// pub const MAX_MESSAGE_SIZE: usize = 10 * 1024 * 1024;

/// Maximum request ID (for wraparound)
pub const MAX_REQUEST_ID: u64 = u64::MAX;

/// Method IDs for all RPC methods
pub mod method_ids {
    use crate::define_methods;
    // Account methods
    define_methods![
        (GET_ACCOUNT_INFO, 1001),
        (GET_MULTIPLE_ACCOUNTS, 1002),
        (GET_BALANCE, 1003),
        (SEND_TRANSACTION, 2001),
        (GET_TRANSACTION, 2002),
        (SIMULATE_TRANSACTION, 2003),
        (GET_BLOCK, 3001),
        (GET_BLOCK_HEIGHT, 3002),
        (GET_RECENT_BLOCKHASH, 3003),
        (ACCOUNT_SUBSCRIBE, 4001),
        (ACCOUNT_UNSUBSCRIBE, 4002),
        (PROGRAM_SUBSCRIBE, 4003),
        (PROGRAM_UNSUBSCRIBE, 4004)
    ]; // macro so I can check valid methods DRY-ly
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
