//! Core transport traits and abstractions
use crate::{
    config::QuicConfig,
    error::{TransportError, TransportResult},
    protocol::StreamedMessage,
};
use async_trait::async_trait;
use bincode::{
    config,
    serde::{decode_from_slice, encode_to_vec},
};
use quinn::{RecvStream, SendStream};
use rpc_io::{reader::MessageReader, writer::MessageWriter};
use serde::{Deserialize, Serialize};

pub trait Config {
    fn config(&self) -> &QuicConfig;
    fn update_config(&mut self, config: QuicConfig) -> &mut QuicConfig;
}

#[async_trait]
pub trait Sendable: Send + Config {
    fn writer(&mut self) -> &mut MessageWriter<SendStream>;
    async fn close(self) -> TransportResult<()>;

    // Send a stream message
    async fn send_message<T: Serialize + Send>(
        &mut self,
        message: StreamedMessage<T>,
    ) -> TransportResult<()> {
        // Tech Debt: handle ProtocolError properly
        message
            .validate(self.config())
            .map_err(|_| TransportError::ProtocolError {
                reason: "".to_string(),
            })?;

        let data = self.serialize_message(message)?;

        // should size check be done on codec layer?
        if data.len() > self.config().max_message_size {
            return Err(TransportError::MessageTooLarge {
                size: data.len(),
                limit: self.config().max_message_size,
            });
        }

        self.writer()
            .write_message(&data)
            .await
            .map_err(|e| TransportError::IOError {
                reason: e.to_string(),
            })?;
        Ok(())
    }

    // Tech Debt: Should be coupled with codec crate and add a better error message
    fn serialize_message<T: Serialize>(&self, msg: StreamedMessage<T>) -> TransportResult<Vec<u8>> {
        let bincode_config = config::standard();
        encode_to_vec(&msg, bincode_config).map_err(|e| TransportError::Internal {
            message: format!("Failed to serialize message: {}", e),
        })
    }
}

#[async_trait]
pub trait Receiveable: Config {
    fn reader(&mut self) -> &mut MessageReader<RecvStream>;

    async fn recv_message<T>(&mut self) -> TransportResult<StreamedMessage<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let data = self
            .reader()
            .read_message()
            .await
            .map_err(|e| TransportError::IOError {
                reason: e.to_string(),
            })?;

        let message = self.deserialize_bytes(data)?;
        Ok(message)
    }

    // Implement Stream deserialization logic here later from codec
    fn deserialize_bytes<T>(&self, data: Vec<u8>) -> TransportResult<StreamedMessage<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let bincode_config = config::standard();

        // Explicitly specify the type to help the compiler
        let message: StreamedMessage<T> = decode_from_slice(&data, bincode_config)
            .map_err(|e| TransportError::Internal {
                message: format!("Failed to deserialize: {}", e),
            })?
            .0; // decode_from_slice returns (T, usize)

        Ok(message)
    }
}
