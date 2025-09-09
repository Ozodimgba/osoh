//! Core transport traits and abstractions

use crate::error::{TransportError, TransportResult};
use async_trait::async_trait;
use std::net::SocketAddr;

/// Core transport trait for sending/receiving raw bytes
#[async_trait]
pub trait Transport: Send + Sync + 'static {
    /// Send raw bytes
    async fn send(&self, data: &[u8]) -> TransportResult<()>;

    /// Receive raw bytes
    async fn recv(&self) -> TransportResult<Vec<u8>>;

    /// Check if transport is connected
    fn is_connected(&self) -> bool;

    /// Get peer address
    fn peer_addr(&self) -> Option<SocketAddr>;

    /// Close the transport
    async fn close(&self) -> TransportResult<()>;
}

/// Stream-based transport (like QUIC)
#[async_trait]
pub trait StreamTransport: Send + Sync + 'static {
    type Stream: Stream;

    /// Open a new stream
    async fn open_stream(&self) -> TransportResult<Self::Stream>;

    /// Accept an incoming stream
    async fn accept_stream(&self) -> TransportResult<Self::Stream>;

    /// Get connection info
    fn connection_info(&self) -> ConnectionInfo;
}

/// Individual stream for bidirectional communication
#[async_trait]
pub trait Stream: Send + Sync + 'static {
    /// Send data on this stream
    async fn send(&mut self, data: &[u8]) -> TransportResult<()>;

    /// Receive data from this stream
    async fn recv(&mut self) -> TransportResult<Vec<u8>>;

    /// Send a framed message (length-prefixed)
    async fn send_message(&mut self, data: &[u8]) -> TransportResult<()> {
        // Default implementation: 4-byte length prefix + data
        let len = data.len() as u32;
        let len_bytes = len.to_le_bytes();

        self.send(&len_bytes).await?;
        self.send(data).await?;

        Ok(())
    }

    /// Receive a framed message
    async fn recv_message(&mut self) -> TransportResult<Vec<u8>> {
        // Default implementation: read 4-byte length, then payload
        let mut len_buf = [0u8; 4];
        self.recv_exact(&mut len_buf).await?;

        let len = u32::from_le_bytes(len_buf) as usize;
        if len > 10 * 1024 * 1024 {
            // 10MB limit
            return Err(TransportError::MessageTooLarge {
                size: len,
                limit: 10 * 1024 * 1024,
            });
        }

        let mut data = vec![0u8; len];
        self.recv_exact(&mut data).await?;

        Ok(data)
    }

    /// Helper: receive exact number of bytes
    async fn recv_exact(&mut self, buf: &mut [u8]) -> TransportResult<()> {
        let mut total_read = 0;

        while total_read < buf.len() {
            let data = self.recv().await?;
            let to_copy = std::cmp::min(data.len(), buf.len() - total_read);
            buf[total_read..total_read + to_copy].copy_from_slice(&data[..to_copy]);
            total_read += to_copy;
        }

        Ok(())
    }

    /// Close this stream
    async fn close(&mut self) -> TransportResult<()>;

    /// Get stream ID
    fn stream_id(&self) -> u64;
}

/// Client-side transport for making requests
#[async_trait]
pub trait ClientTransport: Send + Sync + 'static {
    /// Connect to a server
    async fn connect(addr: SocketAddr) -> TransportResult<Self>
    where
        Self: Sized;

    /// Send a request and wait for response
    async fn request(&self, data: &[u8]) -> TransportResult<Vec<u8>>;

    /// Start a subscription (long-lived stream)
    async fn subscribe(&self) -> TransportResult<Box<dyn Stream>>;
}

/// Server-side transport for handling requests
#[async_trait]
pub trait ServerTransport: Send + Sync + 'static {
    /// Bind to an address and start listening
    async fn bind(addr: SocketAddr) -> TransportResult<Self>
    where
        Self: Sized;

    /// Accept incoming connections
    async fn accept(&self) -> TransportResult<Box<dyn StreamTransport<Stream = Box<dyn Stream>>>>; // shit code, refactor
}

/// Connection metadata
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub local_addr: SocketAddr,
    pub peer_addr: SocketAddr,
    pub connection_id: u64,
    pub established_at: std::time::Instant,
}

/// Stream metadata
#[derive(Debug, Clone)]
pub struct StreamInfo {
    pub stream_id: u64,
    pub is_bidirectional: bool,
    pub bytes_sent: u64,
    pub bytes_received: u64,
}
