use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcMessage<T> {
    pub version: u8,
    pub method_id: u16,
    pub request_id: u64,
    pub message_type: MessageType,
    pub payload: T,
}

// Types of messages in the RPC protocol
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MessageType {
    Request,
    Response,
    Notification,
    Error,
    Subscribe,
    Unsubscribe,
}

impl<T> RpcMessage<T> {
    pub fn new_request(method_id: u16, request_id: u64, payload: T) -> Self {
        RpcMessage {
            version: 1,
            method_id,
            request_id,
            message_type: MessageType::Request,
            payload,
        }
    }

    pub fn new_response(method_id: u16, request_id: u64, payload: T) -> Self {
        RpcMessage {
            version: 1,
            method_id,
            request_id,
            message_type: MessageType::Response,
            payload,
        }
    }

    pub fn new_error(method_id: u16, request_id: u64, error: T) -> Self {
        RpcMessage {
            version: 1,
            method_id,
            request_id,
            message_type: MessageType::Error,
            payload: error,
        }
    }

    pub fn new_subscribe(method_id: u16, request_id: u64, topic: T) -> Self {
        RpcMessage {
            version: 1,
            method_id,
            request_id,
            message_type: MessageType::Subscribe,
            payload: topic,
        }
    }

    pub fn new_unsubscribe(request_id: u64, topic: T) -> Self {
        RpcMessage {
            version: 1,
            method_id: 0,
            request_id,
            message_type: MessageType::Unsubscribe,
            payload: topic,
        }
    }

    pub fn new_notification(method_id: u16, payload: T) -> Self {
        RpcMessage {
            version: 1,
            method_id,
            request_id: 0, // Notifications don't need request IDs
            message_type: MessageType::Notification,
            payload,
        }
    }
}
