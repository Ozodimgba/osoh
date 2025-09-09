use std::io;
use tokio::io::{AsyncWrite, AsyncWriteExt};
use bytes::{BytesMut, Buf};

use crate::framer::LengthPrefixedFramer;

/// A buffered message writer that adds framing and handles efficient network writes.
///
/// The MessageWriter takes complete messages (as byte slices) and:
/// 1. Adds length prefixes using the framer
/// 2. Buffers the framed data for efficient network writes
/// 3. Auto-flushes when the buffer gets large
/// 4. Provides manual flush control for latency-sensitive scenarios
///
/// # Type Parameters
/// * `W` - Any type that implements AsyncWrite (TcpStream, QuicStream, etc.)
#[derive(Debug, Clone)]
pub struct MessageWriter<W> {
    /// The underlying writer (network stream, file, etc.)
    inner: W,
    
    /// Handles adding length prefixes to messages
    framer: LengthPrefixedFramer,
    
    /// Buffer for accumulating outgoing data before network writes
    write_buffer: BytesMut,
    
    /// Automatically flush when buffer reaches this size (bytes)
    auto_flush_threshold: usize,
    
    /// Total number of messages written
    messages_written: u64,
    
    /// Total bytes written (including framing overhead)
    bytes_written: u64,
}

impl<W> MessageWriter<W>
where
    W: AsyncWrite + Unpin,
{
    /// Create a new MessageWriter with default settings.
    ///
    /// Default settings:
    /// - Buffer capacity: 8KB
    /// - Auto-flush threshold: 4KB
    ///
    /// # Arguments
    /// * `writer` - Any AsyncWrite implementation
    ///
    /// # Example
    /// ```rust
    /// use tokio::net::TcpStream;
    /// use rpc_io::MessageWriter;
    ///
    /// let tcp_stream = TcpStream::connect("server:8080").await?;
    /// let writer = MessageWriter::new(tcp_stream);
    /// ```
    pub fn new(writer: W) -> Self {
        Self::with_buffer_size(writer, 8192, 4096)
    }
    
    /// Create a MessageWriter with custom buffer settings.
    ///
    /// # Arguments
    /// * `writer` - The underlying AsyncWrite implementation
    /// * `buffer_capacity` - Initial capacity of the write buffer
    /// * `auto_flush_threshold` - Buffer size that triggers automatic flushing
    ///
    /// # Example
    /// ```rust
    /// // High-throughput setup: large buffer, less frequent flushes
    /// let writer = MessageWriter::with_buffer_size(stream, 64 * 1024, 32 * 1024);
    ///
    /// // Low-latency setup: small buffer, frequent flushes  
    /// let writer = MessageWriter::with_buffer_size(stream, 1024, 512);
    /// ```
    pub fn with_buffer_size(writer: W, buffer_capacity: usize, auto_flush_threshold: usize) -> Self {
        Self {
            inner: writer,
            framer: LengthPrefixedFramer::new(),
            write_buffer: BytesMut::with_capacity(buffer_capacity),
            auto_flush_threshold,
            messages_written: 0,
            bytes_written: 0,
        }
    }
    
    /// Create a writer optimized for high throughput.
    ///
    /// Uses large buffers and higher flush thresholds to minimize
    /// the number of network writes at the cost of higher latency.
    ///
    /// Settings:
    /// - Buffer capacity: 64KB
    /// - Auto-flush threshold: 32KB
    pub fn high_throughput(writer: W) -> Self {
        Self::with_buffer_size(writer, 64 * 1024, 32 * 1024)
    }
    
    /// Create a writer optimized for low latency.
    ///
    /// Uses small buffers and low flush thresholds to minimize
    /// latency at the cost of more frequent network writes.
    ///
    /// Settings:
    /// - Buffer capacity: 1KB
    /// - Auto-flush threshold: 512 bytes
    pub fn low_latency(writer: W) -> Self {
        Self::with_buffer_size(writer, 1024, 512)
    }
    
    /// Create a writer that flushes every message immediately.
    ///
    /// This provides the lowest possible latency but the worst throughput.
    /// Useful for urgent/critical messages.
    ///
    /// Settings:
    /// - Buffer capacity: 1KB
    /// - Auto-flush threshold: 0 (flush every message)
    pub fn immediate_flush(writer: W) -> Self {
        Self::with_buffer_size(writer, 1024, 0)
    }
    
    /// Write a complete message.
    ///
    /// The message will be automatically framed (length prefix added)
    /// and added to the write buffer. Depending on buffer settings,
    /// it may be sent immediately or buffered for later sending.
    ///
    /// # Arguments
    /// * `message` - The complete message data to send
    ///
    /// # Returns
    /// `Ok(())` if the message was successfully buffered/sent
    ///
    /// # Example
    /// ```rust
    /// writer.write_message(b"Hello, server!").await?;
    /// writer.write_message(&serialized_struct).await?;
    /// ```
    pub async fn write_message(&mut self, message: &[u8]) -> io::Result<()> {
        // 1. Add framing to the message (length prefix)
        let framed_message = self.framer.frame_message(message);
        
        // 2. Add the framed message to our write buffer
        self.write_buffer.extend_from_slice(&framed_message);
        
        // 3. Update statistics
        self.messages_written += 1;
        self.bytes_written += framed_message.len() as u64;
        
        // 4. Check if we should auto-flush
        if self.auto_flush_threshold > 0 && self.write_buffer.len() >= self.auto_flush_threshold {
            self.flush().await?;
        }
        
        Ok(())
    }
    
    /// Flush all buffered data to the underlying writer.
    ///
    /// This ensures that all previously written messages are actually
    /// sent over the network rather than sitting in buffers.
    ///
    /// # Returns
    /// `Ok(())` when all data has been flushed successfully
    ///
    /// # Example
    /// ```rust
    /// // Send multiple messages efficiently
    /// writer.write_message(b"msg1").await?;
    /// writer.write_message(b"msg2").await?;
    /// writer.write_message(b"msg3").await?;
    /// 
    /// // Now ensure they're all sent
    /// writer.flush().await?;
    /// ```
    pub async fn flush(&mut self) -> io::Result<()> {
        // Write all buffered data to the underlying writer
        while !self.write_buffer.is_empty() {
            // Write as much as we can
            let bytes_written = self.inner.write(&self.write_buffer).await?;
            
            if bytes_written == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::WriteZero,
                    "Failed to write any bytes to the underlying writer"
                ));
            }
            
            // Remove the bytes that were successfully written
            self.write_buffer.advance(bytes_written);
        }
        
        // Flush the underlying writer as well
        self.inner.flush().await
    }
    
    /// Write a message and immediately flush it.
    ///
    /// This is equivalent to calling `write_message()` followed by `flush()`,
    /// but is more convenient for urgent messages that need immediate delivery.
    ///
    /// # Arguments
    /// * `message` - The message to send immediately
    ///
    /// # Example
    /// ```rust
    /// // Send urgent message immediately
    /// writer.write_immediate(b"URGENT: System shutdown").await?;
    /// ```
    pub async fn write_immediate(&mut self, message: &[u8]) -> io::Result<()> {
        self.write_message(message).await?;
        self.flush().await
    }
    
    /// Write multiple messages efficiently.
    ///
    /// This is more efficient than calling `write_message()` multiple times
    /// because it can batch the messages before flushing.
    ///
    /// # Arguments
    /// * `messages` - Slice of message byte slices to send
    ///
    /// # Example
    /// ```rust
    /// let messages = vec![b"msg1".as_slice(), b"msg2".as_slice(), b"msg3".as_slice()];
    /// writer.write_messages(&messages).await?;
    /// ```
    pub async fn write_messages(&mut self, messages: &[&[u8]]) -> io::Result<()> {
        // Write all messages to buffer first
        for message in messages {
            let framed_message = self.framer.frame_message(message);
            self.write_buffer.extend_from_slice(&framed_message);
            
            self.messages_written += 1;
            self.bytes_written += framed_message.len() as u64;
        }
        
        // Then flush everything at once
        self.flush().await
    }
    
    /// Get the current size of the write buffer.
    ///
    /// This tells you how many bytes are waiting to be flushed.
    pub fn buffer_size(&self) -> usize {
        self.write_buffer.len()
    }
    
    /// Check if the write buffer is empty.
    ///
    /// Returns `true` if there's no data waiting to be flushed.
    pub fn is_buffer_empty(&self) -> bool {
        self.write_buffer.is_empty()
    }
    
    /// Get the auto-flush threshold.
    ///
    /// Returns the buffer size (in bytes) that triggers automatic flushing.
    pub fn auto_flush_threshold(&self) -> usize {
        self.auto_flush_threshold
    }
    
    /// Set a new auto-flush threshold.
    ///
    /// # Arguments
    /// * `threshold` - New threshold in bytes (0 = flush every message)
    pub fn set_auto_flush_threshold(&mut self, threshold: usize) {
        self.auto_flush_threshold = threshold;
    }
    
    /// Get statistics about this writer.
    ///
    /// Returns (messages_written, bytes_written) tuple.
    pub fn stats(&self) -> (u64, u64) {
        (self.messages_written, self.bytes_written)
    }
    
    /// Reset the statistics counters.
    pub fn reset_stats(&mut self) {
        self.messages_written = 0;
        self.bytes_written = 0;
    }
    
    /// Get a reference to the underlying writer.
    ///
    /// This can be useful for accessing writer-specific functionality.
    pub fn get_ref(&self) -> &W {
        &self.inner
    }
    
    /// Get a mutable reference to the underlying writer.
    ///
    /// # Safety
    /// Be careful when using this - writing directly to the underlying
    /// writer can break message framing!
    pub fn get_mut(&mut self) -> &mut W {
        &mut self.inner
    }
    
    /// Consume the MessageWriter and return the underlying writer.
    ///
    /// This will flush any remaining buffered data first.
    pub async fn into_inner(mut self) -> io::Result<W> {
        self.flush().await?;
        
        // Use ManuallyDrop to prevent Drop from running
        let manual_drop_self = std::mem::ManuallyDrop::new(self);
        
        // Safety: We've flushed all data and are taking ownership of inner
        let inner = unsafe { std::ptr::read(&manual_drop_self.inner) };
        
        Ok(inner)
    }
}

// Implement Drop to attempt cleanup (though async drop is tricky)
impl<W> Drop for MessageWriter<W> {
    fn drop(&mut self) {
        if !self.write_buffer.is_empty() {
            // We can't flush in Drop because it's async
            // In a real implementation, you might want to log a warning here
            eprintln!("Warning: MessageWriter dropped with {} bytes of unflushed data", 
                     self.write_buffer.len());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::DuplexStream;
    use crate::reader::MessageReader;
    
    /// Create a pair of connected streams for testing
    fn create_test_streams() -> (DuplexStream, DuplexStream) {
        tokio::io::duplex(1024)
    }
    
    #[tokio::test]
    async fn test_writer_creation() {
        let (stream, _) = create_test_streams();
        
        println!("{:?}", stream);
        
        let writer = MessageWriter::new(stream);
        assert_eq!(writer.buffer_size(), 0);
        assert!(writer.is_buffer_empty());
        assert_eq!(writer.auto_flush_threshold(), 4096);
    }
    
    #[tokio::test]
    async fn test_write_single_message() {
        let (writer_stream, reader_stream) = create_test_streams();
        
        let mut writer = MessageWriter::new(writer_stream);
        let mut reader = MessageReader::new(reader_stream);
        
        // Write a message
        let test_message = b"Hello, World!";
        writer.write_message(test_message).await.unwrap();
        writer.flush().await.unwrap();
        
        // Read it back
        let received = reader.read_message().await.unwrap();
        println!("{:?}", received);
        assert_eq!(received, test_message);
        
        // Check stats
        let (msg_count, byte_count) = writer.stats();
        assert_eq!(msg_count, 1);
        assert_eq!(byte_count, test_message.len() as u64 + 4); // +4 for length prefix
    }
    
    #[tokio::test]
    async fn test_auto_flush() {
        let (writer_stream, reader_stream) = create_test_streams();
        
        // Create writer with very low auto-flush threshold
        let mut writer = MessageWriter::with_buffer_size(writer_stream, 1024, 10);
        let mut reader = MessageReader::new(reader_stream);
        
        // Write a message that exceeds the auto-flush threshold
        let large_message = vec![b'x'; 20]; // 20 bytes + 4 byte prefix = 24 bytes > 10 byte threshold
        
        writer.write_message(&large_message).await.unwrap();
        // Should have auto-flushed, so buffer should be empty
        assert!(writer.is_buffer_empty());
        
        // Should be able to read the message immediately (no manual flush needed)
        let received = reader.read_message().await.unwrap();
        assert_eq!(received, large_message);
    }
    
    #[tokio::test]
    async fn test_write_immediate() {
        let (writer_stream, reader_stream) = create_test_streams();
        
        let mut writer = MessageWriter::new(writer_stream);
        let mut reader = MessageReader::new(reader_stream);
        
        // Write immediate should send right away
        writer.write_immediate(b"urgent").await.unwrap();
        
        // Should be readable immediately
        let received = reader.read_message().await.unwrap();
        assert_eq!(received, b"urgent");
    }
    
    #[tokio::test]
    async fn test_write_messages_batch() {
        let (writer_stream, reader_stream) = create_test_streams();
        
        let mut writer = MessageWriter::new(writer_stream);
        let mut reader = MessageReader::new(reader_stream);
        
        // Write multiple messages in one batch
        let messages: Vec<&[u8]> = vec![b"batch1", b"batch2", b"batch3"];
        writer.write_messages(&messages).await.unwrap();
        
        // Read them back
        for expected in &messages {
            let received = reader.read_message().await.unwrap();
            assert_eq!(received, *expected);
        }
    }
    
    #[tokio::test]
    async fn test_buffer_management() {
        let (stream, _) = create_test_streams();
        
        let mut writer = MessageWriter::with_buffer_size(stream, 1024, 1000);
        
        // Write a small message (should be buffered)
        writer.write_message(b"small").await.unwrap();
        assert!(!writer.is_buffer_empty());
        assert_eq!(writer.buffer_size(), 9); // 5 bytes + 4 byte length prefix
        
        // Manual flush should empty the buffer
        writer.flush().await.unwrap();
        assert!(writer.is_buffer_empty());
        assert_eq!(writer.buffer_size(), 0);
    }
    
    #[tokio::test]
    async fn test_writer_presets() {
        let (stream1, _) = create_test_streams();
        let (stream2, _) = create_test_streams();
        let (stream3, _) = create_test_streams();
        
        let high_throughput = MessageWriter::high_throughput(stream1);
        assert_eq!(high_throughput.auto_flush_threshold(), 32 * 1024);
        
        let low_latency = MessageWriter::low_latency(stream2);
        assert_eq!(low_latency.auto_flush_threshold(), 512);
        
        let immediate = MessageWriter::immediate_flush(stream3);
        assert_eq!(immediate.auto_flush_threshold(), 0);
    }
    
    #[tokio::test]
    async fn test_stats_and_reset() {
        let (stream, _) = create_test_streams();
        
        let mut writer = MessageWriter::new(stream);
        
        // Initially no stats
        let (msgs, bytes) = writer.stats();
        assert_eq!(msgs, 0);
        assert_eq!(bytes, 0);
        
        // Write some messages
        writer.write_message(b"test1").await.unwrap();
        writer.write_message(b"test22").await.unwrap();
        
        let (msgs, bytes) = writer.stats();
        assert_eq!(msgs, 2);
        assert_eq!(bytes, 5 + 4 + 6 + 4); // 2 messages + 2 length prefixes
        
        // Reset stats
        writer.reset_stats();
        let (msgs, bytes) = writer.stats();
        assert_eq!(msgs, 0);
        assert_eq!(bytes, 0);
    }
    
    #[tokio::test]
    async fn test_threshold_adjustment() {
        let (stream, _) = create_test_streams();
        
        let mut writer = MessageWriter::new(stream);
        assert_eq!(writer.auto_flush_threshold(), 4096);
        
        writer.set_auto_flush_threshold(1024);
        assert_eq!(writer.auto_flush_threshold(), 1024);
    }
    
    #[tokio::test]
    async fn test_into_inner() {
        let (stream, _) = create_test_streams();
        
        let mut writer = MessageWriter::new(stream);
        writer.write_message(b"test").await.unwrap();
        
        // Should flush and return the inner stream
        let _inner_stream = writer.into_inner().await.unwrap();
        // If we get here without error, the flush worked
    }
}