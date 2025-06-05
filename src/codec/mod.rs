//! Codec registry and implementations for CIM-IPLD

use crate::{Error, Result};
use std::collections::HashMap;
use std::sync::Arc;

/// Trait for CIM codecs
pub trait CimCodec: Send + Sync {
    /// Unique codec identifier (0x300000-0x3FFFFF range)
    fn code(&self) -> u64;

    /// Human-readable name for the codec
    fn name(&self) -> &str;
}

/// Registry for CIM codecs
pub struct CodecRegistry {
    codecs: HashMap<u64, Arc<dyn CimCodec>>,
}

impl CodecRegistry {
    /// Create a new registry with base codecs
    pub fn new() -> Self {
        let mut registry = Self {
            codecs: HashMap::new(),
        };

        // Register base codecs
        registry.register_base_codecs();
        registry
    }

    /// Register base codecs for common content types
    fn register_base_codecs(&mut self) {
        // These would be implemented as needed
        // For now, we'll use the default JSON encoding
    }

    /// Register a custom codec
    pub fn register(&mut self, codec: Arc<dyn CimCodec>) -> Result<()> {
        let code = codec.code();

        // Validate codec range
        if !(0x300000..=0x3FFFFF).contains(&code) {
            return Err(Error::InvalidCodecRange(code));
        }

        self.codecs.insert(code, codec);
        Ok(())
    }

    /// Get a codec by its code
    pub fn get(&self, code: u64) -> Option<&Arc<dyn CimCodec>> {
        self.codecs.get(&code)
    }

    /// Check if a codec is registered
    pub fn contains(&self, code: u64) -> bool {
        self.codecs.contains_key(&code)
    }

    /// Get all registered codec codes
    pub fn codes(&self) -> Vec<u64> {
        self.codecs.keys().copied().collect()
    }
}

impl Default for CodecRegistry {
    fn default() -> Self {
        Self::new()
    }
}
