use crate::framer::*;
use std::collections::VecDeque;
use tokio::io::{AsyncRead, AsyncReadExt};

#[derive(Debug)]
pub struct MessageReader<R> {
    inner: R, // The raw network stream
    framer: LengthPrefixedFramer,
    read_buffer: [u8; 8192],             // Temporary buffer for network reads
    pending_messages: VecDeque<Vec<u8>>, // Complete messages waiting to be delivered
}

impl<R: AsyncRead + Unpin> MessageReader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            inner: reader,
            framer: LengthPrefixedFramer::new(),
            read_buffer: [0; 8192],
            pending_messages: VecDeque::new(),
        }
    }

    pub async fn read_message(&mut self) -> tokio::io::Result<Vec<u8>> {
        // 1. Return any pending complete messages first
        if let Some(message) = self.pending_messages.pop_front() {
            return Ok(message);
        }

        // 2. Need to read more data from network
        loop {
            // Read raw bytes from network
            let bytes_read = self.inner.read(&mut self.read_buffer).await?;

            if bytes_read == 0 {
                return Err(tokio::io::Error::new(
                    tokio::io::ErrorKind::UnexpectedEof,
                    "Connection closed",
                ));
            }

            // 3. Feed bytes to framer to detect complete messages
            let messages = self.framer.process_bytes(&self.read_buffer[..bytes_read]);

            // 4. Store complete messages
            for msg in messages {
                self.pending_messages.push_back(msg);
            }

            // 5. Return first message if we have any
            if let Some(message) = self.pending_messages.pop_front() {
                return Ok(message);
            }

            // 6. Otherwise, loop and read more network data
        }
    }

    // Non-blocking version
    pub fn try_read_message(&mut self) -> tokio::io::Result<Option<Vec<u8>>> {
        // Return pending message if available
        if let Some(message) = self.pending_messages.pop_front() {
            return Ok(Some(message));
        }

        // Try to read more data (would need try_read support)
        Ok(None)
    }

    /// Consume the MessageReader and return the underlying reader.
    ///
    /// Returns an error if there are pending messages that haven't been read,
    /// to prevent data loss.
    pub fn into_inner(self) -> Result<R, (Self, std::io::Error)> {
        // Check if there are pending messages
        if !self.pending_messages.is_empty() {
            let error = std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "Cannot extract reader: {} pending messages would be lost",
                    self.pending_messages.len()
                ),
            );
            return Err((self, error));
        }

        // Check if framer has partial data
        if self.framer.needs_more_data() && !self.framer.buffer_is_empty() {
            let error = std::io::Error::new(
                std::io::ErrorKind::Other,
                "Cannot extract reader: framer has partial message data",
            );
            return Err((self, error));
        }

        // Safe to extract - use same pattern as MessageWriter
        let manual_drop_self = std::mem::ManuallyDrop::new(self);
        let inner = unsafe { std::ptr::read(&manual_drop_self.inner) };
        Ok(inner)
    }

    pub fn can_extract_safely(&self) -> bool {
        self.pending_messages.is_empty()
            && (!self.framer.needs_more_data() || self.framer.buffer_is_empty())
    }

    /// Get count of pending messages that would be lost
    pub fn pending_message_count(&self) -> usize {
        self.pending_messages.len()
    }
}
