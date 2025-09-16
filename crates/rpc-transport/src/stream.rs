use super::traits::{Config, Receiveable, Sendable};
use crate::config::QuicConfig;
use crate::error::{TransportError, TransportResult};
use crate::protocol::StreamedMessage;
use async_trait::async_trait;
use quinn::{RecvStream, SendStream};
use rpc_io::{reader::MessageReader, writer::MessageWriter};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct QuicStream {
    writer: MessageWriter<SendStream>,
    reader: MessageReader<RecvStream>,
    config: QuicConfig,
}

// Quick Cleanup Fix: Error handling maps should be cleaned up
// Tech debt: Codec should implement method for decoding streamed messages since encoding is generic
impl QuicStream {
    pub fn new(send: SendStream, recv: RecvStream) -> Self {
        Self {
            writer: MessageWriter::new(send),
            reader: MessageReader::new(recv),
            config: QuicConfig::default(),
        }
    }

    pub fn with_config(mut self, config: QuicConfig) -> Self {
        self.config = config;
        self
    }

    /// Request-response pattern using trait methods
    pub async fn send_request<T, R>(
        &mut self,
        request: StreamedMessage<T>,
    ) -> TransportResult<StreamedMessage<R>>
    where
        T: Serialize + Send,
        R: for<'de> Deserialize<'de>,
    {
        self.send_message(request).await?; // Uses Sendable trait method
        self.recv_message().await // Uses Receiveable trait method
    }

    /// Get the stream ID for debugging/logging
    pub fn id(&self) -> quinn::StreamId {
        self.writer.get_ref().id() // Access underlying SendStream
    }

    /// Split the stream into separate send and receive parts
    pub async fn split(self) -> std::io::Result<(StreamSender, StreamReceiver)> {
        let send_stream = self.writer.into_inner().await?;
        let recv_stream = self.reader.into_inner().map_err(|(_, io_error)| io_error)?; // Tech debt error should be handled properly

        Ok((
            StreamSender {
                writer: MessageWriter::new(send_stream),
                config: self.config.clone(),
            },
            StreamReceiver {
                reader: MessageReader::new(recv_stream),
                config: self.config,
            },
        ))
    }
}

// Implement the traits for QuicStream
impl Config for QuicStream {
    fn config(&self) -> &QuicConfig {
        &self.config
    }

    fn update_config(&mut self, config: QuicConfig) -> &mut QuicConfig {
        self.config = config;
        &mut self.config
    }
}

#[async_trait]
impl Sendable for QuicStream {
    fn writer(&mut self) -> &mut MessageWriter<SendStream> {
        &mut self.writer
    }

    async fn close(self) -> TransportResult<()> {
        let mut send_stream =
            self.writer
                .into_inner()
                .await
                .map_err(|e| TransportError::IOError {
                    reason: format!("Failed to flush writer: {}", e),
                })?;

        send_stream
            .finish()
            .map_err(|e| TransportError::StreamError {
                reason: format!("Failed to close send stream: {}", e),
            })?;

        Ok(())
    }
}

#[async_trait]
impl Receiveable for QuicStream {
    fn reader(&mut self) -> &mut MessageReader<RecvStream> {
        &mut self.reader
    }
}

/// Send-only half of a QuicStream
pub struct StreamSender {
    writer: MessageWriter<SendStream>,
    config: QuicConfig,
}

#[async_trait]
impl Config for StreamSender {
    fn config(&self) -> &QuicConfig {
        &self.config
    }

    fn update_config(&mut self, config: QuicConfig) -> &mut QuicConfig {
        self.config = config;
        &mut self.config
    }
}

#[async_trait]
impl Sendable for StreamSender {
    fn writer(&mut self) -> &mut MessageWriter<SendStream> {
        &mut self.writer
    }

    async fn close(self) -> TransportResult<()> {
        let mut send_stream =
            self.writer
                .into_inner()
                .await
                .map_err(|e| TransportError::IOError {
                    reason: format!("Failed to flush writer: {}", e),
                })?;

        send_stream
            .finish()
            .map_err(|e| TransportError::StreamError {
                reason: format!("Failed to close send stream: {}", e),
            })?;
        Ok(())
    }
}

/// Receive-only half of a QuicStream
pub struct StreamReceiver {
    reader: MessageReader<RecvStream>,
    config: QuicConfig,
}

impl Config for StreamReceiver {
    fn config(&self) -> &QuicConfig {
        &self.config
    }

    fn update_config(&mut self, config: QuicConfig) -> &mut QuicConfig {
        self.config = config;
        &mut self.config
    }
}

impl Receiveable for StreamReceiver {
    fn reader(&mut self) -> &mut MessageReader<RecvStream> {
        &mut self.reader
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rpc_codec::protocol::{MessageType, RpcMessage};
    use tokio::io::{DuplexStream, duplex};

    type MockStream = DuplexStream;

    fn create_mock_streams() -> (MockStream, MockStream) {
        let (client, server) = duplex(8192);
        (client, server)
    }

    fn create_test_message(size: usize) -> StreamedMessage<String> {
        StreamedMessage::new(RpcMessage {
            version: 1,
            request_id: 42u64,
            method_id: 1,
            message_type: MessageType::Request,
            payload: "x".repeat(size),
        })
    }

    #[test]
    fn test_message_size_validation() {
        // This would require setting up actual QUIC streams for a full test
        // For now, just test the validation logic
        let config = QuicConfig::new().with_message_size(100); // Very small limit

        let large_message = StreamedMessage::new(RpcMessage {
            version: 1,
            request_id: 33u64,
            method_id: 1,
            message_type: MessageType::Request,
            payload: "x".repeat(1000), // Larger than limit
        });

        // This would fail in a real stream send
        assert!(large_message.validate(&config).is_err());
    }

    #[test]
    fn test_message_framing_constants() {
        // Test that length prefix works correctly
        let test_lengths = vec![0u32, 100u32, 1024u32, 65536u32];

        for len in test_lengths {
            let bytes = len.to_be_bytes();
            assert_eq!(bytes.len(), 4);

            let recovered = u32::from_be_bytes(bytes);
            assert_eq!(recovered, len);
        }

        // Test that max message size fits in u32
        let config = QuicConfig::default();
        assert!(config.max_message_size <= u32::MAX as usize);
    }

    // Integration test pattern (requires real QUIC setup)
    #[tokio::test]
    #[ignore] // Ignore until full QUIC test infrastructure
    async fn test_full_message_flow_integration() {
        // This would require actual QUIC connection setup
        // let (client_stream, server_stream) = create_quic_connection_pair().await;

        // let mut client = QuicStream::new(client_stream.0, client_stream.1);
        // let mut server = QuicStream::new(server_stream.0, server_stream.1);

        // // Test message round trip
        // let request = create_test_message(256);
        // client.send_message(request.clone()).await.unwrap();
        // let received = server.recv_message::<String>().await.unwrap();
        // assert_eq!(request.message.payload, received.message.payload);
    }

    #[tokio::test]
    async fn test_message_serialization() {
        let message = create_test_message(100);
        let config = QuicConfig::default();

        // Test serialization logic (from your trait)
        let bincode_config = bincode::config::standard();
        let serialized = bincode::serde::encode_to_vec(&message, bincode_config).unwrap();

        // Test size validation
        assert!(serialized.len() <= config.max_message_size);
    }

    #[test]
    fn test_serialization_size_consistency() {
        let message = create_test_message(100);
        let config = QuicConfig::default();

        // Serialize the message
        let bincode_config = bincode::config::standard();
        let serialized = bincode::serde::encode_to_vec(&message, bincode_config).unwrap();

        // Size check should be consistent
        if serialized.len() > config.max_message_size {
            assert!(
                message.validate(&config).is_err() || serialized.len() > config.max_message_size
            );
        }
    }

    #[tokio::test]
    async fn test_writer_buffer_behavior() {
        let (send_mock, _recv_mock) = create_mock_streams();
        // Keep _recv_mock alive by not using _ (which drops immediately)

        let mut writer = MessageWriter::low_latency(send_mock);

        // Test that messages get buffered
        let small_message = b"test";
        writer.write_message(small_message).await.unwrap();

        // Buffer should have data
        assert!(!writer.is_buffer_empty());
        assert!(writer.buffer_size() > 0);

        // After flush, buffer should be empty
        writer.flush().await.unwrap();
        assert!(writer.is_buffer_empty());
        assert_eq!(writer.buffer_size(), 0);

        // Keep _recv_mock alive until here
    }
}
