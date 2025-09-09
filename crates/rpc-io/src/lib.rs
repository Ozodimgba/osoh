//! High-performance I/O layer for RPC systems
//!
//! Provides abstract I/O interfaces with multiple backend implementations:
//! - Tokio runtime (standard async I/O)
//! - tokio-uring runtime (high-performance io_uring)
//! - Custom io_uring runtime (maximum performance)

// #![forbid(unsafe_code)]

pub mod error;
pub mod framer;
pub mod reader;
pub mod traits;
pub mod types;
pub mod connection;
pub mod writer;

pub use error::*;
pub use framer::*;
pub use reader::*;
pub use traits::*;
pub use types::*;
pub use connection::*;
pub use writer::*;
