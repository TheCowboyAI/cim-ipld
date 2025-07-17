//! Codec registry and implementations for CIM-IPLD
//!
//! This module provides codec support for encoding and decoding content
//! in various formats compatible with IPLD.
//!
//! # Example
//!
//! ```
//! use cim_ipld::codec::{CodecRegistry, CimCodec};
//! use cim_ipld::{DagJsonCodec, DagCborCodec};
//! 
//! // Create a codec registry with standard codecs pre-registered
//! let registry = CodecRegistry::new();
//! 
//! // Get codec information
//! let dag_json = DagJsonCodec;
//! assert_eq!(dag_json.code(), 0x0129);
//! assert_eq!(dag_json.name(), "dag-json");
//! 
//! let dag_cbor = DagCborCodec;
//! assert_eq!(dag_cbor.code(), 0x71);
//! assert_eq!(dag_cbor.name(), "dag-cbor");
//! ```

pub mod ipld_codecs;

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
    standard_codecs: HashMap<u64, Arc<dyn CimCodec>>,
}

impl CodecRegistry {
    /// Create a new registry with base codecs
    pub fn new() -> Self {
        let mut registry = Self {
            codecs: HashMap::new(),
            standard_codecs: HashMap::new(),
        };

        // Register base codecs
        registry.register_base_codecs();
        registry
    }

    /// Register base codecs for common content types
    fn register_base_codecs(&mut self) {
        // Register standard IPLD codecs
        let _ = ipld_codecs::register_ipld_codecs(self);
        
        // Register CIM-specific JSON codecs
        let _ = ipld_codecs::register_cim_json_codecs(self);
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

    /// Register a standard IPLD codec (no range validation)
    pub fn register_standard(&mut self, codec: Arc<dyn CimCodec>) -> Result<()> {
        let code = codec.code();
        self.standard_codecs.insert(code, codec);
        Ok(())
    }

    /// Get a codec by its code
    pub fn get(&self, code: u64) -> Option<&Arc<dyn CimCodec>> {
        self.codecs.get(&code)
            .or_else(|| self.standard_codecs.get(&code))
    }

    /// Check if a codec is registered
    pub fn contains(&self, code: u64) -> bool {
        self.codecs.contains_key(&code) || self.standard_codecs.contains_key(&code)
    }

    /// Get all registered codec codes
    pub fn codes(&self) -> Vec<u64> {
        let mut codes: Vec<u64> = self.codecs.keys().copied().collect();
        codes.extend(self.standard_codecs.keys().copied());
        codes.sort_unstable();
        codes.dedup();
        codes
    }
}

impl Default for CodecRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test codec implementation
    struct TestCodec {
        code: u64,
        name: String,
    }

    impl CimCodec for TestCodec {
        fn code(&self) -> u64 {
            self.code
        }

        fn name(&self) -> &str {
            &self.name
        }
    }

    #[test]
    fn test_codec_registry_new() {
        let registry = CodecRegistry::new();
        
        // Should have standard codecs pre-registered
        assert!(registry.contains(0x71)); // dag-cbor
        assert!(registry.contains(0x0129)); // dag-json
        assert!(registry.contains(0x55)); // raw
        
        // Should have CIM-specific codecs
        assert!(registry.contains(0x340000)); // alchemist
        assert!(registry.contains(0x340001)); // workflow-graph
    }

    #[test]
    fn test_register_valid_codec() {
        let mut registry = CodecRegistry::new();
        let codec = Arc::new(TestCodec {
            code: 0x300100,
            name: "test-codec".to_string(),
        });

        assert!(registry.register(codec.clone()).is_ok());
        assert!(registry.contains(0x300100));
        
        let retrieved = registry.get(0x300100).unwrap();
        assert_eq!(retrieved.code(), 0x300100);
        assert_eq!(retrieved.name(), "test-codec");
    }

    #[test]
    fn test_register_invalid_codec_range() {
        let mut registry = CodecRegistry::new();
        
        // Test below valid range
        let codec1 = Arc::new(TestCodec {
            code: 0x2FFFFF,
            name: "below-range".to_string(),
        });
        match registry.register(codec1) {
            Err(Error::InvalidCodecRange(code)) => assert_eq!(code, 0x2FFFFF),
            _ => panic!("Expected InvalidCodecRange error"),
        }

        // Test above valid range
        let codec2 = Arc::new(TestCodec {
            code: 0x400000,
            name: "above-range".to_string(),
        });
        match registry.register(codec2) {
            Err(Error::InvalidCodecRange(code)) => assert_eq!(code, 0x400000),
            _ => panic!("Expected InvalidCodecRange error"),
        }
    }

    #[test]
    fn test_register_boundary_codecs() {
        let mut registry = CodecRegistry::new();
        
        // Test minimum valid codec
        let min_codec = Arc::new(TestCodec {
            code: 0x300000,
            name: "min-codec".to_string(),
        });
        assert!(registry.register(min_codec).is_ok());
        assert!(registry.contains(0x300000));

        // Test maximum valid codec
        let max_codec = Arc::new(TestCodec {
            code: 0x3FFFFF,
            name: "max-codec".to_string(),
        });
        assert!(registry.register(max_codec).is_ok());
        assert!(registry.contains(0x3FFFFF));
    }

    #[test]
    fn test_register_duplicate_codec() {
        let mut registry = CodecRegistry::new();
        
        let codec1 = Arc::new(TestCodec {
            code: 0x300200,
            name: "original".to_string(),
        });
        assert!(registry.register(codec1).is_ok());

        // Register duplicate (should overwrite)
        let codec2 = Arc::new(TestCodec {
            code: 0x300200,
            name: "replacement".to_string(),
        });
        assert!(registry.register(codec2).is_ok());
        
        let retrieved = registry.get(0x300200).unwrap();
        assert_eq!(retrieved.name(), "replacement");
    }

    #[test]
    fn test_register_standard_codec() {
        let mut registry = CodecRegistry::new();
        
        // Standard codecs don't have range validation
        let standard = Arc::new(TestCodec {
            code: 0x01, // Outside CIM range
            name: "standard-codec".to_string(),
        });
        
        assert!(registry.register_standard(standard).is_ok());
        assert!(registry.contains(0x01));
    }

    #[test]
    fn test_get_nonexistent_codec() {
        let registry = CodecRegistry::new();
        assert!(registry.get(0x999999).is_none());
    }

    #[test]
    fn test_codes_method() {
        let mut registry = CodecRegistry::new();
        
        // Add some custom codecs
        let codec1 = Arc::new(TestCodec {
            code: 0x300300,
            name: "custom1".to_string(),
        });
        let codec2 = Arc::new(TestCodec {
            code: 0x300301,
            name: "custom2".to_string(),
        });
        
        registry.register(codec1).unwrap();
        registry.register(codec2).unwrap();
        
        let codes = registry.codes();
        
        // Should contain both standard and custom codecs
        assert!(codes.contains(&0x71)); // dag-cbor
        assert!(codes.contains(&0x300300)); // custom1
        assert!(codes.contains(&0x300301)); // custom2
        
        // Should be sorted
        let mut sorted = codes.clone();
        sorted.sort_unstable();
        assert_eq!(codes, sorted);
        
        // Should have no duplicates
        let mut unique = codes.clone();
        unique.dedup();
        assert_eq!(codes.len(), unique.len());
    }

    #[test]
    fn test_codec_trait_implementation() {
        let codec = TestCodec {
            code: 0x300400,
            name: "trait-test".to_string(),
        };
        
        assert_eq!(codec.code(), 0x300400);
        assert_eq!(codec.name(), "trait-test");
    }

    #[test]
    fn test_default_trait() {
        let registry1 = CodecRegistry::new();
        let registry2 = CodecRegistry::default();
        
        // Both should have the same standard codecs
        assert_eq!(registry1.contains(0x71), registry2.contains(0x71));
        assert_eq!(registry1.contains(0x0129), registry2.contains(0x0129));
    }

    #[test]
    fn test_send_sync_bounds() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<CodecRegistry>();
        assert_send_sync::<Arc<dyn CimCodec>>();
    }

    #[test]
    fn test_standard_vs_custom_priority() {
        let mut registry = CodecRegistry::new();
        
        // Register a standard codec
        let standard = Arc::new(TestCodec {
            code: 0x300500,
            name: "standard-version".to_string(),
        });
        registry.register_standard(standard).unwrap();
        
        // Register a custom codec with same code
        let custom = Arc::new(TestCodec {
            code: 0x300500,
            name: "custom-version".to_string(),
        });
        registry.register(custom).unwrap();
        
        // Custom should take priority
        let retrieved = registry.get(0x300500).unwrap();
        assert_eq!(retrieved.name(), "custom-version");
    }
}
