use bytes::BytesMut;

// This is a TOOL used by other components, not the entry point
#[derive(Debug, Clone)]
pub struct LengthPrefixedFramer {
    state: FrameState,
    buffer: BytesMut,
}

#[derive(Debug, Clone)]
enum FrameState {
    ReadingLength,       // Expecting 4-byte length prefix
    ReadingMessage(u32), // Reading message of known length
}

impl LengthPrefixedFramer {
    pub fn new() -> Self {
        Self {
            state: FrameState::ReadingLength, // Start by reading length prefix
            buffer: BytesMut::with_capacity(8192),          // Start with 8kb buffer
        }
    }

    // The KEY method: feed it bytes, get back complete messages
    pub fn process_bytes(&mut self, input: &[u8]) -> Vec<Vec<u8>> {
        self.buffer.extend_from_slice(input);
        let mut complete_messages = Vec::new();

        loop {
            match self.state {
                FrameState::ReadingLength => {
                    if self.buffer.len() >= 4 {
                        // Extract length prefix
                        let len_bytes = self.buffer.split_to(4);
                        let message_len = u32::from_le_bytes([
                            len_bytes[0],
                            len_bytes[1],
                            len_bytes[2],
                            len_bytes[3],
                        ]);

                        self.state = FrameState::ReadingMessage(message_len);
                    } else {
                        break; // Need more bytes for length
                    }
                }

                FrameState::ReadingMessage(expected_len) => {
                    if self.buffer.len() >= expected_len as usize {
                        // Extract complete message
                        let message = self.buffer.split_to(expected_len as usize);
                        complete_messages.push(message.to_vec());

                        self.state = FrameState::ReadingLength; // Ready for next message
                    } else {
                        break; // Need more bytes for message
                    }
                }
            }
        }

        complete_messages
    }

    // For sending: wrap a message with length prefix
    pub fn frame_message(&self, message: &[u8]) -> Vec<u8> {
        let mut framed = Vec::with_capacity(4 + message.len());
        framed.extend_from_slice(&(message.len() as u32).to_le_bytes());
        framed.extend_from_slice(message);
        framed
    }

    // Check if we have partial data that needs more bytes
    pub fn needs_more_data(&self) -> bool {
        match self.state {
            FrameState::ReadingLength => self.buffer.len() < 4,
            FrameState::ReadingMessage(expected) => self.buffer.len() < expected as usize,
        }
    }
}

impl LengthPrefixedFramer {
    /// Create a framer with specific initial buffer capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            state: FrameState::ReadingLength,
            buffer: BytesMut::with_capacity(capacity),
        }
    }
    
    /// Create a framer optimized for small messages
    pub fn small_messages() -> Self {
        Self::with_capacity(1024)  // 1KB buffer
    }
    
    /// Create a framer optimized for large messages  
    pub fn large_messages() -> Self {
        Self::with_capacity(64 * 1024)  // 64KB buffer
    }
}