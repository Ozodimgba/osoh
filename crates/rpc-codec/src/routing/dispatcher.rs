use crate::RpcCodec; // Add this import
use crate::codec::BincodeCodec;
use crate::error::RpcError;
use crate::protocol::{MessageType, RpcMessage};
use crate::routing::{HandlerRegistry, RequestContext};

/// Main request dispatcher
pub struct MessageDispatcher {
    registry: HandlerRegistry,
    codec: BincodeCodec,
}

impl MessageDispatcher {
    pub fn new(registry: HandlerRegistry) -> Self {
        Self {
            registry,
            codec: BincodeCodec::new(),
        }
    }

    /// Dispatch a raw message to the appropriate handler
    pub async fn dispatch(&self, raw_bytes: &[u8], context: RequestContext) -> Vec<u8> {
        // println!("DISPATCH: Processing {} bytes", raw_bytes.len());
        // println!(
        //     "First 20 bytes: {:?}",
        //     &raw_bytes[..std::cmp::min(20, raw_bytes.len())]
        // );

        let envelope = match self.decode_envelope(raw_bytes) {
            Ok(env) => {
                // println!(
                //     "DISPATCH: Successfully decoded envelope - method_id={}, request_id={}",
                //     env.method_id, env.request_id
                // );
                env
            }
            Err(error) => {
                // println!("❌ DISPATCH: Failed to decode envelope: {}", error);
                // println!("❌ This is why you're getting error response bytes!");
                return self.encode_error_response(0, 0, error);
            }
        };

        // Look up the handler
        let handler = match self.registry.get_handler(envelope.method_id) {
            Some(h) => {
                // println!(
                //     "DISPATCH: Found handler for method_id={}",
                //     envelope.method_id
                // );
                h
            }
            None => {
                // println!(
                //     "DISPATCH: No handler found for method_id={}",
                //     envelope.method_id
                // );
                let error = RpcError::method_not_found(&format!(
                    "Method ID {} not found",
                    envelope.method_id
                ));
                return self.encode_error_response(envelope.method_id, envelope.request_id, error);
            }
        };

        // Pass the ORIGINAL raw bytes to the handler
        // println!("DISPATCH: Calling handler.handle_raw()");
        match handler.handle_raw(raw_bytes, context).await {
            Ok(response_bytes) => {
                // println!(
                //     "DISPATCH: Handler succeeded, returning {} bytes",
                //     response_bytes.len()
                // );
                response_bytes
            }
            Err(error) => {
                // println!("❌ DISPATCH: Handler failed: {}", error);
                self.encode_error_response(envelope.method_id, envelope.request_id, error)
            }
        }
    }

    /// Decode just the envelope to get routing info
    fn decode_envelope(&self, raw_bytes: &[u8]) -> Result<EnvelopeInfo, RpcError> {
        if raw_bytes.len() < 11 {
            // version(1) + method_id(2) + request_id(8) = minimum 11 bytes
            return Err(RpcError::invalid_request("Message too short"));
        }

        let config = bincode::config::standard().with_little_endian();
        let mut offset = 0;

        // Parse version (u8)
        let (version, consumed): (u8, usize) =
            bincode::serde::decode_from_slice(&raw_bytes[offset..], config).map_err(|e| {
                RpcError::invalid_request(&format!("Failed to decode version: {}", e))
            })?;
        offset += consumed;

        // Parse method_id (u16)
        let (method_id, consumed): (u16, usize) =
            bincode::serde::decode_from_slice(&raw_bytes[offset..], config).map_err(|e| {
                RpcError::invalid_request(&format!("Failed to decode method_id: {}", e))
            })?;
        offset += consumed;

        // Parse request_id (u64)
        let (request_id, consumed): (u64, usize) =
            bincode::serde::decode_from_slice(&raw_bytes[offset..], config).map_err(|e| {
                RpcError::invalid_request(&format!("Failed to decode request_id: {}", e))
            })?;
        offset += consumed;

        // Parse message_type (MessageType enum)
        let (message_type, _): (MessageType, usize) =
            bincode::serde::decode_from_slice(&raw_bytes[offset..], config).map_err(|e| {
                RpcError::invalid_request(&format!("Failed to decode message_type: {}", e))
            })?;

        Ok(EnvelopeInfo {
            method_id,
            request_id,
            message_type,
        })
    }

    /// Extract just the payload bytes from the full message
    fn extract_payload(&self, raw_bytes: &[u8]) -> Result<Vec<u8>, RpcError> {
        let envelope: RpcMessage<serde_json::Value> = self
            .codec
            .decode(raw_bytes)
            .map_err(|e| RpcError::invalid_request(&format!("Invalid message format: {}", e)))?;

        // Re-encode just the payload
        self.codec
            .encode(&envelope.payload)
            .map_err(|e| RpcError::internal_error(&format!("Failed to extract payload: {}", e)))
    }

    /// Encode a successful response
    fn encode_success_response(
        &self,
        method_id: u16,
        request_id: u64,
        response_bytes: Vec<u8>,
    ) -> Vec<u8> {
        // Decode the response to wrap it in an envelope
        let response_payload =
            serde_json::Value::String(format!("Binary response ({} bytes)", response_bytes.len()));

        let envelope = RpcMessage::new_response(method_id, request_id, response_payload); // Fixed

        self.codec.encode(&envelope).unwrap_or_else(|_| {
            // Fallback error response if we can't encode
            self.encode_error_response(
                method_id,
                request_id,
                RpcError::internal_error("Failed to encode response"),
            )
        })
    }

    /// Encode an error response
    fn encode_error_response(&self, method_id: u16, request_id: u64, error: RpcError) -> Vec<u8> {
        let envelope = RpcMessage::new_error(method_id, request_id, error); // Fixed

        self.codec.encode(&envelope).unwrap_or_else(|_| {
            // If we can't even encode an error, return minimal error bytes
            b"Internal error: failed to encode error response".to_vec()
        })
    }
}

/// Minimal envelope info for routing decisions
#[derive(Debug, Clone)]
struct EnvelopeInfo {
    method_id: u16,
    request_id: u64,
    message_type: MessageType,
}
