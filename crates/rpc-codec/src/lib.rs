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
