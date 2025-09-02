use crate::RpcCodec;
use crate::error::RpcError;
use crate::protocol::RpcMessage;
use crate::routing::context::RequestContext; // Remove handler::ErasedHandler import
use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};
use std::future::Future;
use std::pin::Pin;

pub type HandlerResult<T> = Result<T, RpcError>;
pub type BoxedHandlerFuture<T> = Pin<Box<dyn Future<Output = HandlerResult<T>> + Send + 'static>>;

/// Trait that all RPC method handlers must implement
#[async_trait]
pub trait RpcHandler: Send + Sync + 'static {
    // Add 'static bound
    /// The request type this handler expects
    type Request: DeserializeOwned + Send + 'static;

    /// The response type this handler returns
    type Response: Serialize + Send + 'static;

    /// Handle the RPC request
    async fn handle(
        &self,
        request: Self::Request,
        context: RequestContext,
    ) -> HandlerResult<Self::Response>;

    /// Method name (for debugging/logging)
    fn method_name(&self) -> &'static str;
}

/// Helper trait for type-erased handlers
#[async_trait]
pub trait ErasedHandler: Send + Sync + 'static {
    /// Handle raw bytes and return raw bytes
    async fn handle_raw(
        &self,
        request_bytes: &[u8],
        context: RequestContext,
    ) -> HandlerResult<Vec<u8>>;

    /// Method name
    fn method_name(&self) -> &'static str;
}

/// Wrapper to make any RpcHandler into an ErasedHandler
pub struct HandlerWrapper<H: RpcHandler> {
    handler: H,
    codec: crate::codec::BincodeCodec,
}

impl<H: RpcHandler> HandlerWrapper<H> {
    pub fn new(handler: H) -> Self {
        Self {
            handler,
            codec: crate::codec::BincodeCodec::new(),
        }
    }

    // Optional: constructor with custom codec
    pub fn with_codec(handler: H, codec: crate::codec::BincodeCodec) -> Self {
        Self { handler, codec }
    }
}

#[async_trait]
impl<H: RpcHandler> ErasedHandler for HandlerWrapper<H> {
    async fn handle_raw(
        &self,
        request_bytes: &[u8],
        context: RequestContext,
    ) -> HandlerResult<Vec<u8>> {
        // Decode the full RpcMessage (preserves type information)
        let request_message: RpcMessage<H::Request> = self.codec.decode(request_bytes)?;

        // Call handler with just the payload
        let response = self
            .handler
            .handle(request_message.payload, context)
            .await?;

        // Create complete response message
        let response_message = RpcMessage::new_response(
            request_message.method_id,
            request_message.request_id,
            response,
        );

        // Return complete response bytes
        self.codec.encode(&response_message)
    }

    fn method_name(&self) -> &'static str {
        self.handler.method_name()
    }
}
