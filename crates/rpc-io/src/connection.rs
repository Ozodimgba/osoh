use std::io;
use std::net::SocketAddr;
use tokio::io::{AsyncRead, AsyncWrite, ReadHalf, WriteHalf};
use async_trait::async_trait;

use crate::reader::MessageReader;
use crate::writer::MessageWriter;
use crate::types::ConnectionInfo;
use crate::traits::{
    MessageReader as MessageReaderTrait, 
    MessageWriter as MessageWriterTrait, 
    MessageStream
};


/// A convenient wrapper that combines MessageReader and MessageWriter
/// into a single object that can both send and receive messages.
///
/// This is the main interface users will interact with - it hides the
/// complexity of managing separate reader/writer components.
#[derive(Debug)]
pub struct Connection<T> {
    /// Handles incoming message reading
    reader: MessageReader<ReadHalf<T>>,
    
    /// Handles outgoing message writing
    writer: MessageWriter<WriteHalf<T>>,
    
    /// Connection metadata and information
    info: ConnectionInfo,
}

impl<T> Connection<T>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    /// Create a new connection from any AsyncRead + AsyncWrite stream.
    ///
    /// This will automatically split the stream into read/write halves
    /// and set up the message framing for both directions.
    ///
    /// # Arguments
    /// * `stream` - Any stream that implements AsyncRead + AsyncWrite (TCP, QUIC, etc.)
    ///
    /// # Returns
    /// A new Connection ready for message-based communication
    ///
    /// # Example
    /// ```rust
    /// use tokio::net::TcpStream;
    /// use rpc_io::Connection;
    ///
    /// let tcp_stream = TcpStream::connect("server:8080").await?;
    /// let mut connection = Connection::new(tcp_stream)?;
    /// ```
    pub fn new(stream: T) -> io::Result<Self> {
        // Split the stream into separate read and write halves
        // This allows concurrent reading and writing
        let (read_half, write_half) = tokio::io::split(stream);
        
        // Create the reader and writer components
        let reader = MessageReader::new(read_half);
        let writer = MessageWriter::new(write_half);
        
        // Create connection info with reasonable defaults
        // Note: For real implementations, you'd extract actual addresses from the stream
        let info = ConnectionInfo::new(
            "127.0.0.1:0".parse().unwrap(),  // TODO: Get actual local addr
            "127.0.0.1:0".parse().unwrap(),  // TODO: Get actual remote addr
            "Unknown",                       // TODO: Detect protocol type
        );
        
        Ok(Self {
            reader,
            writer,
            info,
        })
    }
    
    /// Create a connection with explicit connection information.
    ///
    /// This is useful when you want to provide specific metadata
    /// about the connection (addresses, protocol type, etc.)
    pub fn with_info(stream: T, info: ConnectionInfo) -> Self {
        let (read_half, write_half) = tokio::io::split(stream);
        
        Self {
            reader: MessageReader::new(read_half),
            writer: MessageWriter::new(write_half),
            info,
        }
    }
    
    /// Split the connection into separate reader and writer components.
    ///
    /// This is useful when you want to handle reading and writing
    /// in separate tasks for maximum concurrency.
    ///
    /// # Returns
    /// A tuple of (MessageReader, MessageWriter)
    ///
    /// # Example
    /// ```rust
    /// let connection = Connection::new(stream)?;
    /// let (mut reader, mut writer) = connection.split();
    /// 
    /// // Handle reading in one task
    /// tokio::spawn(async move {
    ///     while let Ok(msg) = reader.read_message().await {
    ///         process_message(msg).await;
    ///     }
    /// });
    /// 
    /// // Handle writing in another task
    /// tokio::spawn(async move {
    ///     writer.write_message(b"hello").await?;
    /// });
    /// ```
    pub fn split(self) -> (MessageReader<ReadHalf<T>>, MessageWriter<WriteHalf<T>>) {
        (self.reader, self.writer)
    }
    
    /// Get a reference to the connection information.
    ///
    /// This provides metadata about the connection such as
    /// local/remote addresses, protocol type, connection duration, etc.
    pub fn connection_info(&self) -> &ConnectionInfo {
        &self.info
    }
    
    /// Get the local address of this connection.
    ///
    /// This is a convenience method that extracts the local address
    /// from the connection info.
    pub fn local_addr(&self) -> SocketAddr {
        self.info.local_addr
    }
    
    /// Get the remote peer address of this connection.
    ///
    /// This is a convenience method that extracts the remote address
    /// from the connection info.
    pub fn remote_addr(&self) -> SocketAddr {
        self.info.remote_addr
    }
    
    /// Get the protocol being used for this connection.
    ///
    /// Returns a string like "TCP", "QUIC", "WebSocket", etc.
    pub fn protocol(&self) -> &str {
        &self.info.protocol
    }
    
    /// Get how long this connection has been active.
    ///
    /// Returns the duration since the connection was established.
    pub fn connection_duration(&self) -> std::time::Duration {
        self.info.connection_duration()
    }
    
    /// Get the unique connection ID.
    ///
    /// This can be useful for logging and tracking connections.
    pub fn connection_id(&self) -> &str {
        &self.info.connection_id
    }
    
    /// Gracefully close the connection.
    ///
    /// This will flush any pending writes and then close the connection.
    /// After calling this, the connection should not be used anymore.
    ///
    /// # Example
    /// ```rust
    /// connection.write_message(b"goodbye").await?;
    /// connection.close().await?;
    /// ```
    pub async fn close(mut self) -> io::Result<()> {
        // Flush any pending writes first
        self.writer.flush().await?;
        
        // The connection will be dropped when this function returns,
        // which should close the underlying stream
        Ok(())
    }
    
    /// Check if the connection appears to be still active.
    ///
    /// Note: This is a best-effort check and may not catch all
    /// connection failures immediately.
    pub fn is_connected(&self) -> bool {
        // For now, we assume the connection is active
        // A more sophisticated implementation might check the underlying stream status
        true
    }
    
}

// Implement the MessageStream trait by delegating to our reader/writer components
#[async_trait]
impl<T> MessageReaderTrait for Connection<T>
where
    T: AsyncRead + AsyncWrite + Unpin + Send,
{
    async fn read_message(&mut self) -> io::Result<Vec<u8>> {
        self.reader.read_message().await
    }
    
    fn try_read_message(&mut self) -> io::Result<Option<Vec<u8>>> {
        self.reader.try_read_message()
    }
}

#[async_trait]
impl<T> MessageWriterTrait for Connection<T>
where
    T: AsyncRead + AsyncWrite + Unpin + Send,
{
    async fn write_message(&mut self, message: &[u8]) -> io::Result<()> {
        self.writer.write_message(message).await
    }
    
    async fn flush(&mut self) -> io::Result<()> {
        self.writer.flush().await
    }
}

impl<T> MessageStream for Connection<T>
where
    T: AsyncRead + AsyncWrite + Unpin + Send,
{
    fn connection_info(&self) -> &ConnectionInfo {
        &self.info
    }
}

// Additional convenience implementations

impl<T> Connection<T>
where
    T: AsyncRead + AsyncWrite,
{
    /// Send a message and immediately flush it to the network.
    ///
    /// This is equivalent to calling write_message() followed by flush(),
    /// but is more convenient when you want to ensure immediate delivery.
    ///
    /// # Example
    /// ```rust
    /// // Send urgent message immediately
    /// connection.send_immediate(b"urgent data").await?;
    /// ```
    pub async fn send_immediate(&mut self, message: &[u8]) -> io::Result<()> {
        self.writer.write_message(message).await?;
        self.writer.flush().await
    }
    
    /// Try to read a message without blocking.
    ///
    /// Returns `Ok(Some(message))` if a complete message is available,
    /// `Ok(None)` if no complete message is ready yet, or an error.
    ///
    /// This is useful for non-blocking message processing.
    pub fn try_read_message(&mut self) -> io::Result<Option<Vec<u8>>> {
        self.reader.try_read_message()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{duplex, DuplexStream};
    
    /// Create a pair of connected streams for testing
    fn create_test_streams() -> (DuplexStream, DuplexStream) {
        tokio::io::duplex(1024)
    }
    
    #[tokio::test]
    async fn test_connection_creation() {
        let (stream1, _stream2) = create_test_streams();
        
        let connection = Connection::new(stream1).unwrap();
        
        // Check that connection info is populated
        assert!(!connection.connection_id().is_empty());
        assert_eq!(connection.protocol(), "Unknown"); // Default value
    }
    
    #[tokio::test]
    async fn test_basic_message_flow() {
        let (stream1, stream2) = create_test_streams();
        
        let mut conn1 = Connection::new(stream1).unwrap();
        
        println!("{:?}", conn1);
        let mut conn2 = Connection::new(stream2).unwrap();
        
        // Send a message from conn1 to conn2
        let test_message = b"Hello, World!";
        conn1.write_message(test_message).await.unwrap();
        
        println!("{:?}", conn1);
        conn1.flush().await.unwrap();
        
        // Read the message on conn2
        let received = conn2.read_message().await.unwrap();
        assert_eq!(received, test_message);
    }
    
    #[tokio::test]
    async fn test_connection_splitting() {
        let (stream1, stream2) = create_test_streams();
        
        let conn1 = Connection::new(stream1).unwrap();
        let mut conn2 = Connection::new(stream2).unwrap();
        
        // Split conn1 into reader and writer
        let (mut reader, mut writer) = conn1.split();
        
        // Send from writer to conn2
        writer.write_message(b"test").await.unwrap();
        writer.flush().await.unwrap();
        
        // Read on conn2
        let received = conn2.read_message().await.unwrap();
        assert_eq!(received, b"test");
        
        // Send from conn2 to reader
        conn2.write_message(b"response").await.unwrap();
        conn2.flush().await.unwrap();
        
        // Read on the split reader
        let response = reader.read_message().await.unwrap();
        assert_eq!(response, b"response");
    }
    
    #[tokio::test]
    async fn test_send_immediate() {
        let (stream1, stream2) = create_test_streams();
        
        let mut conn1 = Connection::new(stream1).unwrap();
        let mut conn2 = Connection::new(stream2).unwrap();
        
        // Send immediate (write + flush in one call)
        conn1.send_immediate(b"immediate").await.unwrap();
        
        // Should be received immediately
        let received = conn2.read_message().await.unwrap();
        assert_eq!(received, b"immediate");
    }
    
    #[tokio::test]
    async fn test_connection_info() {
        let (stream, _) = create_test_streams();
        
        let connection = Connection::new(stream).unwrap();
        
        // Test connection info methods
        let info = connection.connection_info();
        assert!(!info.connection_id.is_empty());
        
        let local_addr = connection.local_addr();
        let remote_addr = connection.remote_addr();
        let protocol = connection.protocol();
        let duration = connection.connection_duration();
        
        // Just verify these don't panic
        assert_eq!(local_addr, info.local_addr);
        assert_eq!(remote_addr, info.remote_addr);
        assert_eq!(protocol, &info.protocol);
        assert!(duration.as_nanos() > 0);
    }
    
    #[tokio::test]
    async fn test_graceful_close() {
        let (stream1, stream2) = create_test_streams();
        
        let mut conn1 = Connection::new(stream1).unwrap();
        let mut conn2 = Connection::new(stream2).unwrap();
        
        // Send a message
        conn1.write_message(b"goodbye").await.unwrap();
        
        // Close gracefully (should flush first)
        conn1.close().await.unwrap();
        
        // Should still be able to read the message
        let received = conn2.read_message().await.unwrap();
        assert_eq!(received, b"goodbye");
    }
}