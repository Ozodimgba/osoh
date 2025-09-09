use crate::types::ConnectionInfo;
use async_trait::async_trait;

#[async_trait]
pub trait MessageReader {
    // Fields and methods for the MessageReader struct
    async fn read_message(&mut self) -> std::io::Result<Vec<u8>>;
    fn try_read_message(&mut self) -> std::io::Result<Option<Vec<u8>>>;
}

#[async_trait]
pub trait MessageWriter {
    async fn write_message(&mut self, message: &[u8]) -> std::io::Result<()>;
    async fn flush(&mut self) -> std::io::Result<()>;
}

pub trait MessageStream: MessageReader + MessageWriter {
    fn connection_info(&self) -> &ConnectionInfo;
}
