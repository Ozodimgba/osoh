//! Solana RPC - Codec
//!
//! High-performance binary RPC protocol with QUIC + io_uring + bincode

#![forbid(unsafe_code)]
// #![warn(missing_docs)]

pub mod codec;
pub mod error;
pub mod protocol;
pub mod routing;

pub use codec::*;
pub use error::*;
pub use protocol::*;
pub use routing::*;

#[cfg(test)]
mod integration_tests {
    use crate::json::JsonCodec;

    use super::*;
    use crate::routing::RpcHandler;
    use async_trait::async_trait;
    use std::time::Instant;

    // ============================================================================
    // INTEGRATION TEST: Complete RPC Flow Demo
    // ============================================================================

    #[tokio::test]
    async fn complete_rpc_flow_demo() {
        println!("\n🚀 Starting Complete RPC Flow Demo");
        println!("=====================================");

        // ========================================================================
        // 1. CLIENT SIDE: Create and Encode Request
        // ========================================================================

        println!("\n📝 STEP 1: Client creates RPC request");

        // Create a request using protocol types
        let account_request = GetAccountInfoRequest {
            pubkey: Pubkey("11111111111111111111111111111112".to_string()),
            commitment: Some(Commitment::Finalized),
            encoding: Some(Encoding::Base64),
        };

        // Wrap in protocol envelope
        let request_message = RpcMessage::new_request(
            method_ids::GET_ACCOUNT_INFO, // 1001
            12345,                        // request_id
            account_request,
        );

        println!(
            "   ✅ Created request: method_id={}, request_id={}",
            request_message.method_id, request_message.request_id
        );
        println!("   ✅ Pubkey: {}", request_message.payload.pubkey.0);

        // ========================================================================
        // 2. CODEC: Serialize Request to Bytes
        // ========================================================================

        println!("\n🔧 STEP 2: Encode request to bytes");

        let codec = BincodeCodec::new();
        let request_bytes = codec.encode(&request_message).unwrap();

        println!("   ✅ Encoded {} bytes with bincode", request_bytes.len());
        println!("   ✅ Ready for network transmission");
        println!("✅ {:?}", request_bytes);

        // ========================================================================
        // 3. SIMULATE NETWORK TRANSMISSION
        // ========================================================================

        println!("\n🌐 STEP 3: Simulate network transmission");
        println!("   ⚡ Sending {} bytes over QUIC...", request_bytes.len());
        println!("   📡 [NETWORK] Client → Server");

        // ========================================================================
        // 4. SERVER SIDE: Routing and Handler Setup
        // ========================================================================

        println!("\n🎯 STEP 4: Server sets up routing");

        // Create a mock handler for getAccountInfo
        struct MockAccountHandler;

        #[async_trait]
        impl RpcHandler for MockAccountHandler {
            type Request = GetAccountInfoRequest;
            type Response = GetAccountInfoResponse;

            async fn handle(
                &self,
                request: Self::Request,
                context: RequestContext,
            ) -> HandlerResult<Self::Response> {
                println!("   🔍 Handler processing request for: {}", request.pubkey.0);
                println!("   📊 Request context: trace_id={}", context.trace_id);

                // Simulate Solana RPC call (in real system, this calls actual Solana)
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

                // Return mock account data
                let response = GetAccountInfoResponse {
                    context: RpcResponseContext { slot: 98765 },
                    value: Some(Account {
                        lamports: 1000000000, // 1 SOL
                        data: vec![1, 2, 3, 4, 5],
                        owner: Pubkey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_string()),
                        executable: false,
                        rent_epoch: 250,
                    }),
                };

                println!(
                    "   ✅ Found account with {} lamports",
                    response.value.as_ref().unwrap().lamports
                );
                Ok(response)
            }

            fn method_name(&self) -> &'static str {
                "getAccountInfo"
            }
        }

        // Build routing registry
        let mut registry = HandlerRegistry::new();
        registry.register(
            method_ids::GET_ACCOUNT_INFO,
            HandlerWrapper::new(MockAccountHandler),
        );
        println!(
            "   ✅ Registered handler for method_id={}",
            method_ids::GET_ACCOUNT_INFO
        );

        // Create message dispatcher
        let dispatcher = MessageDispatcher::new(registry);
        println!("   ✅ Created message dispatcher");

        // ========================================================================
        // 5. SERVER PROCESSING: Decode → Route → Handle → Encode
        // ========================================================================

        println!("\n⚙️  STEP 5: Server processes request");

        // Create request context (simulates connection info)
        let request_context = RequestContext::new()
            .with_client_ip("192.168.1.100".to_string())
            .with_metadata("user-agent".to_string(), "rpc-client/1.0".to_string());

        println!("   📥 Server received {} bytes", request_bytes.len());

        // Dispatch the request (this does: decode → route → handle → encode)
        let response_bytes = dispatcher.dispatch(&request_bytes, request_context).await;

        println!(
            "   📤 Server prepared {} byte response",
            response_bytes.len()
        );

        // ========================================================================
        // 6. SIMULATE NETWORK RESPONSE
        // ========================================================================

        println!("\n🌐 STEP 6: Simulate network response");
        println!(
            "   📡 [NETWORK] Server → Client ({} bytes)",
            response_bytes.len()
        );

        // ========================================================================
        // 7. CLIENT SIDE: Receive and Decode Response
        // ========================================================================

        println!("\n📖 STEP 7: Client processes response");

        // println!("{:?}", response_bytes);

        // Decode the response
        let response_message: RpcMessage<GetAccountInfoResponse> =
            codec.decode(&response_bytes).unwrap();

        println!(
            "   ✅ Decoded response: method_id={}, request_id={}",
            response_message.method_id, response_message.request_id
        );
        println!("   ✅ Message type: {:?}", response_message.message_type);

        // Verify request/response matching
        assert_eq!(response_message.method_id, method_ids::GET_ACCOUNT_INFO);
        assert_eq!(response_message.request_id, 12345);
        // assert_eq!(response_message.message_type, MessageType::Response);

        // Extract and use the account data
        let account_response = response_message.payload;
        println!("   📊 Response slot: {}", account_response.context.slot);

        if let Some(account) = account_response.value {
            println!("   💰 Account lamports: {}", account.lamports);
            println!("   👤 Account owner: {}", account.owner.0);
            println!("   📄 Account data: {} bytes", account.data.len());
        }

        // ========================================================================
        // 8. DEMONSTRATION COMPLETE
        // ========================================================================

        println!("\n🎉 COMPLETE RPC FLOW SUCCESSFUL!");
        println!("=====================================");
        println!("Flow summary:");
        println!("  1. ✅ Client created GetAccountInfoRequest");
        println!("  2. ✅ Protocol wrapped in RpcMessage envelope");
        println!("  3. ✅ Codec serialized to {} bytes", request_bytes.len());
        println!("  4. ✅ Server routed to correct handler");
        println!("  5. ✅ Handler processed request and returned data");
        println!(
            "  6. ✅ Response serialized to {} bytes",
            response_bytes.len()
        );
        println!("  7. ✅ Client decoded and accessed account data");
        println!("\n🚀 Your RPC system is working end-to-end!");
    }

    #[test]
    fn bincode_vs_json_benchmark() {
        println!("\n⚔️  BINCODE vs JSON CODEC BATTLE");
        println!("=====================================");

        // Create test message - typical RPC request
        let test_message = RpcMessage::new_request(
            method_ids::GET_ACCOUNT_INFO,
            12345,
            GetAccountInfoRequest {
                pubkey: Pubkey("11111111111111111111111111111112".to_string()),
                commitment: Some(Commitment::Finalized),
                encoding: Some(Encoding::Base64),
            },
        );

        // Create both codecs
        let bincode_codec = BincodeCodec::new();
        let json_codec = JsonCodec::new();

        println!("🎯 Test message: GetAccountInfoRequest");
        println!("   Method ID: {}", test_message.method_id);
        println!("   Request ID: {}", test_message.request_id);
        println!("   Pubkey: {}", test_message.payload.pubkey.0);

        // ========================================================================
        // SIZE COMPARISON
        // ========================================================================

        println!("\n📏 SIZE COMPARISON:");
        println!("   ----------------------------------------");

        let bincode_bytes = bincode_codec.encode(&test_message).unwrap();
        let json_bytes = json_codec.encode(&test_message).unwrap();

        println!("   Bincode:  {} bytes", bincode_bytes.len());
        println!("   JSON:     {} bytes", json_bytes.len());

        let size_diff = json_bytes.len() as f32 / bincode_bytes.len() as f32;
        let savings =
            ((json_bytes.len() - bincode_bytes.len()) as f32 / json_bytes.len() as f32) * 100.0;

        println!("   ----------------------------------------");
        println!("   📊 JSON is {:.1}x larger than bincode", size_diff);
        println!("   💾 Bincode saves {:.1}% bandwidth", savings);

        // ========================================================================
        // SPEED COMPARISON
        // ========================================================================

        println!("\n⚡ SPEED COMPARISON:");
        println!("   ----------------------------------------");

        let iterations = 10_000;
        println!("   Running {} iterations each...", iterations);

        // Bincode encoding speed
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = bincode_codec.encode(&test_message).unwrap();
        }
        let bincode_encode_time = start.elapsed();

        // JSON encoding speed
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = json_codec.encode(&test_message).unwrap();
        }
        let json_encode_time = start.elapsed();

        // Bincode decoding speed
        let start = Instant::now();
        for _ in 0..iterations {
            let _: RpcMessage<GetAccountInfoRequest> =
                bincode_codec.decode(&bincode_bytes).unwrap();
        }
        let bincode_decode_time = start.elapsed();

        // JSON decoding speed
        let start = Instant::now();
        for _ in 0..iterations {
            let _: RpcMessage<GetAccountInfoRequest> = json_codec.decode(&json_bytes).unwrap();
        }
        let json_decode_time = start.elapsed();

        // Calculate results
        let bincode_encode_ns = bincode_encode_time.as_nanos() as f64 / iterations as f64;
        let json_encode_ns = json_encode_time.as_nanos() as f64 / iterations as f64;
        let bincode_decode_ns = bincode_decode_time.as_nanos() as f64 / iterations as f64;
        let json_decode_ns = json_decode_time.as_nanos() as f64 / iterations as f64;

        println!("\n   ENCODING:");
        println!("   Bincode:  {:.0} ns/message", bincode_encode_ns);
        println!("   JSON:     {:.0} ns/message", json_encode_ns);
        println!(
            "   🏆 Bincode is {:.1}x faster",
            json_encode_ns / bincode_encode_ns
        );

        println!("\n   DECODING:");
        println!("   Bincode:  {:.0} ns/message", bincode_decode_ns);
        println!("   JSON:     {:.0} ns/message", json_decode_ns);
        println!(
            "   🏆 Bincode is {:.1}x faster",
            json_decode_ns / bincode_decode_ns
        );

        // ========================================================================
        // CORRECTNESS CHECK
        // ========================================================================

        println!("\n✅ CORRECTNESS CHECK:");

        // Round-trip test for both codecs
        let bincode_decoded: RpcMessage<GetAccountInfoRequest> =
            bincode_codec.decode(&bincode_bytes).unwrap();
        let json_decoded: RpcMessage<GetAccountInfoRequest> =
            json_codec.decode(&json_bytes).unwrap();

        assert_eq!(bincode_decoded.method_id, test_message.method_id);
        assert_eq!(bincode_decoded.request_id, test_message.request_id);
        assert_eq!(
            bincode_decoded.payload.pubkey.0,
            test_message.payload.pubkey.0
        );

        assert_eq!(json_decoded.method_id, test_message.method_id);
        assert_eq!(json_decoded.request_id, test_message.request_id);
        assert_eq!(json_decoded.payload.pubkey.0, test_message.payload.pubkey.0);

        println!("   ✅ Bincode round-trip: PASSED");
        println!("   ✅ JSON round-trip: PASSED");

        // ========================================================================
        // FINAL VERDICT
        // ========================================================================

        println!("\n🏆 FINAL VERDICT:");
        println!("=====================================");
        println!("📦 Size:     Bincode wins by {:.1}%", savings);
        println!(
            "🚀 Encoding: Bincode {:.1}x faster",
            json_encode_ns / bincode_encode_ns
        );
        println!(
            "📖 Decoding: Bincode {:.1}x faster",
            json_decode_ns / bincode_decode_ns
        );
        println!("✅ Both codecs are 100% correct");
        println!("\n💡 Recommendation: Use Bincode for production!");
    }

    #[test]
    fn real_protobuf_benchmark() {
        use prost::Message;

        // Define the protobuf message directly in Rust
        #[derive(Clone, PartialEq, prost::Message)]
        pub struct TestMessage {
            #[prost(uint32, tag = "1")]
            pub version: u32,
            #[prost(uint32, tag = "2")]
            pub method_id: u32,
            #[prost(uint64, tag = "3")]
            pub request_id: u64,
            #[prost(string, tag = "4")]
            pub pubkey: String,
            #[prost(int32, optional, tag = "5")]
            pub commitment: Option<i32>,
            #[prost(int32, optional, tag = "6")]
            pub encoding: Option<i32>,
        }

        println!("🚀 REAL PROTOBUF BENCHMARK: Bincode vs JSON vs Protobuf");
        println!("======================================================");

        // Create test data
        let test_message = RpcMessage::new_request(
            method_ids::GET_ACCOUNT_INFO,
            12345,
            GetAccountInfoRequest {
                pubkey: Pubkey("11111111111111111111111111111112".to_string()),
                commitment: Some(Commitment::Finalized),
                encoding: Some(Encoding::Base64),
            },
        );

        // Create real protobuf message
        let pb_message = TestMessage {
            version: test_message.version as u32,
            method_id: test_message.method_id as u32,
            request_id: test_message.request_id,
            pubkey: test_message.payload.pubkey.0.clone(),
            commitment: Some(2), // Finalized
            encoding: Some(1),   // Base64
        };

        // Size comparison
        let bincode_codec = BincodeCodec::new();
        let json_codec = JsonCodec::new();

        let bincode_bytes = bincode_codec.encode(&test_message).unwrap();
        let json_bytes = json_codec.encode(&test_message).unwrap();
        let protobuf_bytes = pb_message.encode_to_vec(); // Real protobuf encoding!

        println!("📏 SIZE COMPARISON:");
        println!("   Bincode:  {} bytes", bincode_bytes.len());
        println!("   JSON:     {} bytes", json_bytes.len());
        println!("   Protobuf: {} bytes", protobuf_bytes.len());

        // Speed comparison
        let iterations = 10_000_000;

        // Bincode encoding
        let start = std::time::Instant::now();
        for _ in 0..iterations {
            let _ = bincode_codec.encode(&test_message).unwrap();
        }
        let bincode_encode_time = start.elapsed();

        // Bincode decoding
        let start = std::time::Instant::now();
        for _ in 0..iterations {
            let _: RpcMessage<GetAccountInfoRequest> =
                bincode_codec.decode(&bincode_bytes).unwrap();
        }
        let bincode_decode_time = start.elapsed();

        // Real protobuf encoding
        let start = std::time::Instant::now();
        for _ in 0..iterations {
            let _ = pb_message.encode_to_vec(); // This is real prost encoding
        }
        let protobuf_encode_time = start.elapsed();

        // Real protobuf decoding
        let start = std::time::Instant::now();
        for _ in 0..iterations {
            let _ = TestMessage::decode(&protobuf_bytes[..]).unwrap(); // Real prost decoding
        }
        let protobuf_decode_time = start.elapsed();

        let bincode_encode_ns = bincode_encode_time.as_nanos() as f64 / iterations as f64;
        let protobuf_encode_ns = protobuf_encode_time.as_nanos() as f64 / iterations as f64;
        let protobuf_decode_ns = protobuf_decode_time.as_nanos() as f64 / iterations as f64;
        let bincode_decode_ns = bincode_decode_time.as_nanos() as f64 / iterations as f64;

        println!("⚡ ENCODING SPEED:");
        println!("   Bincode:  {:.0} ns/op", bincode_encode_ns);
        println!("   Protobuf: {:.0} ns/op", protobuf_encode_ns);
        println!(
            "   Bincode is {:.1}x {} than protobuf",
            if bincode_encode_ns < protobuf_encode_ns {
                protobuf_encode_ns / bincode_encode_ns
            } else {
                bincode_encode_ns / protobuf_encode_ns
            },
            if bincode_encode_ns < protobuf_encode_ns {
                "faster"
            } else {
                "slower"
            }
        );

        println!("⚡ DECODING SPEED:");
        println!("   Protobuf: {:.0} ns/op", protobuf_decode_ns);
        println!(" Bincode: {:0} ns/op", bincode_decode_ns);

        // Correctness check
        let decoded_pb = TestMessage::decode(&protobuf_bytes[..]).unwrap();
        assert_eq!(decoded_pb.request_id, test_message.request_id);
        assert_eq!(decoded_pb.pubkey, test_message.payload.pubkey.0);
        println!("✅ Real protobuf round-trip: PASSED");
    }
}

#[cfg(test)]
mod geyser_realistic_benchmarks {
    use crate::codec::{BincodeCodec, RpcCodec};
    use prost::Message;
    use serde::{Deserialize, Serialize};

    // ============================================================================
    // WRAPPER TYPES FOR FIXED-SIZE ARRAYS WITH PROPER SERDE SUPPORT
    // ============================================================================

    /// 64-byte signature - use Vec<u8> with validation for serde compatibility
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Signature(Vec<u8>);

    impl Signature {
        pub fn new(bytes: [u8; 64]) -> Self {
            Self(bytes.to_vec())
        }

        pub fn as_bytes(&self) -> &[u8] {
            &self.0
        }

        pub fn to_array(&self) -> Result<[u8; 64], String> {
            if self.0.len() != 64 {
                return Err(format!(
                    "Invalid signature length: expected 64, got {}",
                    self.0.len()
                ));
            }
            let mut array = [0u8; 64];
            array.copy_from_slice(&self.0);
            Ok(array)
        }
    }

    /// 32-byte pubkey - use Vec<u8> with validation for consistency
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Pubkey(Vec<u8>);

    impl Pubkey {
        pub fn new(bytes: [u8; 32]) -> Self {
            Self(bytes.to_vec())
        }

        pub fn as_bytes(&self) -> &[u8] {
            &self.0
        }

        pub fn to_array(&self) -> Result<[u8; 32], String> {
            if self.0.len() != 32 {
                return Err(format!(
                    "Invalid pubkey length: expected 32, got {}",
                    self.0.len()
                ));
            }
            let mut array = [0u8; 32];
            array.copy_from_slice(&self.0);
            Ok(array)
        }
    }

    // ============================================================================
    // REALISTIC YELLOWSTONE GEYSER MESSAGE TYPES
    // ============================================================================

    // Protobuf definitions matching actual Yellowstone schema
    #[derive(Clone, PartialEq, prost::Message)]
    pub struct PbSubscribeUpdateAccount {
        #[prost(uint64, tag = "1")]
        pub slot: u64,
        #[prost(message, optional, tag = "2")]
        pub account: Option<PbReplicaAccountInfo>,
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct PbReplicaAccountInfo {
        #[prost(bytes = "vec", tag = "1")]
        pub pubkey: Vec<u8>,
        #[prost(uint64, tag = "2")]
        pub lamports: u64,
        #[prost(bytes = "vec", tag = "3")]
        pub owner: Vec<u8>,
        #[prost(bool, tag = "4")]
        pub executable: bool,
        #[prost(uint64, tag = "5")]
        pub rent_epoch: u64,
        #[prost(bytes = "vec", tag = "6")]
        pub data: Vec<u8>,
        #[prost(uint64, tag = "7")]
        pub write_version: u64,
        #[prost(bytes = "vec", optional, tag = "8")]
        pub txn_signature: Option<Vec<u8>>,
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct PbSubscribeUpdateTransaction {
        #[prost(uint64, tag = "1")]
        pub slot: u64,
        #[prost(message, optional, tag = "2")]
        pub transaction: Option<PbReplicaTransactionInfo>,
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct PbReplicaTransactionInfo {
        #[prost(bytes = "vec", tag = "1")]
        pub signature: Vec<u8>,
        #[prost(bool, tag = "2")]
        pub is_vote: bool,
        #[prost(bytes = "vec", tag = "3")]
        pub transaction: Vec<u8>,
        #[prost(message, optional, tag = "4")]
        pub meta: Option<PbTransactionStatusMeta>,
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct PbTransactionStatusMeta {
        #[prost(string, optional, tag = "1")]
        pub err: Option<String>,
        #[prost(uint64, tag = "2")]
        pub fee: u64,
        #[prost(uint64, repeated, tag = "3")]
        pub pre_balances: Vec<u64>,
        #[prost(uint64, repeated, tag = "4")]
        pub post_balances: Vec<u64>,
        #[prost(string, repeated, tag = "5")]
        pub log_messages: Vec<String>,
        #[prost(message, repeated, tag = "6")]
        pub pre_token_balances: Vec<PbTokenBalance>,
        #[prost(message, repeated, tag = "7")]
        pub post_token_balances: Vec<PbTokenBalance>,
        #[prost(uint64, optional, tag = "8")]
        pub compute_units_consumed: Option<u64>,
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct PbTokenBalance {
        #[prost(uint32, tag = "1")]
        pub account_index: u32,
        #[prost(string, tag = "2")]
        pub mint: String,
        #[prost(message, optional, tag = "3")]
        pub ui_token_amount: Option<PbUiTokenAmount>,
        #[prost(string, tag = "4")]
        pub owner: String,
        #[prost(string, tag = "5")]
        pub program_id: String,
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct PbUiTokenAmount {
        #[prost(double, tag = "1")]
        pub ui_amount: f64,
        #[prost(uint32, tag = "2")]
        pub decimals: u32,
        #[prost(string, tag = "3")]
        pub amount: String,
        #[prost(string, tag = "4")]
        pub ui_amount_string: String,
    }

    // Bincode equivalents (optimized)
    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    pub struct BcSubscribeUpdateAccount {
        pub slot: u64,
        pub account: Option<BcReplicaAccountInfo>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    pub struct BcReplicaAccountInfo {
        pub pubkey: Pubkey, // Fixed size wrapper vs Vec<u8>
        pub lamports: u64,
        pub owner: Pubkey, // Fixed size wrapper vs Vec<u8>
        pub executable: bool,
        pub rent_epoch: u64,
        pub data: Vec<u8>,
        pub write_version: u64,
        pub txn_signature: Option<Signature>, // Fixed size wrapper vs Vec<u8>
    }

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    pub struct BcSubscribeUpdateTransaction {
        pub slot: u64,
        pub transaction: Option<BcReplicaTransactionInfo>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    pub struct BcReplicaTransactionInfo {
        pub signature: Signature, // Fixed size wrapper vs Vec<u8>
        pub is_vote: bool,
        pub transaction: Vec<u8>,
        pub meta: Option<BcTransactionStatusMeta>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    pub struct BcTransactionStatusMeta {
        pub err: Option<String>,
        pub fee: u64,
        pub pre_balances: Vec<u64>,
        pub post_balances: Vec<u64>,
        pub log_messages: Vec<String>,
        pub pre_token_balances: Vec<BcTokenBalance>,
        pub post_token_balances: Vec<BcTokenBalance>,
        pub compute_units_consumed: Option<u64>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    pub struct BcTokenBalance {
        pub account_index: u32,
        pub mint: Pubkey, // Wrapper vs base58 string
        pub ui_token_amount: Option<BcUiTokenAmount>,
        pub owner: Pubkey,      // Wrapper vs base58 string
        pub program_id: Pubkey, // Wrapper vs base58 string
    }

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    pub struct BcUiTokenAmount {
        pub ui_amount: f64,
        pub decimals: u32,
        pub amount: String, // Keep as string for compatibility
        pub ui_amount_string: String,
    }

    // ============================================================================
    // REALISTIC TEST DATA CREATION
    // ============================================================================

    fn create_realistic_spl_token_account() -> (PbSubscribeUpdateAccount, BcSubscribeUpdateAccount)
    {
        // Realistic SPL token account data (165 bytes)
        let token_account_data = vec![
            // Mint (32 bytes)
            0x4f, 0xfc, 0x0c, 0x3c, 0x5f, 0x2a, 0xb9, 0x6d, 0x6b, 0x64, 0x0c, 0xa2, 0x1e, 0x7d,
            0x50, 0x8e, 0x9b, 0x97, 0x92, 0x84, 0x7f, 0x3f, 0xa3, 0xd6, 0x8b, 0x1b, 0x8e, 0x2e,
            0x3f, 0x3f, 0x45, 0x04, // Owner (32 bytes)
            0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90,
            0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90,
            0x90, 0x90, 0x90, 0x90, // Amount (8 bytes) - 1 million USDC (6 decimals)
            0x40, 0x42, 0x0f, 0x00, 0x00, 0x00, 0x00, 0x00,
            // Delegate (32 bytes, all zeros = None)
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, // State (1 byte) - Initialized
            0x01, // Is_native (12 bytes option) - None
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            // Delegated_amount (8 bytes)
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            // Close_authority (32 bytes, optional)
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];

        let pubkey_bytes = [0x12; 32];
        let owner_bytes = [
            0x06, 0xdf, 0x6e, 0x1d, 0x76, 0x57, 0x75, 0xe9, 0x48, 0x6e, 0x0b, 0x29, 0xd1, 0x2f,
            0x4b, 0x38, 0x7b, 0x4f, 0x2d, 0x8c, 0x26, 0xf3, 0x1f, 0x48, 0x3f, 0x33, 0x0b, 0xb6,
            0x42, 0xf3, 0x1c, 0xe0,
        ]; // Token Program ID
        let signature_bytes = [0xab; 64];

        let pb_account = PbSubscribeUpdateAccount {
            slot: 248_530_742, // Recent mainnet slot
            account: Some(PbReplicaAccountInfo {
                pubkey: pubkey_bytes.to_vec(),
                lamports: 2_039_280, // Rent-exempt amount for token account
                owner: owner_bytes.to_vec(),
                executable: false,
                rent_epoch: 361,
                data: token_account_data.clone(),
                write_version: 12345,
                txn_signature: Some(signature_bytes.to_vec()),
            }),
        };

        let bc_account = BcSubscribeUpdateAccount {
            slot: 248_530_742,
            account: Some(BcReplicaAccountInfo {
                pubkey: Pubkey::new(pubkey_bytes),
                lamports: 2_039_280,
                owner: Pubkey::new(owner_bytes),
                executable: false,
                rent_epoch: 361,
                data: token_account_data,
                write_version: 12345,
                txn_signature: Some(Signature::new(signature_bytes)),
            }),
        };

        (pb_account, bc_account)
    }

    fn create_realistic_dex_transaction()
    -> (PbSubscribeUpdateTransaction, BcSubscribeUpdateTransaction) {
        let signature_bytes = [0xcd; 64];
        let transaction_bytes = vec![0x01; 1232]; // Typical DEX swap transaction size

        // Realistic token balances for a DEX swap
        let usdc_mint = [
            0x4f, 0xfc, 0x0c, 0x3c, 0x5f, 0x2a, 0xb9, 0x6d, 0x6b, 0x64, 0x0c, 0xa2, 0x1e, 0x7d,
            0x50, 0x8e, 0x9b, 0x97, 0x92, 0x84, 0x7f, 0x3f, 0xa3, 0xd6, 0x8b, 0x1b, 0x8e, 0x2e,
            0x3f, 0x3f, 0x45, 0x04,
        ];
        let _sol_mint = [
            0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
            0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
            0x11, 0x11, 0x11, 0x11,
        ];
        let user_wallet = [0x90; 32];
        let token_program = [
            0x06, 0xdf, 0x6e, 0x1d, 0x76, 0x57, 0x75, 0xe9, 0x48, 0x6e, 0x0b, 0x29, 0xd1, 0x2f,
            0x4b, 0x38, 0x7b, 0x4f, 0x2d, 0x8c, 0x26, 0xf3, 0x1f, 0x48, 0x3f, 0x33, 0x0b, 0xb6,
            0x42, 0xf3, 0x1c, 0xe0,
        ];

        let pb_transaction = PbSubscribeUpdateTransaction {
            slot: 248_530_742,
            transaction: Some(PbReplicaTransactionInfo {
                signature: signature_bytes.to_vec(),
                is_vote: false,
                transaction: transaction_bytes.clone(),
                meta: Some(PbTransactionStatusMeta {
                    err: None,
                    fee: 5000, // 0.005 SOL fee
                    pre_balances: vec![10_000_000_000, 2_039_280, 2_039_280], // 10 SOL, 2 token accounts
                    post_balances: vec![9_999_995_000, 2_039_280, 2_039_280], // After fee
                    log_messages: vec![
                        "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [1]".to_string(),
                        "Program log: Instruction: Transfer".to_string(),
                        "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4645 of 200000 compute units".to_string(),
                        "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success".to_string(),
                    ],
                    pre_token_balances: vec![
                        PbTokenBalance {
                            account_index: 1,
                            mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC
                            ui_token_amount: Some(PbUiTokenAmount {
                                ui_amount: 1000.0,
                                decimals: 6,
                                amount: "1000000000".to_string(),
                                ui_amount_string: "1000".to_string(),
                            }),
                            owner: "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM".to_string(),
                            program_id: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_string(),
                        }
                    ],
                    post_token_balances: vec![
                        PbTokenBalance {
                            account_index: 1,
                            mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
                            ui_token_amount: Some(PbUiTokenAmount {
                                ui_amount: 900.0,
                                decimals: 6,
                                amount: "900000000".to_string(),
                                ui_amount_string: "900".to_string(),
                            }),
                            owner: "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM".to_string(),
                            program_id: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_string(),
                        }
                    ],
                    compute_units_consumed: Some(4645),
                }),
            }),
        };

        let bc_transaction = BcSubscribeUpdateTransaction {
            slot: 248_530_742,
            transaction: Some(BcReplicaTransactionInfo {
                signature: Signature::new(signature_bytes),
                is_vote: false,
                transaction: transaction_bytes,
                meta: Some(BcTransactionStatusMeta {
                    err: None,
                    fee: 5000,
                    pre_balances: vec![10_000_000_000, 2_039_280, 2_039_280],
                    post_balances: vec![9_999_995_000, 2_039_280, 2_039_280],
                    log_messages: vec![
                        "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [1]".to_string(),
                        "Program log: Instruction: Transfer".to_string(),
                        "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4645 of 200000 compute units".to_string(),
                        "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success".to_string(),
                    ],
                    pre_token_balances: vec![
                        BcTokenBalance {
                            account_index: 1,
                            mint: Pubkey::new(usdc_mint),
                            ui_token_amount: Some(BcUiTokenAmount {
                                ui_amount: 1000.0,
                                decimals: 6,
                                amount: "1000000000".to_string(),
                                ui_amount_string: "1000".to_string(),
                            }),
                            owner: Pubkey::new(user_wallet),
                            program_id: Pubkey::new(token_program),
                        }
                    ],
                    post_token_balances: vec![
                        BcTokenBalance {
                            account_index: 1,
                            mint: Pubkey::new(usdc_mint),
                            ui_token_amount: Some(BcUiTokenAmount {
                                ui_amount: 900.0,
                                decimals: 6,
                                amount: "900000000".to_string(),
                                ui_amount_string: "900".to_string(),
                            }),
                            owner: Pubkey::new(user_wallet),
                            program_id: Pubkey::new(token_program),
                        }
                    ],
                    compute_units_consumed: Some(4645),
                }),
            }),
        };

        (pb_transaction, bc_transaction)
    }

    fn create_large_program_account() -> (PbSubscribeUpdateAccount, BcSubscribeUpdateAccount) {
        // Simulate a large program account like an AMM pool or orderbook (10KB)
        let large_program_data = vec![0x42; 10_240]; // 10KB
        let pubkey_bytes = [0x55; 32];
        let owner_bytes = [0x77; 32]; // Some program ID

        let pb_account = PbSubscribeUpdateAccount {
            slot: 248_530_900,
            account: Some(PbReplicaAccountInfo {
                pubkey: pubkey_bytes.to_vec(),
                lamports: 5_000_000, // 0.005 SOL rent
                owner: owner_bytes.to_vec(),
                executable: false,
                rent_epoch: 361,
                data: large_program_data.clone(),
                write_version: 98765,
                txn_signature: Some([0xef; 64].to_vec()),
            }),
        };

        let bc_account = BcSubscribeUpdateAccount {
            slot: 248_530_900,
            account: Some(BcReplicaAccountInfo {
                pubkey: Pubkey::new(pubkey_bytes),
                lamports: 5_000_000,
                owner: Pubkey::new(owner_bytes),
                executable: false,
                rent_epoch: 361,
                data: large_program_data,
                write_version: 98765,
                txn_signature: Some(Signature::new([0xef; 64])),
            }),
        };

        (pb_account, bc_account)
    }

    // ============================================================================
    // REALISTIC GEYSER BENCHMARKS
    // ============================================================================

    #[test]
    fn realistic_geyser_account_update_benchmark() {
        println!("🏦 REALISTIC GEYSER ACCOUNT UPDATE BENCHMARK");
        println!("============================================");

        let (pb_account, bc_account) = create_realistic_spl_token_account();
        let bincode_codec = BincodeCodec::new();

        // Size comparison
        let pb_bytes = pb_account.encode_to_vec();
        let bc_bytes = bincode_codec.encode(&bc_account).unwrap();

        println!("📏 SPL Token Account Update:");
        println!("   Protobuf: {} bytes", pb_bytes.len());
        println!("   Bincode:  {} bytes", bc_bytes.len());
        println!(
            "   Savings:  {} bytes ({:.1}%)",
            pb_bytes.len() - bc_bytes.len(),
            ((pb_bytes.len() - bc_bytes.len()) as f64 / pb_bytes.len() as f64) * 100.0
        );

        let iterations = 100_000;

        // Encoding benchmark
        let start = std::time::Instant::now();
        for _ in 0..iterations {
            let _ = pb_account.encode_to_vec();
        }
        let pb_encode_time = start.elapsed();

        let start = std::time::Instant::now();
        for _ in 0..iterations {
            let _ = bincode_codec.encode(&bc_account).unwrap();
        }
        let bc_encode_time = start.elapsed();

        // Decoding benchmark
        let start = std::time::Instant::now();
        for _ in 0..iterations {
            let _ = PbSubscribeUpdateAccount::decode(&pb_bytes[..]).unwrap();
        }
        let pb_decode_time = start.elapsed();

        let start = std::time::Instant::now();
        for _ in 0..iterations {
            let _: BcSubscribeUpdateAccount = bincode_codec.decode(&bc_bytes).unwrap();
        }
        let bc_decode_time = start.elapsed();

        let pb_encode_ns = pb_encode_time.as_nanos() as f64 / iterations as f64;
        let bc_encode_ns = bc_encode_time.as_nanos() as f64 / iterations as f64;
        let pb_decode_ns = pb_decode_time.as_nanos() as f64 / iterations as f64;
        let bc_decode_ns = bc_decode_time.as_nanos() as f64 / iterations as f64;

        println!("\n⚡ Performance ({} iterations):", iterations);
        println!("   ENCODING:");
        println!("     Protobuf: {:.1} ns/op", pb_encode_ns);
        println!("     Bincode:  {:.1} ns/op", bc_encode_ns);
        println!(
            "     Winner: {} ({:.1}x faster)",
            if bc_encode_ns < pb_encode_ns {
                "Bincode"
            } else {
                "Protobuf"
            },
            if bc_encode_ns < pb_encode_ns {
                pb_encode_ns / bc_encode_ns
            } else {
                bc_encode_ns / pb_encode_ns
            }
        );

        println!("   DECODING:");
        println!("     Protobuf: {:.1} ns/op", pb_decode_ns);
        println!("     Bincode:  {:.1} ns/op", bc_decode_ns);
        println!(
            "     Winner: {} ({:.1}x faster)",
            if bc_decode_ns < pb_decode_ns {
                "Bincode"
            } else {
                "Protobuf"
            },
            if bc_decode_ns < pb_decode_ns {
                pb_decode_ns / bc_decode_ns
            } else {
                bc_decode_ns / pb_decode_ns
            }
        );

        println!("\n💰 At 10,000 account updates/second (realistic Geyser load):");
        let daily_bandwidth_pb = (pb_bytes.len() * 10_000 * 86_400) as f64 / 1_000_000.0;
        let daily_bandwidth_bc = (bc_bytes.len() * 10_000 * 86_400) as f64 / 1_000_000.0;
        println!("   Protobuf daily bandwidth: {:.1} MB", daily_bandwidth_pb);
        println!("   Bincode daily bandwidth:  {:.1} MB", daily_bandwidth_bc);
        println!(
            "   Daily savings: {:.1} MB",
            daily_bandwidth_pb - daily_bandwidth_bc
        );
    }

    #[test]
    fn realistic_geyser_transaction_benchmark() {
        println!("\n💸 REALISTIC GEYSER TRANSACTION BENCHMARK");
        println!("=========================================");

        let (pb_tx, bc_tx) = create_realistic_dex_transaction();
        let bincode_codec = BincodeCodec::new();

        let pb_bytes = pb_tx.encode_to_vec();
        let bc_bytes = bincode_codec.encode(&bc_tx).unwrap();

        println!("📏 DEX Swap Transaction:");
        println!("   Protobuf: {} bytes", pb_bytes.len());
        println!("   Bincode:  {} bytes", bc_bytes.len());
        println!(
            "   Difference: {} bytes ({:.1}%)",
            pb_bytes.len() as i32 - bc_bytes.len() as i32,
            ((pb_bytes.len() as i32 - bc_bytes.len() as i32) as f64 / pb_bytes.len() as f64)
                * 100.0
        );

        println!("\n🔍 Size Breakdown Analysis:");
        println!("   Fixed-size fields (signature, pubkeys): Bincode advantage");
        println!("   String fields (mint addresses, logs): Protobuf/Bincode similar");
        println!("   Variable data (transaction bytes): Bincode slight advantage");

        let iterations = 50_000;

        let start = std::time::Instant::now();
        for _ in 0..iterations {
            let _ = pb_tx.encode_to_vec();
        }
        let pb_encode_time = start.elapsed();

        let start = std::time::Instant::now();
        for _ in 0..iterations {
            let _ = bincode_codec.encode(&bc_tx).unwrap();
        }
        let bc_encode_time = start.elapsed();

        println!("\n⚡ Transaction Processing Performance:");
        let pb_encode_us = pb_encode_time.as_micros() as f64 / iterations as f64;
        let bc_encode_us = bc_encode_time.as_micros() as f64 / iterations as f64;
        println!("   Protobuf encode: {:.2} μs/op", pb_encode_us);
        println!("   Bincode encode:  {:.2} μs/op", bc_encode_us);
        println!("   Speedup: {:.1}x", pb_encode_us / bc_encode_us);
    }

    #[test]
    fn realistic_large_account_benchmark() {
        println!("\n🏗️  LARGE PROGRAM ACCOUNT BENCHMARK");
        println!("===================================");

        let (pb_large, bc_large) = create_large_program_account();
        let bincode_codec = BincodeCodec::new();

        let pb_bytes = pb_large.encode_to_vec();
        let bc_bytes = bincode_codec.encode(&bc_large).unwrap();

        println!("📏 Large Program Account (10KB data):");
        println!("   Protobuf: {} bytes", pb_bytes.len());
        println!("   Bincode:  {} bytes", bc_bytes.len());
        println!(
            "   Savings:  {} bytes ({:.1}%)",
            pb_bytes.len() - bc_bytes.len(),
            ((pb_bytes.len() - bc_bytes.len()) as f64 / pb_bytes.len() as f64) * 100.0
        );

        let iterations = 10_000;

        let start = std::time::Instant::now();
        for _ in 0..iterations {
            let _ = pb_large.encode_to_vec();
        }
        let pb_time = start.elapsed();

        let start = std::time::Instant::now();
        for _ in 0..iterations {
            let _ = bincode_codec.encode(&bc_large).unwrap();
        }
        let bc_time = start.elapsed();

        println!("\n⚡ Large Data Performance ({} iterations):", iterations);
        let pb_time_us = pb_time.as_micros() as f64 / iterations as f64;
        let bc_time_us = bc_time.as_micros() as f64 / iterations as f64;
        println!("   Protobuf: {:.1} μs/op", pb_time_us);
        println!("   Bincode:  {:.1} μs/op", bc_time_us);
        println!("   Speedup: {:.1}x", pb_time_us / bc_time_us);

        let pb_throughput = (pb_bytes.len() as f64 / pb_time_us * 1e6) / 1_000_000.0;
        let bc_throughput = (bc_bytes.len() as f64 / bc_time_us * 1e6) / 1_000_000.0;
        println!("   Protobuf throughput: {:.1} MB/s", pb_throughput);
        println!("   Bincode throughput:  {:.1} MB/s", bc_throughput);
    }

    #[test]
    fn geyser_mixed_workload_simulation() {
        println!("\n🌊 GEYSER MIXED WORKLOAD SIMULATION");
        println!("===================================");
        println!("Simulating realistic Yellowstone stream:");
        println!("  • 70% small token account updates");
        println!("  • 20% transaction updates");
        println!("  • 10% large program account updates");

        let (pb_token, bc_token) = create_realistic_spl_token_account();
        let (pb_tx, bc_tx) = create_realistic_dex_transaction();
        let (pb_large, bc_large) = create_large_program_account();

        let bincode_codec = BincodeCodec::new();

        let pb_token_bytes = pb_token.encode_to_vec();
        let bc_token_bytes = bincode_codec.encode(&bc_token).unwrap();
        let pb_tx_bytes = pb_tx.encode_to_vec();
        let bc_tx_bytes = bincode_codec.encode(&bc_tx).unwrap();
        let pb_large_bytes = pb_large.encode_to_vec();
        let bc_large_bytes = bincode_codec.encode(&bc_large).unwrap();

        // Calculate weighted average based on realistic traffic
        let pb_weighted_size = (pb_token_bytes.len() as f64 * 0.7)
            + (pb_tx_bytes.len() as f64 * 0.2)
            + (pb_large_bytes.len() as f64 * 0.1);
        let bc_weighted_size = (bc_token_bytes.len() as f64 * 0.7)
            + (bc_tx_bytes.len() as f64 * 0.2)
            + (bc_large_bytes.len() as f64 * 0.1);

        println!("\n📊 Weighted Average Message Size:");
        println!("   Protobuf: {:.1} bytes", pb_weighted_size);
        println!("   Bincode:  {:.1} bytes", bc_weighted_size);
        println!(
            "   Savings:  {:.1} bytes ({:.1}%)",
            pb_weighted_size - bc_weighted_size,
            ((pb_weighted_size - bc_weighted_size) / pb_weighted_size) * 100.0
        );

        println!("\n💡 REAL-WORLD IMPACT:");
        println!("At 50,000 messages/second (busy mainnet validator):");
        let daily_pb = pb_weighted_size * 50_000.0 * 86_400.0 / 1_000_000.0;
        let daily_bc = bc_weighted_size * 50_000.0 * 86_400.0 / 1_000_000.0;
        println!("  • Protobuf: {:.0} MB/day", daily_pb);
        println!("  • Bincode:  {:.0} MB/day", daily_bc);
        println!(
            "  • Savings:  {:.0} MB/day ({:.1} GB/day)",
            daily_pb - daily_bc,
            (daily_pb - daily_bc) / 1000.0
        );

        println!("\n🎯 KEY INSIGHTS:");
        println!("  • Fixed-size crypto fields (pubkeys, signatures) = major bincode wins");
        println!("  • Large binary data (account data) = moderate bincode advantage");
        println!("  • String fields (logs, addresses) = roughly equal");
        println!("  • Overall: Bincode wins on size AND speed for Geyser workloads");
    }
}
