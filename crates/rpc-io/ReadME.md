eventual file structure

```
src/
├── lib.rs
├── error.rs
├── types.rs  
├── framing.rs           # ← Shared (no I/O dependencies)
├── traits.rs            # ← Shared (abstract interfaces)
├── config.rs            # ← Shared (configuration)
│
├── tokio/               # ← Tokio-based implementations
│   ├── mod.rs
│   ├── reader.rs
│   ├── writer.rs
│   └── connection.rs
│
├── uring/               # ← io_uring-based implementations  
│   ├── mod.rs
│   ├── reader.rs
│   ├── writer.rs
│   └── connection.rs
│
└── quic/                # ← QUIC-specific utilities
    ├── mod.rs
    └── stream_adapter.rs
    ```