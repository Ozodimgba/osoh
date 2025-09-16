# RPC Transport Examples

This directory contains examples demonstrating the QUIC-based RPC transport layer.

## Examples

### 🧪 `basic_test.rs` - Concepts Demo

A conceptual demonstration showing how the transport layer works without actual networking:

```bash
cargo run --example basic_test
```

**What it demonstrates:**
- Configuration presets (default, high-throughput, low-latency, minimal)
- Message creation and validation
- Priority levels and timeouts  
- Stream multiplexing concepts
- Error recovery strategies

### 🔗 `transport_demo.rs` - Working Connection Test

A complete transport demonstration using in-memory channels to simulate networking:

```bash
cargo run --example transport_demo
```

**What it demonstrates:**
- Real message serialization with Bincode
- Client-server request-response patterns
- Protocol validation and error handling
- Configuration presets and validation
- Priority-based message handling
- Type-safe RPC message processing

**Note:** Uses channels instead of actual QUIC networking to focus on transport layer functionality.

## Running Tests

Run the example tests:

```bash
# Test basic concepts
cargo test --example basic_test

# Test transport demonstration
cargo test --example transport_demo

# Test main transport functionality
cargo test
```

## Key Concepts Demonstrated

### 1. **Configuration Management**
```rust
let config = QuicConfig::low_latency()
    .with_message_size(1024 * 1024)  // 1MB messages
    .with_max_streams(50)
    .with_idle_timeout(Duration::from_secs(15));
```

### 2. **Message Creation**
```rust
let request = StreamedMessage::new(RpcMessage {
    version: 1,
    request_id: 42,
    method_id: method_ids::GET_ACCOUNT_INFO,
    message_type: MessageType::Request,
    payload: my_data,
}).with_priority(MessagePriority::High);
```

### 3. **Connection Lifecycle**
```rust
// Client side
let connection = QuicConnection::connect(addr, "localhost", config).await?;
let mut stream = connection.open_stream().await?;
let response = stream.send_request(request).await?;
stream.close().await?;
```

### 4. **Server Side**
```rust
// Accept incoming connections
while let Some(connecting) = endpoint.accept().await {
    let conn = QuicConnection::from_connecting(connecting, config).await?;
    tokio::spawn(handle_client(conn));
}
```

## Architecture Layers

The examples show how these components work together:

```
┌─────────────────┐
│   Application   │  ← Your RPC methods
├─────────────────┤
│ rpc-transport   │  ← QUIC networking (this crate)
├─────────────────┤
│   rpc-codec     │  ← Message serialization  
├─────────────────┤
│    rpc-io       │  ← Async I/O primitives
└─────────────────┘
```

## Performance Features

- **Stream Multiplexing**: Multiple RPC calls over one connection
- **Priority Queuing**: Critical messages get priority
- **Connection Pooling**: Reuse connections efficiently  
- **Flow Control**: Per-stream and connection-level limits
- **Binary Protocol**: Faster than JSON with Bincode

## Current Limitations

These are proof-of-concept examples. For production use, you'll need:

- [ ] Replace channels with actual QUIC networking
- [ ] Proper certificate management  
- [ ] Connection recovery and retry logic
- [ ] Comprehensive error handling
- [ ] Performance tuning
- [ ] Security hardening
- [ ] Monitoring and observability

## Next Steps

1. **Try the basic demo** to understand concepts
2. **Run the transport demo** to see message flow in action  
3. **Check the main transport tests** with `cargo test`
4. **Explore the source code** to understand implementation
5. **Build your RPC methods** on top of this foundation

The transport layer is designed to be the high-performance backbone for Solana RPC operations, providing the networking foundation for fast, reliable blockchain interactions.