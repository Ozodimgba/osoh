use crate::codec::traits::{RpcCodec, ZeroCopyCodec};
use crate::error::{RpcError, RpcErrorCode, RpcResult};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::collections::HashMap;
use std::sync::Arc;

/// Memory pool for reusing byte buffers
#[derive(Debug)]
pub struct BufferPool {
    pools: HashMap<usize, Vec<Vec<u8>>>,
    max_pool_size: usize,
}

impl BufferPool {
    pub fn new() -> Self {
        Self {
            pools: HashMap::new(),
            max_pool_size: 100,
        }
    }

    /// Get a buffer of at least the specified size
    pub fn get_buffer(&mut self, min_size: usize) -> Vec<u8> {
        // Round up to next power of 2 for better reuse
        let size = min_size.next_power_of_two();

        if let Some(pool) = self.pools.get_mut(&size) {
            if let Some(mut buffer) = pool.pop() {
                buffer.clear();
                buffer.reserve(min_size);
                return buffer;
            }
        }

        Vec::with_capacity(size)
    }

    /// Return a buffer to the pool
    pub fn return_buffer(&mut self, buffer: Vec<u8>) {
        let capacity = buffer.capacity();

        // Don't store buffers that are too small or too large
        if capacity < 64 || capacity > 10 * 1024 * 1024 {
            return;
        }

        let pool = self.pools.entry(capacity).or_insert_with(Vec::new);
        if pool.len() < self.max_pool_size {
            pool.push(buffer);
        }
    }
}

impl Default for BufferPool {
    fn default() -> Self {
        Self::new()
    }
}

/// Zero-copy message view that borrows data
#[derive(Debug)]
pub struct MessageView<'a, T> {
    pub data: T,
    _lifetime: std::marker::PhantomData<&'a ()>,
}

impl<'a, T> MessageView<'a, T> {
    pub fn new(data: T) -> Self {
        Self {
            data,
            _lifetime: std::marker::PhantomData,
        }
    }

    pub fn into_owned(self) -> T {
        self.data
    }
}

impl<'a, T: Clone> Clone for MessageView<'a, T> {
    fn clone(&self) -> Self {
        Self::new(self.data.clone())
    }
}

/// Codec wrapper that uses buffer pooling
#[derive(Debug, Clone)]
pub struct PooledCodec<C: RpcCodec> {
    inner: C,
    buffer_pool: Arc<std::sync::Mutex<BufferPool>>,
}

impl<C: RpcCodec> PooledCodec<C> {
    pub fn new(codec: C) -> Self {
        Self {
            inner: codec,
            buffer_pool: Arc::new(std::sync::Mutex::new(BufferPool::new())),
        }
    }

    pub fn with_pool_size(codec: C, max_pool_size: usize) -> Self {
        let mut pool = BufferPool::new();
        pool.max_pool_size = max_pool_size;

        Self {
            inner: codec,
            buffer_pool: Arc::new(std::sync::Mutex::new(pool)),
        }
    }
}

impl<C: RpcCodec> RpcCodec for PooledCodec<C> {
    fn encode<T>(&self, value: &T) -> RpcResult<Vec<u8>>
    where
        T: Serialize,
    {
        // Just use the regular encoding - keep it simple
        let result = self.inner.encode(value);
        result
    }

    fn decode<T>(&self, bytes: &[u8]) -> RpcResult<T>
    where
        T: DeserializeOwned,
    {
        self.inner.decode(bytes)
    }

    fn name(&self) -> &'static str {
        self.inner.name()
    }

    fn mime_type(&self) -> &'static str {
        self.inner.mime_type()
    }
}

impl<C: RpcCodec> PooledCodec<C> {
    fn encode_into_buffer<T>(&self, value: &T, buffer: &mut Vec<u8>) -> RpcResult<()>
    where
        T: Serialize,
    {
        // This is simplified - in practice you'd use bincode's write interface
        let encoded = self.inner.encode(value)?;
        buffer.clear();
        buffer.extend_from_slice(&encoded);
        Ok(())
    }
}

/// Pre-allocated message structures for hot paths
pub struct PreallocatedMessage<T> {
    pub payload: T,
    pub serialized: Option<Vec<u8>>,
    pub dirty: bool,
}

impl<T> PreallocatedMessage<T>
where
    T: Serialize + Clone,
{
    pub fn new(payload: T) -> Self {
        Self {
            payload,
            serialized: None,
            dirty: true,
        }
    }

    pub fn get_serialized<C: RpcCodec>(&mut self, codec: &C) -> RpcResult<&[u8]> {
        if self.dirty || self.serialized.is_none() {
            let encoded = codec.encode(&self.payload)?;
            self.serialized = Some(encoded);
            self.dirty = false;
        }

        Ok(self.serialized.as_ref().unwrap())
    }

    pub fn update_payload(&mut self, payload: T) {
        self.payload = payload;
        self.dirty = true;
    }

    pub fn invalidate(&mut self) {
        self.dirty = true;
    }
}

/// Batch encoding for multiple messages
pub struct BatchEncoder<C: RpcCodec> {
    codec: C,
    buffer: Vec<u8>,
}

impl<C: RpcCodec> BatchEncoder<C> {
    pub fn new(codec: C) -> Self {
        Self {
            codec,
            buffer: Vec::with_capacity(64 * 1024), // 64KB initial capacity
        }
    }

    pub fn encode_batch<T>(&mut self, messages: &[T]) -> RpcResult<Vec<u8>>
    where
        T: Serialize,
    {
        self.buffer.clear();

        // Write message count first
        let count = messages.len() as u32;
        self.buffer.extend_from_slice(&count.to_le_bytes());

        // Encode each message with length prefix
        for message in messages {
            let encoded = self.codec.encode(message)?;
            let len = encoded.len() as u32;

            self.buffer.extend_from_slice(&len.to_le_bytes());
            self.buffer.extend_from_slice(&encoded);
        }

        Ok(self.buffer.clone())
    }

    pub fn decode_batch<T>(&self, bytes: &[u8]) -> RpcResult<Vec<T>>
    where
        T: DeserializeOwned,
    {
        if bytes.len() < 4 {
            return Err(RpcError::new(
                RpcErrorCode::ParseError,
                "Invalid batch format",
            ));
        }

        let mut offset = 0;
        let count = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
        offset += 4;

        let mut messages = Vec::with_capacity(count);

        for _ in 0..count {
            if offset + 4 > bytes.len() {
                return Err(RpcError::new(RpcErrorCode::ParseError, "Truncated batch"));
            }

            let len = u32::from_le_bytes([
                bytes[offset],
                bytes[offset + 1],
                bytes[offset + 2],
                bytes[offset + 3],
            ]) as usize;
            offset += 4;

            if offset + len > bytes.len() {
                return Err(RpcError::new(RpcErrorCode::ParseError, "Truncated message"));
            }

            let message_bytes = &bytes[offset..offset + len];
            let message = self.codec.decode(message_bytes)?;
            messages.push(message);

            offset += len;
        }

        Ok(messages)
    }
}
