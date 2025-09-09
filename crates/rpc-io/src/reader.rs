use crate::framer::*;
use std::collections::VecDeque;
use tokio::io::{AsyncRead, AsyncReadExt};

#[derive(Debug)]
pub struct MessageReader<R> {
    inner: R,                            // The raw network stream
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
}
