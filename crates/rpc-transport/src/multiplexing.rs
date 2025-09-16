//! Stream multiplexing for QUIC RPC transport
//!
//! This module provides QUIC-specific stream management on top of rpc-io abstractions.
//! It manages multiple streams over a single QUIC connection while using generic
//! rpc-io traits for actual I/O operations.
//! Simplified QUIC stream multiplexing
//!
//! Provides priority-based stream pooling on top of Quinn QUIC connections.

use crate::{
    config::QuicConfig,
    error::TransportError,
    protocol::MessagePriority,
};
use rpc_codec::RpcCodec;
use rpc_io::traits::{MessageReader, MessageWriter};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, warn};

/// A stream that can send and receive messages
pub trait MultiplexedStream: MessageReader + MessageWriter + Send + 'static {
    /// Close this stream gracefully
    fn close(self) -> impl std::future::Future<Output = Result<(), std::io::Error>> + Send;
}

/// Configuration for the multiplexer
#[derive(Debug, Clone)]
pub struct MultiplexConfig {
    pub max_streams_per_priority: usize,
    pub idle_timeout: Duration,
    pub operation_timeout: Duration,
    pub quic_config: QuicConfig,
}

impl Default for MultiplexConfig {
    fn default() -> Self {
        Self {
            max_streams_per_priority: 10,
            idle_timeout: Duration::from_secs(30),
            operation_timeout: Duration::from_secs(5),
            quic_config: QuicConfig::default(),
        }
    }
}

/// Trait for creating new streams
#[async_trait::async_trait]
pub trait StreamConnector: Send + Sync + 'static {
    type Stream: MultiplexedStream;

    async fn open_stream(&self) -> Result<Self::Stream, TransportError>;
    fn is_connected(&self) -> bool;
}

/// Commands for the multiplexer
#[derive(Debug)]
enum Command {
    SendMessage {
        message: Vec<u8>,
        priority: MessagePriority,
        response: oneshot::Sender<Result<(), TransportError>>,
    },
    Shutdown {
        response: oneshot::Sender<Result<(), TransportError>>,
    },
}

/// Pool entry for a managed stream
struct PooledStream<S> {
    stream: S,
    last_used: Instant,
}

impl<S> PooledStream<S> {
    fn new(stream: S) -> Self {
        Self {
            stream,
            last_used: Instant::now(),
        }
    }

    fn is_idle(&self, threshold: Duration) -> bool {
        self.last_used.elapsed() > threshold
    }

    fn mark_used(&mut self) {
        self.last_used = Instant::now();
    }
}

/// Simplified stream multiplexer
pub struct StreamMultiplexer<C>
where
    C: RpcCodec,
{
    command_tx: mpsc::Sender<Command>,
    codec: C,
    config: MultiplexConfig,
}

impl<C> StreamMultiplexer<C>
where
    C: RpcCodec,
{
    pub async fn new<S, Connector>(
        connector: Arc<Connector>,
        codec: C,
        config: MultiplexConfig,
    ) -> Result<Self, TransportError> 
    where
        S: MultiplexedStream,
        Connector: StreamConnector<Stream = S>,
    {
        let (command_tx, command_rx) = mpsc::channel(100);

        // Start coordinator
        tokio::spawn(Self::run_coordinator::<S, Connector>(command_rx, connector, config.clone()));

        let multiplexer = Self {
            command_tx,
            codec,
            config,
        };

        // Start cleanup task
        multiplexer.start_cleanup().await;
        Ok(multiplexer)
    }

    /// Send a message with automatic stream management
    pub async fn send_message<T>(&self, message: T, priority: MessagePriority) -> Result<(), TransportError>
    where
        T: serde::Serialize,
    {
        let payload = self.codec.encode(&message)
            .map_err(|e| TransportError::CodecError(e))?;

        let (response_tx, response_rx) = oneshot::channel();
        
        self.command_tx
            .send(Command::SendMessage {
                message: payload,
                priority,
                response: response_tx,
            })
            .await
            .map_err(|_| TransportError::Internal {
                message: "Multiplexer unavailable".into(),
            })?;

        tokio::time::timeout(self.config.operation_timeout, response_rx)
            .await
            .map_err(|_| TransportError::Timeout {
                duration: self.config.operation_timeout,
                operation: "send_message",
            })?
            .map_err(|_| TransportError::Internal {
                message: "Operation failed".into(),
            })?
    }

    /// Shutdown the multiplexer
    pub async fn shutdown(self) -> Result<(), TransportError> {
        let (response_tx, response_rx) = oneshot::channel();

        if self.command_tx.send(Command::Shutdown { response: response_tx }).await.is_err() {
            return Ok(()); // Already shut down
        }

        tokio::time::timeout(self.config.operation_timeout, response_rx)
            .await
            .map_err(|_| TransportError::Timeout {
                duration: self.config.operation_timeout,
                operation: "shutdown",
            })?
            .map_err(|_| TransportError::Internal {
                message: "Shutdown failed".into(),
            })?
    }

    async fn start_cleanup(&self) {
        // Simplified: just send periodic cleanup signals
        let command_tx = self.command_tx.clone();
        let interval = self.config.idle_timeout;

        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            loop {
                ticker.tick().await;
                // In a real implementation, send a cleanup command
                // For now, let idle streams get cleaned up on next use
            }
        });
    }

    async fn run_coordinator<S, Connector>(
        mut command_rx: mpsc::Receiver<Command>,
        connector: Arc<Connector>,
        config: MultiplexConfig,
    ) where
        S: MultiplexedStream,
        Connector: StreamConnector<Stream = S>,
    {
        debug!("Starting stream multiplexer coordinator");

        // Simple stream pools by priority
        let mut pools: HashMap<MessagePriority, Vec<PooledStream<S>>> = HashMap::new();
        
        // Initialize pools
        for priority in [MessagePriority::Low, MessagePriority::Normal, MessagePriority::High, MessagePriority::Critical] {
            pools.insert(priority, Vec::new());
        }

        while let Some(command) = command_rx.recv().await {
            match command {
                Command::SendMessage { message, priority, response } => {
                    let result = Self::handle_send_message::<S, Connector>(&mut pools, &connector, message, priority, &config).await;
                    let _ = response.send(result);
                }
                Command::Shutdown { response } => {
                    let result = Self::handle_shutdown::<S>(&mut pools).await;
                    let _ = response.send(result);
                    break;
                }
            }
        }

        debug!("Multiplexer coordinator shutdown");
    }

    async fn handle_send_message<S, Connector>(
        pools: &mut HashMap<MessagePriority, Vec<PooledStream<S>>>,
        connector: &Arc<Connector>,
        message: Vec<u8>,
        priority: MessagePriority,
        config: &MultiplexConfig,
    ) -> Result<(), TransportError> 
    where
        S: MultiplexedStream,
        Connector: StreamConnector<Stream = S>,
    {
        // Get or create stream
        let mut stream = 
            Self::get_or_create_stream::<S, Connector>(pools, connector, priority.clone(), config).await?; // TECH DEBT: clone is bad code here

        // Send message
        match stream.write_message(&message).await {
            Ok(_) => {
                // Return healthy stream to pool
                Self::return_stream_to_pool::<S>(pools, stream, priority.clone(), config);
                Ok(())
            }
            Err(e) => {
                // Don't return broken stream, just drop it
                Err(TransportError::IOError { reason: e.to_string() })
            }
        }
    }

    async fn get_or_create_stream<S, Connector>(
        pools: &mut HashMap<MessagePriority, Vec<PooledStream<S>>>,
        connector: &Arc<Connector>,
        priority: MessagePriority,
        config: &MultiplexConfig,
    ) -> Result<S, TransportError> 
    where
        S: MultiplexedStream,
        Connector: StreamConnector<Stream = S>,
    {
        let pool = pools.get_mut(&priority).expect("Priority pool should exist");

        // Clean up idle streams while we're here
        pool.retain(|p| !p.is_idle(config.idle_timeout));

        // Try to reuse existing stream
        if let Some(mut pooled) = pool.pop() {
            pooled.mark_used();
            debug!("Reused stream for priority {:?}", priority);
            return Ok(pooled.stream);
        }

        // Create new stream
        match connector.open_stream().await {
            Ok(stream) => {
                debug!("Created new stream for priority {:?}", priority);
                Ok(stream)
            }
            Err(e) => {
                warn!("Failed to create stream: {:?}", e);
                Err(e)
            }
        }
    }

    fn return_stream_to_pool<S>(
        pools: &mut HashMap<MessagePriority, Vec<PooledStream<S>>>,
        stream: S,
        priority: MessagePriority,
        config: &MultiplexConfig,
    ) where
        S: MultiplexedStream,
    {
        let pool = pools.get_mut(&priority).expect("Priority pool should exist");

        if pool.len() < config.max_streams_per_priority {
            pool.push(PooledStream::new(stream));
            debug!("Returned stream to pool for priority {:?}", priority);
        } else {
            debug!("Pool full, dropping stream for priority {:?}", priority);
            // Stream gets dropped and closed
        }
    }

    async fn handle_shutdown<S>(
        pools: &mut HashMap<MessagePriority, Vec<PooledStream<S>>>,
    ) -> Result<(), TransportError> 
    where
        S: MultiplexedStream,
    {
        debug!("Shutting down all stream pools");

        for (priority, pool) in pools.iter_mut() {
            let count = pool.len();
            for pooled in pool.drain(..) {
                if let Err(e) = pooled.stream.close().await {
                    warn!("Error closing stream: {:?}", e);
                }
            }
            debug!("Closed {} streams for priority {:?}", count, priority);
        }

        Ok(())
    }
}