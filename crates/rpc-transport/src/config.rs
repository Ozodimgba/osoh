use quinn::{ClientConfig, ServerConfig, TransportConfig, VarInt};
use std::{sync::Arc, time::Duration};

/// QUIC transport configuration that works with both client and server
///
/// QUIC settings that gets converted to Quinn's various config types as needed.
#[derive(Debug, Clone)]
pub struct QuicConfig {
    // Flow control limits
    pub max_message_size: usize,  // For protocol validation
    pub max_stream_data: u64,     // Per-stream flow control
    pub max_connection_data: u64, // Total connection flow control

    // Stream limits
    pub max_concurrent_streams: u32, // How many streams can be open
    pub max_streams_bidi: u64,       // Max bidirectional streams
    pub max_streams_uni: u64,        // Max unidirectional streams

    // Timeout settings
    pub max_idle_timeout: Duration,  // Connection idle timeout
    pub handshake_timeout: Duration, // TLS handshake timeout

    // Performance settings
    pub initial_mtu: u16,  // Starting packet size
    pub min_mtu: u16,      // Minimum packet size
    pub enable_0rtt: bool, // Enable 0-RTT optimization

    // Reliability settings
    pub keep_alive_interval: Option<Duration>, // Keep connection alive
    pub max_retries: u32,                      // Max retransmission attempts
}

impl Default for QuicConfig {
    fn default() -> Self {
        Self {
            max_message_size: 16 * 1024 * 1024,    // 16MB
            max_stream_data: 16 * 1024 * 1024,     // 16MB per stream
            max_connection_data: 32 * 1024 * 1024, // 32MB total (allow multiple streams)
            max_concurrent_streams: 100,
            max_streams_bidi: 100,
            max_streams_uni: 100,
            max_idle_timeout: Duration::from_secs(30),
            handshake_timeout: Duration::from_secs(10),
            initial_mtu: 1200,
            min_mtu: 536,
            enable_0rtt: true,
            keep_alive_interval: Some(Duration::from_secs(10)),
            max_retries: 3,
        }
    }
}

impl QuicConfig {
    /// Create a new QuicConfig with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Convert to Quinn's TransportConfig
    pub fn to_transport_config(&self) -> TransportConfig {
        let mut config = TransportConfig::default();

        // Flow control settings
        config
            .receive_window(VarInt::from_u64(self.max_connection_data).unwrap())
            .stream_receive_window(VarInt::from_u64(self.max_stream_data).unwrap());

        // Stream limits
        config
            .max_concurrent_uni_streams(VarInt::from_u32(self.max_concurrent_streams).into())
            .max_concurrent_bidi_streams(VarInt::from_u32(self.max_concurrent_streams).into());

        // Timeout settings
        config.max_idle_timeout(Some(self.max_idle_timeout.try_into().unwrap()));

        // Keep alive
        if let Some(interval) = self.keep_alive_interval {
            config.keep_alive_interval(Some(interval));
        }

        // MTU settings
        config.initial_mtu(self.initial_mtu).min_mtu(self.min_mtu);

        config
    }

    /// provide the certificate chain and private key when calling this method
    pub fn to_server_config(
        &self,
        cert_chain: Vec<rustls::pki_types::CertificateDer<'static>>,
        private_key: rustls::pki_types::PrivateKeyDer<'static>,
    ) -> Result<ServerConfig, Box<dyn std::error::Error>> {
        // Create rustls server config with defaults
        let rustls_config = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(cert_chain, private_key)?;

        // Convert to Quinn's crypto format
        let quic_crypto_config = quinn::crypto::rustls::QuicServerConfig::try_from(rustls_config)?;

        // Create Quinn server config
        let transport = Arc::new(self.to_transport_config());
        let mut server_config = ServerConfig::with_crypto(Arc::new(quic_crypto_config));
        server_config.transport_config(transport);

        Ok(server_config)
    }

    /// Convert to Quinn's ClientConfig with basic TLS setup
    pub fn to_client_config(&self) -> Result<ClientConfig, Box<dyn std::error::Error>> {
        // Tech debt
        let rustls_config = rustls::ClientConfig::builder()
            .with_root_certificates(rustls::RootCertStore::from_iter(
                webpki_roots::TLS_SERVER_ROOTS.iter().cloned(),
            ))
            .with_no_client_auth();

        // Convert to Quinn's crypto format
        let quic_crypto_config = quinn::crypto::rustls::QuicClientConfig::try_from(rustls_config)?;

        // Create Quinn client config
        let transport = Arc::new(self.to_transport_config());
        let mut client_config = ClientConfig::new(Arc::new(quic_crypto_config));
        client_config.transport_config(transport);

        Ok(client_config)
    }

    // Builder pattern methods
    pub fn with_message_size(mut self, size: usize) -> Self {
        self.max_message_size = size;
        self
    }

    pub fn with_stream_data(mut self, size: u64) -> Self {
        self.max_stream_data = size;
        self
    }

    pub fn with_connection_data(mut self, size: u64) -> Self {
        self.max_connection_data = size;
        self
    }

    pub fn with_max_streams(mut self, count: u32) -> Self {
        self.max_concurrent_streams = count;
        self.max_streams_bidi = count as u64;
        self.max_streams_uni = count as u64;
        self
    }

    pub fn with_idle_timeout(mut self, timeout: Duration) -> Self {
        self.max_idle_timeout = timeout;
        self
    }

    pub fn with_keep_alive(mut self, interval: Option<Duration>) -> Self {
        self.keep_alive_interval = interval;
        self
    }

    pub fn with_0rtt(mut self, enable: bool) -> Self {
        self.enable_0rtt = enable;
        self
    }

    pub fn with_mtu_range(mut self, initial: u16, min: u16) -> Self {
        self.initial_mtu = initial;
        self.min_mtu = min;
        self
    }

    // Preset configurations
    pub fn high_throughput() -> Self {
        Self::default()
            .with_message_size(64 * 1024 * 1024) // 64MB messages
            .with_stream_data(64 * 1024 * 1024) // 64MB per stream
            .with_connection_data(256 * 1024 * 1024) // 256MB total
            .with_max_streams(500) // More concurrent streams
            .with_idle_timeout(Duration::from_secs(60))
    }

    pub fn low_latency() -> Self {
        Self::default()
            .with_message_size(1024 * 1024) // 1MB messages
            .with_stream_data(4 * 1024 * 1024) // 4MB per stream
            .with_connection_data(16 * 1024 * 1024) // 16MB total
            .with_max_streams(50) // Fewer streams
            .with_idle_timeout(Duration::from_secs(15))
            .with_keep_alive(Some(Duration::from_secs(5)))
    }

    pub fn minimal() -> Self {
        Self::default()
            .with_message_size(256 * 1024) // 256KB messages
            .with_stream_data(1024 * 1024) // 1MB per stream
            .with_connection_data(4 * 1024 * 1024) // 4MB total
            .with_max_streams(10) // Very few streams
            .with_idle_timeout(Duration::from_secs(10))
            .with_keep_alive(None) // No keep alive
    }
}
