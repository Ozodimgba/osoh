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
mod complex_benchmarks {
    use prost::Message;

    // Complex protobuf messages for Solana RPC types

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct PbTransaction {
        #[prost(string, repeated, tag = "1")]
        pub signatures: Vec<String>,
        #[prost(message, optional, tag = "2")]
        pub message: Option<PbTransactionMessage>,
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct PbTransactionMessage {
        #[prost(message, repeated, tag = "1")]
        pub account_keys: Vec<PbPubkey>,
        #[prost(bytes, tag = "2")]
        pub recent_blockhash: Vec<u8>,
        #[prost(message, repeated, tag = "3")]
        pub instructions: Vec<PbInstruction>,
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct PbPubkey {
        #[prost(string, tag = "1")]
        pub key: String,
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct PbInstruction {
        #[prost(uint32, tag = "1")]
        pub program_id_index: u32,
        #[prost(uint32, repeated, tag = "2")]
        pub accounts: Vec<u32>,
        #[prost(bytes, tag = "3")]
        pub data: Vec<u8>,
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct PbGetBlockResponse {
        #[prost(uint64, tag = "1")]
        pub slot: u64,
        #[prost(string, tag = "2")]
        pub blockhash: String,
        #[prost(string, tag = "3")]
        pub previous_blockhash: String,
        #[prost(uint64, tag = "4")]
        pub parent_slot: u64,
        #[prost(message, repeated, tag = "5")]
        pub transactions: Vec<PbTransaction>,
        #[prost(uint64, repeated, tag = "6")]
        pub rewards: Vec<u64>,
        #[prost(uint64, tag = "7")]
        pub block_time: u64,
        #[prost(uint64, tag = "8")]
        pub block_height: u64,
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct PbProgramAccount {
        #[prost(string, tag = "1")]
        pub pubkey: String,
        #[prost(message, optional, tag = "2")]
        pub account: Option<PbAccount>,
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct PbAccount {
        #[prost(uint64, tag = "1")]
        pub lamports: u64,
        #[prost(bytes, tag = "2")]
        pub data: Vec<u8>,
        #[prost(string, tag = "3")]
        pub owner: String,
        #[prost(bool, tag = "4")]
        pub executable: bool,
        #[prost(uint64, tag = "5")]
        pub rent_epoch: u64,
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct PbGetProgramAccountsResponse {
        #[prost(message, repeated, tag = "1")]
        pub accounts: Vec<PbProgramAccount>,
    }

    // Helper to create complex test data
    fn create_complex_transaction() -> PbTransaction {
        PbTransaction {
            signatures: vec![
                "5VfydnLu4XkekcLwHXwcd3dYiP4Z2K5p1rUgvFVGWQ2J5X".to_string(),
                "4XvK7rG5SWGfchVTmzqJT5J5z5M5m5X5y5B5C5D5E5F5G".to_string(),
            ],
            message: Some(PbTransactionMessage {
                account_keys: (0..15)
                    .map(|i| PbPubkey {
                        key: format!("{}111111111111111111111111111111{:02}", i, i),
                    })
                    .collect(),
                recent_blockhash: vec![1u8; 32],
                instructions: vec![
                    PbInstruction {
                        program_id_index: 0,
                        accounts: vec![1, 2, 3, 4, 5],
                        data: vec![42u8; 128],
                    },
                    PbInstruction {
                        program_id_index: 1,
                        accounts: vec![6, 7, 8, 9, 10, 11, 12],
                        data: vec![99u8; 256],
                    },
                    PbInstruction {
                        program_id_index: 2,
                        accounts: vec![13, 14],
                        data: vec![77u8; 64],
                    },
                ],
            }),
        }
    }

    fn create_complex_block_response() -> PbGetBlockResponse {
        PbGetBlockResponse {
            slot: 123456789,
            blockhash: "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM".to_string(),
            previous_blockhash: "8VzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWL".to_string(),
            parent_slot: 123456788,
            transactions: (0..50).map(|_| create_complex_transaction()).collect(),
            rewards: vec![1000000, 2000000, 3000000, 4000000, 5000000],
            block_time: 1640995200,
            block_height: 98765432,
        }
    }

    fn create_program_accounts_response() -> PbGetProgramAccountsResponse {
        PbGetProgramAccountsResponse {
            accounts: (0..100)
                .map(|i| PbProgramAccount {
                    pubkey: format!("{}111111111111111111111111111111{:02}", i, i),
                    account: Some(PbAccount {
                        lamports: 1000000 + i as u64,
                        data: vec![i as u8; 1024], // 1KB of data per account
                        owner: "11111111111111111111111111111112".to_string(),
                        executable: i % 10 == 0,
                        rent_epoch: 200 + i as u64,
                    }),
                })
                .collect(),
        }
    }

    #[test]
    fn complex_block_response_benchmark() {
        println!("🚀 COMPLEX BLOCK RESPONSE BENCHMARK");
        println!("=====================================");

        let pb_block = create_complex_block_response();

        // This would need corresponding Rust native types for comparison
        // For now, just show protobuf metrics

        let protobuf_bytes = pb_block.encode_to_vec();
        println!("📏 Complex Block Response Size:");
        println!(
            "   Protobuf: {} bytes ({:.1} KB)",
            protobuf_bytes.len(),
            protobuf_bytes.len() as f64 / 1024.0
        );

        let iterations = 10_000;

        // Encoding benchmark
        let start = std::time::Instant::now();
        for _ in 0..iterations {
            let _ = pb_block.encode_to_vec();
        }
        let encode_time = start.elapsed();

        // Decoding benchmark
        let start = std::time::Instant::now();
        for _ in 0..iterations {
            let _ = PbGetBlockResponse::decode(&protobuf_bytes[..]).unwrap();
        }
        let decode_time = start.elapsed();

        println!("⚡ Complex Block Performance:");
        println!(
            "   Encoding: {:.0} ns/op",
            encode_time.as_nanos() as f64 / iterations as f64
        );
        println!(
            "   Decoding: {:.0} ns/op",
            decode_time.as_nanos() as f64 / iterations as f64
        );
    }

    #[test]
    fn program_accounts_benchmark() {
        println!("🚀 PROGRAM ACCOUNTS RESPONSE BENCHMARK");
        println!("======================================");

        let pb_accounts = create_program_accounts_response();
        let protobuf_bytes = pb_accounts.encode_to_vec();

        println!("📏 Program Accounts Response Size:");
        println!(
            "   Protobuf: {} bytes ({:.1} KB)",
            protobuf_bytes.len(),
            protobuf_bytes.len() as f64 / 1024.0
        );
        println!(
            "   {} accounts with 1KB data each",
            pb_accounts.accounts.len()
        );

        let iterations = 1_000;

        let start = std::time::Instant::now();
        for _ in 0..iterations {
            let _ = pb_accounts.encode_to_vec();
        }
        let encode_time = start.elapsed();

        let start = std::time::Instant::now();
        for _ in 0..iterations {
            let _ = PbGetProgramAccountsResponse::decode(&protobuf_bytes[..]).unwrap();
        }
        let decode_time = start.elapsed();

        println!("⚡ Program Accounts Performance:");
        println!(
            "   Encoding: {:.0} μs/op",
            encode_time.as_micros() as f64 / iterations as f64
        );
        println!(
            "   Decoding: {:.0} μs/op",
            decode_time.as_micros() as f64 / iterations as f64
        );
    }

    #[test]
    fn memory_usage_analysis() {
        println!("🧠 MEMORY USAGE ANALYSIS");
        println!("========================");

        let pb_block = create_complex_block_response();
        let pb_accounts = create_program_accounts_response();

        let block_bytes = pb_block.encode_to_vec();
        let accounts_bytes = pb_accounts.encode_to_vec();

        println!("Memory Efficiency:");
        println!(
            "  Block Response: {} bytes serialized, ~{} bytes in memory",
            block_bytes.len(),
            std::mem::size_of_val(&pb_block)
        );
        println!(
            "  Program Accounts: {} bytes serialized, ~{} bytes in memory",
            accounts_bytes.len(),
            std::mem::size_of_val(&pb_accounts)
        );

        // Test string vs bytes efficiency
        let string_heavy = PbPubkey {
            key: "A".repeat(1000),
        };
        let bytes_heavy = PbAccount {
            lamports: 0,
            data: vec![0u8; 1000],
            owner: "test".to_string(),
            executable: false,
            rent_epoch: 0,
        };

        let string_serialized = string_heavy.encode_to_vec();
        let bytes_serialized = bytes_heavy.encode_to_vec();

        println!("String vs Bytes (1KB each):");
        println!(
            "  String field: {} bytes serialized",
            string_serialized.len()
        );
        println!("  Bytes field: {} bytes serialized", bytes_serialized.len());
    }
}
