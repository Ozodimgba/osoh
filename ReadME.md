# OSOH: High Performance RPC Framework
### Outperforming gRPC with QUIC + io_uring + Bincode
<img width="1650" height="660" alt="image" src="https://github.com/user-attachments/assets/76619c32-de81-43fe-8ee1-bfecc7bd91de" />


**10x faster than JSON-RPC, designed to beat gRPC performance for ultra-low latency microservices.**

Built with modern protocols:
- **QUIC transport** - Multiplexed streams, 0-RTT, built-in encryption  
- **io_uring backend (optional)** - Kernel-bypass I/O for maximum throughput
- **Bincode serialization** - 2.8x faster encoding than Protocol Buffers

## Architecture

```
Application ──── QUIC Transport ──── Microservice
     │               │                     │
     └─────── Bincode Codec ──────────────┘
                     │
             io_uring I/O Backend
```

## Components

- **rpc-codec** ✅ - Binary serialization, protocol messages, request routing
- **rpc-transport** 🚧 - QUIC networking layer
- **rpc-io** 🚧 - io_uring backend
- **rpc-client** 🚧 - Client library
- **rpc-server** 🚧 - Server implementation
- **geyser-plugin** 🚧 - Real-time streaming

## Why OSOH Beats gRPC

**Serialization Performance** (Bincode vs Protocol Buffers):

```
Encoding:  (Bincode) 3.0ns vs (Prost) 8.4ns  (2.8x faster) 
                               ^
                               |
          (on stack allocation which we can do for Slolana)
=================================================================
Decoding:  (Bincode) 24.7ns vs (Prost) 40.5ns (1.6x faster)
=================================================================
Size:      (Bincode) 25 vs (Prost) 23 bytes   (8% larger)
=================================================================
Bincode outperforms Prost in serialized byte size for bigger values
When Benchmarked with Max Value: u32::MAX vs u64::MAX 

Data sizes:
  Protobuf: 31 bytes
  Bincode:  27 bytes
```

**Transport Advantages** (QUIC vs HTTP/2):
- Zero round-trip connection establishment
- No head-of-line blocking
- Built-in multiplexing without stream dependencies
- Better congestion control and loss recovery

At 50k msg/sec: ~2GB/day bandwidth savings + significant CPU reduction.

## Perfect For

- **High-frequency trading** - Microsecond latency requirements
- **Real-time gaming** - Fast state synchronization  
- **IoT/Edge computing** - Efficient binary protocols
- **Blockchain applications** - Including Solana RPC acceleration
- **Microservice mesh** - Ultra-fast service-to-service communication

## Status

Only `rpc-codec` is implemented. Transport, I/O, client/server still needed.

## Usage

```bash
# See working demo
cargo test complete_rpc_flow_demo -- --nocapture

# Run benchmarks
cargo bench
```

**Mission**: Replace slow JSON-RPC and outperform gRPC with modern binary protocols optimized for today's demanding applications.
