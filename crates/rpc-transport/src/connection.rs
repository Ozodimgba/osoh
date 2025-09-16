use crate::{QuicConfig, QuicStream, TransportError};
use quinn::{Connecting, Connection, Endpoint, VarInt};

pub struct QuicConnection {
    connection: Connection,
    config: QuicConfig,
    _endpoint: Option<Endpoint>,
}

impl QuicConnection {
    /// Connect to a QUIC server as a client
    pub async fn connect(
        server_addr: std::net::SocketAddr,
        server_name: &str,
        config: QuicConfig,
    ) -> Result<Self, TransportError> {
        let client_config =
            config
                .to_client_config()
                .map_err(|e| TransportError::ConnectionFailed {
                    reason: format!("Failed to create client config: {}", e),
                })?;

        let mut endpoint = Endpoint::client("0.0.0.0:0".parse().unwrap()).map_err(|e| {
            TransportError::ConnectionFailed {
                reason: format!("Failed to create endpoint: {}", e),
            }
        })?;

        endpoint.set_default_client_config(client_config);

        let connecting = endpoint.connect(server_addr, server_name).map_err(|e| {
            TransportError::ConnectionFailed {
                reason: format!("Failed to connect to server: {}", e),
            }
        })?;

        let connection = connecting
            .await
            .map_err(|e| TransportError::ConnectionFailed {
                reason: format!("Handshake failed: {}", e),
            })?;

        Ok(Self {
            connection,
            config,
            _endpoint: Some(endpoint),
        })
    }

    // Servers need this to accept incoming streams
    pub async fn accept_stream(&self) -> Result<Option<QuicStream>, TransportError> {
        match self.connection.accept_bi().await {
            Ok((send, recv)) => Ok(Some(QuicStream::new(send, recv))),
            Err(quinn::ConnectionError::ApplicationClosed(_)) => Ok(None),
            Err(e) => Err(TransportError::StreamError {
                reason: format!("Failed to accept stream: {}", e),
            }),
        }
    }

    /// Create connection from incoming server connection
    pub fn from_incoming(connection: Connection, config: QuicConfig) -> Self {
        Self {
            connection,
            config,
            _endpoint: None,
        }
    }

    pub async fn from_connecting(
        connecting: Connecting,
        config: QuicConfig,
    ) -> Result<Self, TransportError> {
        let connection = connecting
            .await
            .map_err(|e| TransportError::ConnectionFailed {
                reason: format!("Handshake failed: {}", e),
            })?;

        Ok(Self {
            connection,
            config,
            _endpoint: None,
        })
    }

    /// Open a new bidirectional stream
    pub async fn open_stream(&self) -> Result<QuicStream, TransportError> {
        let (send, recv) =
            self.connection
                .open_bi()
                .await
                .map_err(|e| TransportError::StreamError {
                    reason: format!("Failed to open stream: {}", e),
                })?;

        Ok(QuicStream::new(send, recv))
    }

    pub fn is_connected(&self) -> bool {
        self.connection.close_reason().is_none()
    }

    /// Get remote peer address
    pub fn remote_address(&self) -> std::net::SocketAddr {
        self.connection.remote_address()
    }

    /// Get connection statistics
    pub fn stats(&self) -> quinn::ConnectionStats {
        self.connection.stats()
    }

    /// Get the maximum message size for this connection
    pub fn max_message_size(&self) -> usize {
        self.config.max_message_size
    }

    /// Gracefully close the connection
    pub fn close(&self, reason: &str) {
        self.connection
            .close(VarInt::from_u32(0), reason.as_bytes());
    }

    /// Wait for the connection to be fully closed
    pub async fn closed(&self) -> quinn::ConnectionError {
        self.connection.closed().await
    }

    /// Get a reference to the underlying Quinn connection
    pub fn inner(&self) -> &Connection {
        &self.connection
    }

    /// Get the QUIC configuration used for this connection
    pub fn config(&self) -> &QuicConfig {
        &self.config
    }
}

// Healthcheck implementation

impl QuicConnection {
    pub async fn ping(&self) -> Result<(), TransportError> {
        let stream = self.open_stream().await?;
        drop(stream); // Auto-closes
        Ok(())
    }

    // Get current stream count
    pub fn active_streams(&self) -> u64 {
        self.stats().frame_tx.max_streams_bidi
    }
}

impl Drop for QuicConnection {
    fn drop(&mut self) {
        if self.is_connected() {
            self.connection
                .close(VarInt::from_u32(0), b"Connection dropped");
        }
    }
}
