//! Transport layer demonstration using in-memory channels
//!
//! This example demonstrates the RPC transport functionality using
//! tokio channels to simulate network communication between client and server.
//! It shows real message serialization, protocol handling, and RPC patterns.

use rpc_codec::protocol::{MessageType, RpcMessage, constants::method_ids};

use rpc_transport::{
    QuicConfig,
    protocol::{MessagePriority, StreamedMessage},
};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::mpsc;

// Test message types for demonstration
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
struct GetAccountRequest {
    pub_key: String,
    encoding: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct GetAccountResponse {
    account: AccountInfo,
    slot: u64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct AccountInfo {
    lamports: u64,
    owner: String,
    executable: bool,
    data: Vec<u8>,
}

// Mock transport connection using channels
struct MockTransport {
    writer: mpsc::UnboundedSender<Vec<u8>>,
    reader: mpsc::UnboundedReceiver<Vec<u8>>,
    config: QuicConfig,
}

impl MockTransport {
    fn new_pair() -> (Self, Self) {
        let (tx1, rx1) = mpsc::unbounded_channel();
        let (tx2, rx2) = mpsc::unbounded_channel();

        let client = Self {
            writer: tx1,
            reader: rx2,
            config: QuicConfig::default(),
        };

        let server = Self {
            writer: tx2,
            reader: rx1,
            config: QuicConfig::default(),
        };

        (client, server)
    }

    async fn send_message<T>(&mut self, message: StreamedMessage<T>) -> Result<(), String>
    where
        T: Serialize + Send,
    {
        message
            .validate(&self.config)
            .map_err(|e| format!("Message validation failed: {:?}", e))?;


        let data = bincode::serde::encode_to_vec(&message, bincode::config::standard())
            .map_err(|e| format!("Serialization failed: {}", e))?;

        // Send over channel
        self.writer
            .send(data)
            .map_err(|_| "Channel send failed".to_string())?;

        Ok(())
    }

    async fn recv_message<T>(&mut self) -> Result<StreamedMessage<T>, String>
    where
        T: for<'de> Deserialize<'de>,
    {
        let data = self.reader.recv().await.ok_or("Channel receive failed")?;

        // Deserialize message
        let (message, _): (StreamedMessage<T>, usize) =
            bincode::serde::decode_from_slice(&data, bincode::config::standard())
                .map_err(|e| format!("Deserialization failed: {}", e))?;

        Ok(message)
    }

    async fn send_request<T, R>(
        &mut self,
        request: StreamedMessage<T>,
    ) -> Result<StreamedMessage<R>, String>
    where
        T: Serialize + Send,
        R: for<'de> Deserialize<'de>,
    {
        // Send request
        self.send_message(request).await?;

        // Wait for response
        self.recv_message().await
    }
}

// Server-side RPC handler
async fn handle_get_account_request(
    request: &GetAccountRequest,
) -> Result<GetAccountResponse, String> {
    println!(
        "🖥️  Server: Processing GetAccount request for {}",
        request.pub_key
    );

    // Simulate account lookup
    let account_info = AccountInfo {
        lamports: 1000000000, // 1 SOL
        owner: "11111111111111111111111111111112".to_string(),
        executable: false,
        data: vec![0; 32], // Empty data
    };

    let response = GetAccountResponse {
        account: account_info,
        slot: 123456789,
    };

    println!(
        "✅ Server: Found account with {} lamports",
        response.account.lamports
    );
    Ok(response)
}

// Server main loop
async fn run_server(mut transport: MockTransport) -> Result<(), String> {
    println!("🚀 Server: Starting RPC server");

    loop {
        // Receive request
        let request: StreamedMessage<GetAccountRequest> =
            transport.recv_message().await.map_err(|e| e.to_string())?;

        println!(
            "📬 Server: Received request ID {}",
            request.message.request_id
        );

        // Validate request
        if request.message.method_id != method_ids::GET_ACCOUNT_INFO {
            eprintln!(
                "❌ Server: Invalid method ID: {}",
                request.message.method_id
            );
            continue;
        }

        // Process request
        match handle_get_account_request(request.get_stream_payload()).await {
            Ok(response_data) => {
                // Create response message
                let response = StreamedMessage::new(RpcMessage {
                    version: request.message.version,
                    request_id: request.message.request_id,
                    method_id: request.message.method_id,
                    message_type: MessageType::Response,
                    payload: response_data,
                })
                .with_priority(MessagePriority::High);

                // Send response
                if let Err(e) = transport.send_message(response).await {
                    eprintln!("❌ Server: Failed to send response: {}", e);
                } else {
                    println!(
                        "📤 Server: Sent response for request {}",
                        request.message.request_id
                    );
                }
            }
            Err(e) => {
                eprintln!("❌ Server: Request processing failed: {}", e);
            }
        }

        // For demo, handle just one request
        break;
    }

    Ok(())
}

// Client operations
async fn run_client(mut transport: MockTransport) -> Result<(), String> {
    println!("💻 Client: Starting RPC client");

    // Create account info request
    let request_data = GetAccountRequest {
        pub_key: "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM".to_string(),
        encoding: "base64".to_string(),
    };

    let request = StreamedMessage::new(RpcMessage {
        version: 1,
        request_id: 42,
        method_id: method_ids::GET_ACCOUNT_INFO,
        message_type: MessageType::Request,
        payload: request_data.clone(),
    })
    .with_priority(MessagePriority::Normal);

    println!(
        "📤 Client: Sending GetAccount request for {}",
        request_data.pub_key
    );

    // Send request and wait for response
    let response: StreamedMessage<GetAccountResponse> = transport
        .send_request(request)
        .await
        .map_err(|e| e.to_string())?;

    println!(
        "📬 Client: Received response with {} lamports",
        response.get_stream_payload().account.lamports
    );

    // Validate response
    assert_eq!(response.message.request_id, 42);
    assert_eq!(response.message.method_id, method_ids::GET_ACCOUNT_INFO);
    assert!(matches!(
        response.message.message_type,
        MessageType::Response
    ));
    assert_eq!(response.get_stream_payload().account.lamports, 1000000000);

    println!("✅ Client: Response validation passed");

    Ok(())
}

// Demonstrate configuration and validation
fn demo_transport_features() {
    println!("\n🔧 Transport Features Demo");
    println!("=========================");

    // Configuration presets
    let configs = [
        ("Default", QuicConfig::default()),
        ("High Throughput", QuicConfig::high_throughput()),
        ("Low Latency", QuicConfig::low_latency()),
        ("Minimal", QuicConfig::minimal()),
    ];

    for (name, config) in configs {
        println!(
            "📋 {}: {}MB messages, {} streams, {}s timeout",
            name,
            config.max_message_size / (1024 * 1024),
            config.max_concurrent_streams,
            config.max_idle_timeout.as_secs()
        );
    }

    // Message priorities
    let priorities = [
        MessagePriority::Low,
        MessagePriority::Normal,
        MessagePriority::High,
        MessagePriority::Critical,
    ];

    println!("\n🚦 Message Priorities:");
    for priority in priorities {
        let level = priority as u8;
        println!("   {:?}: Level {}", priority, level);
    }

    // Validation example
    println!("\n🔍 Message Validation:");
    let config = QuicConfig::new().with_message_size(1000); // Small limit

    let large_request = GetAccountRequest {
        pub_key: "x".repeat(500), // Large key
        encoding: "base64".to_string(),
    };

    let message = StreamedMessage::new(RpcMessage {
        version: 1,
        request_id: 1,
        method_id: method_ids::GET_ACCOUNT_INFO,
        message_type: MessageType::Request,
        payload: large_request,
    });

    match message.validate(&config) {
        Ok(()) => println!("   ✅ Message validation passed"),
        Err(e) => println!("   ❌ Message validation failed: {:?}", e),
    }
}

// Performance characteristics demo
fn demo_performance_benefits() {
    println!("\n⚡ Performance Benefits");
    println!("=====================");

    println!("🚀 QUIC Transport Advantages:");
    println!("   - Multiplexed streams (multiple RPC calls per connection)");
    println!("   - 0-RTT connection establishment");
    println!("   - Built-in encryption and authentication");
    println!("   - Better congestion control than TCP");
    println!("   - Reduced head-of-line blocking");

    println!("\n📊 Binary Protocol Benefits:");
    println!("   - Faster serialization/deserialization vs JSON");
    println!("   - Smaller message sizes");
    println!("   - Type-safe message handling");
    println!("   - Built-in validation");

    println!("\n🎯 Expected Performance Improvements:");
    println!("   - 10x+ throughput vs traditional JSON-RPC");
    println!("   - Lower latency through connection reuse");
    println!("   - Better resource utilization");
    println!("   - Improved error handling and recovery");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 RPC Transport Demonstration");
    println!("==============================");

    // 1. Feature demonstration
    demo_transport_features();

    // 2. Performance benefits
    demo_performance_benefits();

    // 3. Working transport test
    println!("\n🔗 Transport Communication Test");
    println!("==============================");

    // Create transport pair
    let (client_transport, server_transport) = MockTransport::new_pair();

    // Start server
    let server_handle = tokio::spawn(run_server(server_transport));

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(10)).await;

    // Run client
    let client_handle = tokio::spawn(run_client(client_transport));

    // Wait for both to complete
    let (server_result, client_result) = tokio::join!(server_handle, client_handle);

    match (server_result, client_result) {
        (Ok(Ok(())), Ok(Ok(()))) => {
            println!("\n🎉 TRANSPORT DEMONSTRATION SUCCESSFUL!");
            println!("✅ Successfully demonstrated:");
            println!("   - Message serialization with Bincode");
            println!("   - Request-response RPC pattern");
            println!("   - Protocol validation and error handling");
            println!("   - Configuration management");
            println!("   - Priority-based message handling");
            println!("   - Type-safe message processing");
        }
        (server_err, client_err) => {
            eprintln!("❌ Test failed:");
            if let Err(e) = server_err {
                eprintln!("   Server error: {:?}", e);
            } else if let Ok(Err(e)) = server_err {
                eprintln!("   Server error: {}", e);
            }
            if let Err(e) = client_err {
                eprintln!("   Client error: {:?}", e);
            } else if let Ok(Err(e)) = client_err {
                eprintln!("   Client error: {}", e);
            }
            return Err("Transport test failed".into());
        }
    }

    println!("\n🎯 This demonstrates the foundation for:");
    println!("   - High-performance Solana RPC operations");
    println!("   - Real-time account data streaming");
    println!("   - Efficient transaction processing");
    println!("   - Scalable blockchain interactions");

    println!("\n🚀 Next: Replace channels with actual QUIC networking!");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_transport_communication() {
        let (mut client, mut server) = MockTransport::new_pair();

        // Create test message
        let request = StreamedMessage::new(RpcMessage {
            version: 1,
            request_id: 123,
            method_id: method_ids::GET_BALANCE,
            message_type: MessageType::Request,
            payload: GetAccountRequest {
                pub_key: "test_key".to_string(),
                encoding: "base64".to_string(),
            },
        });

        // Send from client
        client.send_message(request.clone()).await.unwrap();

        // Receive on server
        let received: StreamedMessage<GetAccountRequest> = server.recv_message().await.unwrap();

        assert_eq!(received.message.request_id, request.message.request_id);
        assert_eq!(received.get_stream_payload().pub_key, "test_key");
    }

    #[tokio::test]
    async fn test_request_response_pattern() {
        let (client_transport, server_transport) = MockTransport::new_pair();

        let server_task = tokio::spawn(run_server(server_transport));
        let client_task = tokio::spawn(run_client(client_transport));

        let (server_result, client_result) = tokio::join!(server_task, client_task);

        assert!(server_result.is_ok());
        assert!(client_result.is_ok());
        assert!(server_result.unwrap().is_ok());
        assert!(client_result.unwrap().is_ok());
    }

    #[test]
    fn test_message_validation() {
        let config = QuicConfig::default();

        let valid_message = StreamedMessage::new(RpcMessage {
            version: 1,
            request_id: 1,
            method_id: method_ids::GET_ACCOUNT_INFO,
            message_type: MessageType::Request,
            payload: GetAccountRequest {
                pub_key: "valid_key".to_string(),
                encoding: "base64".to_string(),
            },
        });

        assert!(valid_message.validate(&config).is_ok());
    }

    #[test]
    fn test_serialization_roundtrip() {
        let original = GetAccountRequest {
            pub_key: "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM".to_string(),
            encoding: "base64".to_string(),
        };

        let serialized =
            bincode::serde::encode_to_vec(&original, bincode::config::standard()).unwrap();
        let (deserialized, _): (GetAccountRequest, usize) =
            bincode::serde::decode_from_slice(&serialized, bincode::config::standard()).unwrap();

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_config_presets() {
        let default = QuicConfig::default();
        let high_throughput = QuicConfig::high_throughput();
        let low_latency = QuicConfig::low_latency();

        assert!(high_throughput.max_message_size > default.max_message_size);
        assert!(low_latency.max_idle_timeout < default.max_idle_timeout);
    }
}
