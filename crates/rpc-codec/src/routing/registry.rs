use crate::protocol::constants::method_ids;
use crate::routing::handler::ErasedHandler;
use std::collections::HashMap;
use std::sync::Arc;

/// Registry of all RPC method handlers
#[derive(Clone)]
pub struct HandlerRegistry {
    handlers: HashMap<u16, Arc<dyn ErasedHandler>>,
}

impl HandlerRegistry {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// Register a handler for a specific method ID
    pub fn register<H: ErasedHandler>(&mut self, method_id: u16, handler: H) {
        self.handlers.insert(method_id, Arc::new(handler));
    }

    /// Get handler for a method ID
    pub fn get_handler(&self, method_id: u16) -> Option<Arc<dyn ErasedHandler>> {
        self.handlers.get(&method_id).cloned()
    }

    /// Get all registered method IDs
    pub fn registered_methods(&self) -> Vec<u16> {
        self.handlers.keys().copied().collect()
    }

    /// Check if a method is registered
    pub fn has_method(&self, method_id: u16) -> bool {
        self.handlers.contains_key(&method_id)
    }

    /// Remove a handler
    pub fn unregister(&mut self, method_id: u16) -> Option<Arc<dyn ErasedHandler>> {
        self.handlers.remove(&method_id)
    }
}

impl Default for HandlerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating a registry with common handlers
pub struct HandlerRegistryBuilder {
    registry: HandlerRegistry,
}

impl HandlerRegistryBuilder {
    pub fn new() -> Self {
        Self {
            registry: HandlerRegistry::new(),
        }
    }

    /// Register a handler with builder pattern
    pub fn with_handler<H: ErasedHandler>(mut self, method_id: u16, handler: H) -> Self {
        self.registry.register(method_id, handler);
        self
    }

    /// Build the final registry
    pub fn build(self) -> HandlerRegistry {
        self.registry
    }
}

impl Default for HandlerRegistryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Macro to make registration easier
#[macro_export]
macro_rules! register_handlers {
    ($registry:expr, { $($method_id:expr => $handler:expr),* $(,)? }) => {
        $(
            $registry.register($method_id, $handler);
        )*
    };
}
