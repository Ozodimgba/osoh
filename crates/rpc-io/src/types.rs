use std::net::SocketAddr;
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    /// Local address of this connection
    pub local_addr: SocketAddr,

    /// Remote peer address
    pub remote_addr: SocketAddr,

    /// When the connection was established
    pub connected_at: SystemTime,

    /// Protocol being used (TCP, QUIC, etc.)
    pub protocol: String,

    /// Connection ID for tracking
    pub connection_id: String,

    /// Optional TLS/encryption info
    pub encryption_info: Option<EncryptionInfo>,

    /// Connection-specific metadata
    pub metadata: std::collections::HashMap<String, String>,
}

// TODO: Protocol has to be an enum
impl ConnectionInfo {
    pub fn new(local_addr: SocketAddr, remote_addr: SocketAddr, protocol: &str) -> Self {
        Self {
            local_addr,
            remote_addr,
            connected_at: SystemTime::now(),
            protocol: protocol.to_string(),
            connection_id: uuid::Uuid::new_v4().to_string(),
            encryption_info: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    pub fn connection_duration(&self) -> Duration {
        self.connected_at.elapsed().unwrap_or(Duration::ZERO)
    }
}

#[derive(Debug, Clone)]
pub struct EncryptionInfo {
    pub cipher_suite: String,
    pub protocol_version: String,
    pub peer_certificates: Vec<String>,
}
