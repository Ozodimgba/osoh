use crate::protocol::types::*;
use serde::{Deserialize, Serialize};

// Account Methods

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GetAccountInfoRequest {
    pub pubkey: Pubkey,
    pub commitment: Option<Commitment>,
    pub encoding: Option<Encoding>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GetAccountInfoResponse {
    pub context: RpcResponseContext,
    pub value: Option<Account>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GetBalanceRequest {
    pub pubkey: Pubkey,
    pub commitment: Option<Commitment>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GetBalanceResponse {
    pub context: RpcResponseContext,
    pub value: u64,
}

// Transaction Methods

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SendTransactionRequest {
    pub transaction: String,
    pub commitment: Option<Commitment>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SendTransactionResponse {
    pub signature: Signature,
}

// ============================================================================
// Block Methods
// ============================================================================

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GetBlockRequest {
    pub slot: u64,
    pub commitment: Option<Commitment>,
    pub encoding: Option<Encoding>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GetBlockResponse {
    pub blockhash: Blockhash,
    pub parent_slot: u64,
    pub transactions: Vec<String>,
}
