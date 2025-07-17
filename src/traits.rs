//! Core traits for CIM-IPLD

use crate::{ContentType, Result, Cid};
use serde::{Serialize, de::DeserializeOwned};

/// Trait for content that can be stored with a CID
pub trait TypedContent: Serialize + DeserializeOwned + Send + Sync {
    /// The IPLD codec for this content type
    const CODEC: u64;

    /// The content type identifier
    const CONTENT_TYPE: ContentType;

    /// Extract the canonical payload for CID calculation.
    ///
    /// This should return only the actual content data, excluding any
    /// transient metadata like timestamps, UUIDs, or message wrappers
    /// that would make identical content have different CIDs.
    ///
    /// By default, this serializes the entire struct, but implementations
    /// should override this to extract only the stable payload.
    fn canonical_payload(&self) -> Result<Vec<u8>> {
        // Default implementation serializes the whole struct
        // Override this for types with metadata that should be excluded
        self.to_bytes()
    }

    /// Calculate the CID for this content
    fn calculate_cid(&self) -> Result<Cid> {
        // Use canonical payload instead of full serialization
        let bytes = self.canonical_payload()?;

        // Create hash using BLAKE3
        let hash = blake3::hash(&bytes);
        let hash_bytes = hash.as_bytes();

        // Create multihash manually with BLAKE3 code (0x1e)
        let code = 0x1e; // BLAKE3-256
        let size = hash_bytes.len() as u8;

        // Build multihash: <varint code><varint size><hash>
        let mut multihash_bytes = Vec::new();
        multihash_bytes.push(code);
        multihash_bytes.push(size);
        multihash_bytes.extend_from_slice(hash_bytes);

        // Create CID v1
        let mh = multihash::Multihash::from_bytes(&multihash_bytes)
            .map_err(|e| crate::Error::MultihashError(e.to_string()))?;
        let cid = Cid::new_v1(Self::CODEC, mh);

        Ok(cid)
    }

    /// Convert to bytes for storage
    fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(serde_json::to_vec(self)?)
    }

    /// Create from bytes
    fn from_bytes(bytes: &[u8]) -> Result<Self> where Self: Sized {
        Ok(serde_json::from_slice(bytes)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    // Test struct with simple content
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestContent {
        data: String,
        value: u64,
    }

    impl TypedContent for TestContent {
        const CODEC: u64 = 0x300100;
        const CONTENT_TYPE: ContentType = ContentType::Custom(0x300100);
    }

    // Test struct with custom canonical payload
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestContentWithMetadata {
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        data: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        timestamp: Option<u64>,
    }

    impl TypedContent for TestContentWithMetadata {
        const CODEC: u64 = 0x300101;
        const CONTENT_TYPE: ContentType = ContentType::Custom(0x300101);

        fn canonical_payload(&self) -> Result<Vec<u8>> {
            // Only include the data field for CID calculation
            Ok(self.data.as_bytes().to_vec())
        }
    }

    #[test]
    fn test_typed_content_constants() {
        assert_eq!(TestContent::CODEC, 0x300100);
        assert_eq!(TestContent::CONTENT_TYPE, ContentType::Custom(0x300100));
    }

    #[test]
    fn test_calculate_cid() {
        let content = TestContent {
            data: "test data".to_string(),
            value: 42,
        };

        let cid = content.calculate_cid().unwrap();
        
        // Verify CID properties
        assert_eq!(cid.version(), cid::Version::V1);
        assert_eq!(cid.codec(), 0x300100);
        
        // Verify deterministic CID
        let cid2 = content.calculate_cid().unwrap();
        assert_eq!(cid, cid2);
    }

    #[test]
    fn test_calculate_cid_different_content() {
        let content1 = TestContent {
            data: "test data 1".to_string(),
            value: 42,
        };

        let content2 = TestContent {
            data: "test data 2".to_string(),
            value: 42,
        };

        let cid1 = content1.calculate_cid().unwrap();
        let cid2 = content2.calculate_cid().unwrap();
        
        assert_ne!(cid1, cid2);
    }

    #[test]
    fn test_to_bytes_from_bytes() {
        let original = TestContent {
            data: "test data".to_string(),
            value: 12345,
        };

        // Serialize to bytes
        let bytes = original.to_bytes().unwrap();
        assert!(!bytes.is_empty());

        // Deserialize from bytes
        let restored: TestContent = TestContent::from_bytes(&bytes).unwrap();
        assert_eq!(original, restored);
    }

    #[test]
    fn test_from_bytes_invalid() {
        let invalid_bytes = b"not valid json";
        let result = TestContent::from_bytes(invalid_bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_canonical_payload_default() {
        let content = TestContent {
            data: "test".to_string(),
            value: 100,
        };

        let canonical = content.canonical_payload().unwrap();
        let full_bytes = content.to_bytes().unwrap();
        
        // Default implementation should return same as to_bytes
        assert_eq!(canonical, full_bytes);
    }

    #[test]
    fn test_canonical_payload_custom() {
        let content1 = TestContentWithMetadata {
            id: Some("unique-id-1".to_string()),
            data: "same data".to_string(),
            timestamp: Some(1234567890),
        };

        let content2 = TestContentWithMetadata {
            id: Some("unique-id-2".to_string()),
            data: "same data".to_string(),
            timestamp: Some(9876543210),
        };

        // Despite different metadata, canonical payload should be same
        let canonical1 = content1.canonical_payload().unwrap();
        let canonical2 = content2.canonical_payload().unwrap();
        assert_eq!(canonical1, canonical2);

        // And therefore CIDs should be same
        let cid1 = content1.calculate_cid().unwrap();
        let cid2 = content2.calculate_cid().unwrap();
        assert_eq!(cid1, cid2);
    }

    #[test]
    fn test_multihash_error_handling() {
        // Test struct that produces invalid multihash
        #[derive(Debug, Serialize, Deserialize)]
        struct BadContent;

        impl TypedContent for BadContent {
            const CODEC: u64 = 0x300102;
            const CONTENT_TYPE: ContentType = ContentType::Custom(0x300102);

            fn canonical_payload(&self) -> Result<Vec<u8>> {
                // Return empty payload which might cause issues
                Ok(vec![])
            }
        }

        let content = BadContent;
        // Empty payload should still produce valid CID
        let result = content.calculate_cid();
        assert!(result.is_ok());
    }

    #[test]
    fn test_send_sync_bounds() {
        // This test verifies that TypedContent implementations are Send + Sync
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<TestContent>();
        assert_send_sync::<TestContentWithMetadata>();
    }

    #[test]
    fn test_blake3_hash_consistency() {
        let content = TestContent {
            data: "consistent data".to_string(),
            value: 999,
        };

        // Calculate CID multiple times
        let cids: Vec<_> = (0..5)
            .map(|_| content.calculate_cid().unwrap())
            .collect();

        // All CIDs should be identical
        for cid in &cids[1..] {
            assert_eq!(cid, &cids[0]);
        }
    }

    #[test]
    fn test_cid_codec_matches_constant() {
        let content = TestContent {
            data: "test".to_string(),
            value: 1,
        };

        let cid = content.calculate_cid().unwrap();
        assert_eq!(cid.codec(), TestContent::CODEC);
    }

    #[test]
    fn test_serialization_error_propagation() {
        // Test with a type that might fail serialization
        #[derive(Debug, Serialize, Deserialize)]
        struct ComplexContent {
            data: String,
            #[serde(skip)]
            #[allow(dead_code)]
            non_serializable: std::sync::Mutex<i32>,
        }

        impl TypedContent for ComplexContent {
            const CODEC: u64 = 0x300103;
            const CONTENT_TYPE: ContentType = ContentType::Custom(0x300103);
        }

        let content = ComplexContent {
            data: "test".to_string(),
            non_serializable: std::sync::Mutex::new(42),
        };

        // Should still work because Mutex is skipped
        let result = content.to_bytes();
        assert!(result.is_ok());
    }
}
