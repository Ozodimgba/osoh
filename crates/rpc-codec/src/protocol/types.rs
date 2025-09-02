use serde::{Deserialize, Serialize};

/// A Solana public key
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Pubkey(pub String);

/// Account information
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Account {
    pub lamports: u64,
    pub data: Vec<u8>,
    pub owner: Pubkey,
    pub executable: bool,
    pub rent_epoch: u64,
}

/// Transaction signature
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Signature(pub String);

/// Block hash
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Blockhash(pub String);

/// RPC response context
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct RpcResponseContext {
    pub slot: u64,
}

/// Commitment level
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Commitment {
    Processed,
    Confirmed,
    Finalized,
}

/// Encoding options
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Encoding {
    Base58,
    Base64,
    JsonParsed,
}
