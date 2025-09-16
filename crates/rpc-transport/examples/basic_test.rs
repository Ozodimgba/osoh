//! Basic QUIC transport connection test
//!
//! This is a minimal example showing the transport layer concepts
//! without the complexity of full server setup.

use rpc_codec::protocol::{MessageType, RpcMessage, constants::method_ids};
use rpc_transport::{
    QuicConfig,
    protocol::{MessagePriority, StreamedMessage},
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct TestMessage {
    content: String,
    number: u32,
}

/// Demonstrate basic transport concepts without actual networking
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Basic QUIC Transport Concepts Demo");
    println!("====================================");

    // 1. Configuration
    demo_configuration().await;

    // 2. Message Creation
    demo_message_creation().await;

    // 3. Stream Concepts
    demo_stream_concepts().await;

    println!("Basic concepts demonstration complete!");
    Ok(())
}

async fn demo_configuration() {
    println!("1. Configuration Demo");
    println!("-----------------------");

    // Different configuration presets
    let configs = vec![
        ("Default", QuicConfig::default()),
        ("High Throughput", QuicConfig::high_throughput()),
        ("Low Latency", QuicConfig::low_latency()),
        ("Minimal", QuicConfig::minimal()),
    ];

    for (name, config) in configs {
        println!(
            "{}: max_message_size={}KB, max_streams={}",
            name,
            config.max_message_size / 1024,
            config.max_concurrent_streams
        );
    }

    // Custom configuration
    let custom = QuicConfig::new()
        .with_message_size(512 * 1024) // 512KB
        .with_max_streams(25)
        .with_idle_timeout(Duration::from_secs(45));

    println!(
        "Custom config: {}KB messages, {} streams, {}s timeout",
        custom.max_message_size / 1024,
        custom.max_concurrent_streams,
        custom.max_idle_timeout.as_secs()
    );
}

async fn demo_message_creation() {
    println!("2. Message Creation Demo");
    println!("---------------------------");

    // Create test payload
    let payload = TestMessage {
        content: "Hello QUIC Transport!".to_string(),
        number: 42,
    };

    // Wrap in RPC envelope
    let rpc_message = RpcMessage {
        version: 1,
        request_id: 123,
        method_id: method_ids::GET_ACCOUNT_INFO,
        message_type: MessageType::Request,
        payload: payload.clone(),
    };

    // Wrap in transport envelope
    let streamed_message =
        StreamedMessage::new(rpc_message.clone()).with_priority(MessagePriority::Normal);

    println!("Created message:");
    println!("  - Content: '{}'", payload.content);
    println!("  - Number: {}", payload.number);
    println!("  - Request ID: {}", streamed_message.message.request_id);
    println!("  - Priority: {:?}", streamed_message.priority);

    // Demonstrate serialization size
    let config = QuicConfig::default();
    match streamed_message.validate(&config) {
        Ok(()) => println!("Message validation passed"),
        Err(e) => println!("❌ Message validation failed: {:?}", e),
    }

    // Show different priorities
    let priorities = vec![
        MessagePriority::Low,
        MessagePriority::Normal,
        MessagePriority::High,
        MessagePriority::Critical,
    ];

    println!("Priority levels:");
    for priority in priorities {
        let msg = StreamedMessage::new(rpc_message.clone()).with_priority(priority.clone());
        println!("  - {:?}: timeout={}s", priority, msg.timeout.as_secs());
    }
}

async fn demo_stream_concepts() {
    println!("3. Stream Concepts Demo");
    println!("--------------------------");

    // Show how connection would work conceptually
    println!("Connection lifecycle:");
    println!("  1. QuicConnection::connect(addr, hostname, config)");
    println!("  2. connection.open_stream()");
    println!("  3. stream.send_request(message)");
    println!("  4. stream.recv_message()");
    println!("  5. stream.close()");

    println!("Stream multiplexing benefits:");
    println!("  - Multiple RPC calls over one connection");
    println!("  - Priority-based message scheduling");
    println!("  - Automatic stream pooling and reuse");
    println!("  - Flow control per stream");

    // Demonstrate different message sizes
    let config = QuicConfig::default();
    let test_sizes = vec![1_000, 10_000, 100_000, 1_000_000];

    println!("Message size validation:");
    for size in test_sizes {
        let large_payload = TestMessage {
            content: "x".repeat(size),
            number: size as u32,
        };

        let message = StreamedMessage::new(RpcMessage {
            version: 1,
            request_id: 1,
            method_id: method_ids::GET_ACCOUNT_INFO,
            message_type: MessageType::Request,
            payload: large_payload,
        });

        match message.validate(&config) {
            Ok(()) => println!(" {}KB message: OK", size / 1000),
            Err(_) => println!("  ❌ {}KB message: Too large", size / 1000),
        }
    }

    println!(" Error recovery strategies:");
    println!("  - Connection-level errors: Reconnect");
    println!("  - Stream-level errors: New stream");
    println!("  - Timeout errors: Retry with backoff");
    println!("  - Protocol errors: Validate and reject");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_configuration_presets() {
        let default = QuicConfig::default();
        let high_throughput = QuicConfig::high_throughput();
        let low_latency = QuicConfig::low_latency();

        // High throughput should have larger limits
        assert!(high_throughput.max_message_size > default.max_message_size);
        assert!(high_throughput.max_concurrent_streams > default.max_concurrent_streams);

        // Low latency should have shorter timeouts
        assert!(low_latency.max_idle_timeout < default.max_idle_timeout);
    }

    #[test]
    fn test_message_validation() {
        let config = QuicConfig::new().with_message_size(1000); // Very small limit

        let small_message = StreamedMessage::new(RpcMessage {
            version: 1,
            request_id: 1,
            method_id: method_ids::GET_ACCOUNT_INFO,
            message_type: MessageType::Request,
            payload: TestMessage {
                content: "small".to_string(),
                number: 1,
            },
        });

        let large_message = StreamedMessage::new(RpcMessage {
            version: 1,
            request_id: 1,
            method_id: method_ids::GET_ACCOUNT_INFO,
            message_type: MessageType::Request,
            payload: TestMessage {
                content: "x".repeat(10000), // Very large
                number: 1,
            },
        });

        assert!(small_message.validate(&config).is_ok());
        assert!(large_message.validate(&config).is_err());
    }

    #[test]
    fn test_priority_ordering() {
        assert_eq!(MessagePriority::Low as u8, 0);
        assert_eq!(MessagePriority::Normal as u8, 1);
        assert_eq!(MessagePriority::High as u8, 2);
        assert_eq!(MessagePriority::Critical as u8, 3);

        // Higher numeric value = higher priority
        assert!(MessagePriority::Critical as u8 > MessagePriority::Low as u8);
    }

    #[tokio::test]
    async fn test_basic_concepts() {
        // This test just ensures our demo functions don't panic
        demo_configuration().await;
        demo_message_creation().await;
        demo_stream_concepts().await;
    }
}
