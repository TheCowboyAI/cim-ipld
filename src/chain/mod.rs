// Copyright 2025 Cowboy AI, LLC.

//! Content-addressed chains for integrity
//!
//! This module provides chain-based content management with cryptographic
//! linking between content items, ensuring integrity and auditability.
//!
//! # Example
//!
//! ```
//! use cim_ipld::chain::ChainedContent;
//! use cim_ipld::{TypedContent, ContentType};
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Debug, Clone, Serialize, Deserialize)]
//! struct MyContent {
//!     message: String,
//! }
//!
//! impl TypedContent for MyContent {
//!     const CODEC: u64 = 0x0129; // DAG-JSON
//!     const CONTENT_TYPE: ContentType = ContentType::Json;
//! }
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create the first item in a chain
//! let content1 = MyContent { message: "First".to_string() };
//! let chained1 = ChainedContent::new(content1, None)?;
//! 
//! // Add another item linked to the first
//! let content2 = MyContent { message: "Second".to_string() };
//! let chained2 = ChainedContent::new(content2, Some(&chained1))?;
//! 
//! // Verify chain properties
//! assert_eq!(chained1.sequence, 0);
//! assert_eq!(chained2.sequence, 1);
//! assert_eq!(chained2.previous_cid, Some(chained1.cid.clone()));
//! # Ok(())
//! # }
//! ```

use crate::{Cid, Error, Result, TypedContent};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// A content item with chain linking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainedContent<T> {
    /// The actual content
    pub content: T,

    /// Content identifier for this item
    pub cid: String, // Store as string for easier serialization

    /// CID of the previous item in the chain
    pub previous_cid: Option<String>,

    /// Sequence number in the chain
    pub sequence: u64,

    /// Timestamp when the item was chained
    pub timestamp: SystemTime,
}

impl<T: TypedContent> ChainedContent<T> {
    /// Create a new chained content item
    pub fn new(content: T, previous: Option<&ChainedContent<T>>) -> Result<Self> {
        let sequence = previous.map(|p| p.sequence + 1).unwrap_or(0);
        let previous_cid = previous.map(|p| p.cid.clone());
        let timestamp = SystemTime::now();

        // Create a temporary item to calculate CID
        let mut chained = Self {
            content,
            cid: String::new(), // Temporary
            previous_cid,
            sequence,
            timestamp,
        };

        // Calculate and set the actual CID
        chained.cid = chained.calculate_cid()?;

        Ok(chained)
    }

    /// Calculate the CID for this chained content
    fn calculate_cid(&self) -> Result<String> {
        // Create a deterministic representation for hashing
        let chain_data = ChainData {
            content: &self.content,
            previous_cid: &self.previous_cid,
            sequence: self.sequence,
            // Don't include timestamp in CID calculation for determinism
        };

        // Serialize to calculate CID
        let bytes = serde_json::to_vec(&chain_data)?;

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
            .map_err(|e| Error::MultihashError(e.to_string()))?;
        let cid = Cid::new_v1(T::CODEC, mh);

        Ok(cid.to_string())
    }

    /// Validate this item against a previous item
    pub fn validate_chain(&self, previous: Option<&ChainedContent<T>>) -> Result<()> {
        match (previous, &self.previous_cid) {
            // First item in chain
            (None, None) => {
                if self.sequence != 0 {
                    return Err(Error::SequenceValidationError {
                        expected: 0,
                        actual: self.sequence,
                    });
                }
            }
            // Continuing chain
            (Some(prev), Some(prev_cid)) => {
                // Validate CID link
                if prev.cid != *prev_cid {
                    return Err(Error::ChainValidationError {
                        expected: prev.cid.clone(),
                        actual: prev_cid.clone(),
                    });
                }
                // Validate sequence
                if self.sequence != prev.sequence + 1 {
                    return Err(Error::SequenceValidationError {
                        expected: prev.sequence + 1,
                        actual: self.sequence,
                    });
                }
            }
            // Mismatch
            _ => {
                return Err(Error::ChainValidationError {
                    expected: previous.map(|p| p.cid.clone()).unwrap_or_default(),
                    actual: self.previous_cid.clone().unwrap_or_default(),
                });
            }
        }

        // Verify our own CID
        let calculated_cid = self.calculate_cid()?;
        if calculated_cid != self.cid {
            return Err(Error::InvalidCid(format!("CID mismatch: expected {}, calculated {}", self.cid, calculated_cid)));
        }

        Ok(())
    }

    /// Parse a CID from string
    pub fn parse_cid(cid_str: &str) -> Result<Cid> {
        Cid::try_from(cid_str).map_err(|e| Error::InvalidCid(e.to_string()))
    }
}

/// Content used for CID calculation (excludes mutable fields)
#[derive(Serialize)]
struct ChainData<'a, T: Serialize> {
    content: &'a T,
    previous_cid: &'a Option<String>,
    sequence: u64,
}

/// A chain of content items with validation
#[derive(Debug, Clone)]
pub struct ContentChain<T: TypedContent> {
    items: Vec<ChainedContent<T>>,
}

impl<T: TypedContent> ContentChain<T> {
    /// Create a new empty chain
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Add content to the chain
    pub fn append(&mut self, content: T) -> Result<&ChainedContent<T>> {
        let previous = self.items.last();
        let chained = ChainedContent::new(content, previous)?;

        // Validate the chain
        chained.validate_chain(previous)?;

        self.items.push(chained);
        Ok(self.items.last().unwrap())
    }

    /// Validate the entire chain
    pub fn validate(&self) -> Result<()> {
        let mut previous: Option<&ChainedContent<T>> = None;

        for item in &self.items {
            item.validate_chain(previous)?;
            previous = Some(item);
        }

        Ok(())
    }

    /// Get the chain head (latest item)
    pub fn head(&self) -> Option<&ChainedContent<T>> {
        self.items.last()
    }

    /// Get the chain length
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if chain is empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Get all items
    pub fn items(&self) -> &[ChainedContent<T>] {
        &self.items
    }

    /// Get items since a specific CID
    pub fn items_since(&self, cid: &str) -> Result<Vec<&ChainedContent<T>>> {
        // Find the item with the given CID
        let start_idx = self
            .items
            .iter()
            .position(|e| e.cid == cid)
            .ok_or_else(|| Error::InvalidCid(format!("CID not found in chain: {cid}")))?;

        // Return all items after that one
        Ok(self.items[start_idx + 1..].iter().collect())
    }
}

impl<T: TypedContent> Default for ContentChain<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ContentType;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestContent {
        id: String,
        data: String,
    }

    impl TypedContent for TestContent {
        const CODEC: u64 = 0x300000;
        const CONTENT_TYPE: ContentType = ContentType::Event;
    }

    #[test]
    fn test_chained_content_creation() {
        // Given
        let content = TestContent {
            id: "test-1".to_string(),
            data: "Test data".to_string(),
        };

        // When
        let chained = ChainedContent::new(content.clone(), None).unwrap();

        // Then
        assert_eq!(chained.sequence, 0);
        assert!(chained.previous_cid.is_none());
        assert!(!chained.cid.is_empty());

        // Verify CID format
        let cid = ChainedContent::<TestContent>::parse_cid(&chained.cid).unwrap();
        assert_eq!(cid.version(), cid::Version::V1);
    }

    #[test]
    fn test_content_chain_append() {
        // Given
        let mut chain = ContentChain::new();
        let content1 = TestContent {
            id: "test-1".to_string(),
            data: "Data 1".to_string(),
        };
        let content2 = TestContent {
            id: "test-2".to_string(),
            data: "Data 2".to_string(),
        };

        // When
        let chained1 = chain.append(content1).unwrap();
        let cid1 = chained1.cid.clone();

        let chained2 = chain.append(content2).unwrap();
        let sequence2 = chained2.sequence;
        let prev_cid2 = chained2.previous_cid.clone();

        // Then
        assert_eq!(chain.len(), 2);
        assert_eq!(sequence2, 1);
        assert_eq!(prev_cid2, Some(cid1));
    }

    #[test]
    fn test_empty_chain_operations() {
        let chain = ContentChain::<TestContent>::new();
        
        assert!(chain.is_empty());
        assert_eq!(chain.len(), 0);
        assert!(chain.head().is_none());
        assert!(chain.validate().is_ok());
        assert!(chain.items_since("invalid-cid").is_err());
    }

    #[test]
    fn test_chain_with_single_item() {
        let mut chain = ContentChain::new();
        let content = TestContent {
            id: "single".to_string(),
            data: "only item".to_string(),
        };
        
        chain.append(content.clone()).unwrap();
        
        assert!(!chain.is_empty());
        assert_eq!(chain.len(), 1);
        assert!(chain.head().is_some());
        assert_eq!(chain.head().unwrap().sequence, 0);
        assert!(chain.head().unwrap().previous_cid.is_none());
        assert!(chain.validate().is_ok());
    }

    #[test]
    fn test_invalid_cid_parsing() {
        // Test various invalid CID formats
        assert!(ChainedContent::<TestContent>::parse_cid("").is_err());
        assert!(ChainedContent::<TestContent>::parse_cid("not-a-cid").is_err());
        assert!(ChainedContent::<TestContent>::parse_cid("123456").is_err());
        assert!(ChainedContent::<TestContent>::parse_cid("Qm...").is_err()); // Invalid V0 CID
    }

    #[test]
    fn test_items_since_edge_cases() {
        let mut chain = ContentChain::new();
        
        // Add multiple items
        for i in 0..5 {
            chain.append(TestContent {
                id: format!("item-{}", i),
                data: format!("data-{}", i),
            }).unwrap();
        }
        
        // Test with head CID (should return empty)
        let head_cid = &chain.head().unwrap().cid;
        let items = chain.items_since(head_cid).unwrap();
        assert_eq!(items.len(), 0);
        
        // Test with first CID (should return all but first)
        let first_cid = &chain.items()[0].cid;
        let items = chain.items_since(first_cid).unwrap();
        assert_eq!(items.len(), 4);
        
        // Test with non-existent CID
        assert!(chain.items_since("non-existent-cid").is_err());
    }

    #[test]
    fn test_chain_validation_with_wrong_codec() {
        // Create content with one codec
        let content = TestContent {
            id: "test".to_string(),
            data: "data".to_string(),
        };
        
        let item1 = ChainedContent::new(content.clone(), None).unwrap();
        
        // Try to create next item with wrong previous CID calculation
        let mut item2 = ChainedContent::new(content, Some(&item1)).unwrap();
        
        // Tamper with the previous CID
        item2.previous_cid = Some("wrong-cid".to_string());
        
        // Validation should fail
        assert!(item2.validate_chain(Some(&item1)).is_err());
    }

    #[test]
    fn test_large_chain_performance() {
        let mut chain = ContentChain::new();
        
        // Add many items
        for i in 0..100 {
            chain.append(TestContent {
                id: format!("item-{}", i),
                data: format!("Large data content for item {}: {}", i, "x".repeat(100)),
            }).unwrap();
        }
        
        assert_eq!(chain.len(), 100);
        assert!(chain.validate().is_ok());
        
        // Test items_since with middle item
        let mid_cid = &chain.items()[50].cid;
        let items = chain.items_since(mid_cid).unwrap();
        assert_eq!(items.len(), 49);
    }

    #[test]
    fn test_chain_default_trait() {
        let chain1 = ContentChain::<TestContent>::new();
        let chain2 = ContentChain::<TestContent>::default();
        
        assert_eq!(chain1.len(), chain2.len());
        assert_eq!(chain1.is_empty(), chain2.is_empty());
    }

    #[test]
    fn test_content_with_special_characters() {
        let mut chain = ContentChain::new();
        
        // Test with various special characters
        let special_contents = vec![
            TestContent {
                id: "emoji".to_string(),
                data: "🚀 Rocket emoji and 中文 Chinese text".to_string(),
            },
            TestContent {
                id: "newlines".to_string(),
                data: "Line 1\nLine 2\r\nLine 3\tTabbed".to_string(),
            },
            TestContent {
                id: "quotes".to_string(),
                data: r#"Single ' and double " quotes"#.to_string(),
            },
            TestContent {
                id: "null".to_string(),
                data: "\0Null byte\0".to_string(),
            },
        ];
        
        for content in special_contents {
            let result = chain.append(content);
            assert!(result.is_ok());
        }
        
        assert_eq!(chain.len(), 4);
        assert!(chain.validate().is_ok());
    }

    #[test]
    fn test_cid_consistency_across_serialization() {
        let content = TestContent {
            id: "consistent".to_string(),
            data: "Should have same CID".to_string(),
        };
        
        // Create multiple chained items with same content
        let item1 = ChainedContent::new(content.clone(), None).unwrap();
        let item2 = ChainedContent::new(content.clone(), None).unwrap();
        
        // They should have the same CID
        assert_eq!(item1.cid, item2.cid);
        
        // But different timestamps mean they're not equal
        assert_ne!(item1.timestamp, item2.timestamp);
    }

    #[test]
    fn test_chain_validation() {
        // Given
        let mut chain = ContentChain::new();

        // Add multiple items
        for i in 0..5 {
            let content = TestContent {
                id: format!("test-{i}"),
                data: format!("Data {i}"),
            };
            chain.append(content).unwrap();
        }

        // When/Then - validation should pass
        chain.validate().unwrap();

        // Verify sequence numbers
        for (i, item) in chain.items().iter().enumerate() {
            assert_eq!(item.sequence, i as u64);
        }
    }

    #[test]
    fn test_items_since() {
        // Given
        let mut chain = ContentChain::new();
        let mut cids = Vec::new();

        // Add items and track CIDs
        for i in 0..5 {
            let content = TestContent {
                id: format!("test-{i}"),
                data: format!("Data {i}"),
            };
            let chained = chain.append(content).unwrap();
            cids.push(chained.cid.clone());
        }

        // When - get items since index 2
        let items_since = chain.items_since(&cids[2]).unwrap();

        // Then
        assert_eq!(items_since.len(), 2); // Items 3 and 4
        assert_eq!(items_since[0].sequence, 3);
        assert_eq!(items_since[1].sequence, 4);
    }

    #[test]
    fn test_cid_determinism() {
        // Given - same content
        let content = TestContent {
            id: "test-determinism".to_string(),
            data: "Deterministic data".to_string(),
        };

        // When - create two chained items
        let chained1 = ChainedContent::new(content.clone(), None).unwrap();
        let chained2 = ChainedContent::new(content, None).unwrap();

        // Then - CIDs should be identical (deterministic)
        assert_eq!(chained1.cid, chained2.cid);
    }

    #[test]
    fn test_chain_tampering_detection() {
        // Given
        let mut chain = ContentChain::new();
        let content = TestContent {
            id: "test-tamper".to_string(),
            data: "Original data".to_string(),
        };
        chain.append(content).unwrap();

        // When - tamper with the item
        let mut tampered_item = chain.items[0].clone();
        tampered_item.sequence = 999; // Change sequence

        // Then - validation should fail
        let result = tampered_item.validate_chain(None);
        assert!(result.is_err());

        match result {
            Err(Error::SequenceValidationError { expected: 0, actual: 999 }) => {}, // Sequence mismatch detected
            Err(Error::InvalidCid(_)) => {}, // CID mismatch detected
            _ => panic!("Expected validation error"),
        }
    }

    #[test]
    fn test_chain_validation_mismatch_cases() {
        let content1 = TestContent {
            id: "test-1".to_string(),
            data: "Data 1".to_string(),
        };
        
        // Test case 1: Item has previous_cid but no previous item provided
        let mut chained_with_prev = ChainedContent::new(content1.clone(), None).unwrap();
        chained_with_prev.previous_cid = Some("fake-cid".to_string());
        chained_with_prev.sequence = 1;
        
        let result = chained_with_prev.validate_chain(None);
        assert!(result.is_err());
        match result {
            Err(Error::ChainValidationError { expected, actual }) => {
                assert_eq!(expected, "");
                assert_eq!(actual, "fake-cid");
            }
            _ => panic!("Expected ChainValidationError"),
        }
        
        // Test case 2: Item has no previous_cid but previous item provided
        let first_item = ChainedContent::new(content1.clone(), None).unwrap();
        let mut second_item = ChainedContent::new(content1, None).unwrap();
        second_item.previous_cid = None; // Remove the previous CID
        
        let result = second_item.validate_chain(Some(&first_item));
        assert!(result.is_err());
        match result {
            Err(Error::ChainValidationError { expected, actual }) => {
                assert_eq!(expected, first_item.cid);
                assert_eq!(actual, "");
            }
            _ => panic!("Expected ChainValidationError"),
        }
    }

    #[test]
    fn test_cid_mismatch_detection() {
        let content = TestContent {
            id: "test".to_string(),
            data: "test data".to_string(),
        };
        
        // Create a valid chained item
        let mut chained = ChainedContent::new(content, None).unwrap();
        
        // Tamper with the CID
        chained.cid = "bafyreigdyrzt5sfp7udm7hu76uh7y26fake".to_string();
        
        // Validation should detect CID mismatch
        let result = chained.validate_chain(None);
        assert!(result.is_err());
        match result {
            Err(Error::InvalidCid(msg)) => {
                assert!(msg.contains("CID mismatch"));
                assert!(msg.contains("expected"));
                assert!(msg.contains("calculated"));
            }
            _ => panic!("Expected InvalidCid error"),
        }
    }

    #[test]
    fn test_chain_data_serialization() {
        // Test the ChainData struct serialization
        let content = TestContent {
            id: "serialize-test".to_string(),
            data: "data".to_string(),
        };
        
        let chain_data = ChainData {
            content: &content,
            previous_cid: &Some("previous-cid".to_string()),
            sequence: 42,
        };
        
        // Serialize and verify it works
        let serialized = serde_json::to_string(&chain_data).unwrap();
        assert!(serialized.contains("serialize-test"));
        assert!(serialized.contains("previous-cid"));
        assert!(serialized.contains("42"));
    }

    #[test]
    fn test_multihash_error_handling() {
        // Test error handling in calculate_cid when multihash creation fails
        // This is hard to trigger naturally, but we can test the error path exists
        let content = TestContent {
            id: "test".to_string(),
            data: "data".to_string(),
        };
        
        let chained = ChainedContent::new(content, None).unwrap();
        
        // The multihash creation with valid BLAKE3 should succeed
        assert!(!chained.cid.is_empty());
        
        // Test with invalid multihash bytes
        let invalid_bytes = vec![0xff, 0xff]; // Invalid varint
        let result = multihash::Multihash::<64>::from_bytes(&invalid_bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_append_after_push_edge_case() {
        // Test that the unwrap() in append() is safe
        let mut chain = ContentChain::new();
        
        // The Vec is never empty after push, so unwrap is safe
        let content = TestContent {
            id: "test".to_string(),
            data: "data".to_string(),
        };
        
        let result = chain.append(content);
        assert!(result.is_ok());
        
        // Verify the last() unwrap worked
        let appended = result.unwrap();
        assert_eq!(appended.sequence, 0);
    }

    #[test]
    fn test_chain_validation_with_corrupted_cid() {
        let content1 = TestContent {
            id: "first".to_string(),
            data: "first data".to_string(),
        };
        let content2 = TestContent {
            id: "second".to_string(),
            data: "second data".to_string(),
        };
        
        // Create first item
        let first = ChainedContent::new(content1, None).unwrap();
        
        // Create second item with corrupted previous_cid
        let mut second = ChainedContent::new(content2, Some(&first)).unwrap();
        second.previous_cid = Some("corrupted-cid".to_string());
        
        // Validation should fail
        let result = second.validate_chain(Some(&first));
        assert!(result.is_err());
        match result {
            Err(Error::ChainValidationError { expected, actual }) => {
                assert_eq!(expected, first.cid);
                assert_eq!(actual, "corrupted-cid");
            }
            _ => panic!("Expected ChainValidationError"),
        }
    }

    #[test]
    fn test_sequence_validation_errors() {
        let content = TestContent {
            id: "test".to_string(),
            data: "data".to_string(),
        };
        
        // Test case 1: First item with non-zero sequence
        let mut first = ChainedContent::new(content.clone(), None).unwrap();
        first.sequence = 5;
        
        let result = first.validate_chain(None);
        assert!(result.is_err());
        match result {
            Err(Error::SequenceValidationError { expected: 0, actual: 5 }) => {}
            _ => panic!("Expected SequenceValidationError"),
        }
        
        // Test case 2: Non-sequential sequence numbers
        let prev = ChainedContent::new(content.clone(), None).unwrap();
        let mut next = ChainedContent::new(content, Some(&prev)).unwrap();
        next.sequence = 10; // Should be 1
        
        let result = next.validate_chain(Some(&prev));
        assert!(result.is_err());
        match result {
            Err(Error::SequenceValidationError { expected: 1, actual: 10 }) => {}
            _ => panic!("Expected SequenceValidationError"),
        }
    }

    #[test]
    fn test_empty_chain_edge_cases() {
        let chain = ContentChain::<TestContent>::new();
        
        // Test items() on empty chain
        assert_eq!(chain.items().len(), 0);
        
        // Test validate() on empty chain
        assert!(chain.validate().is_ok());
        
        // Test items_since with any CID on empty chain
        let result = chain.items_since("any-cid");
        assert!(result.is_err());
        match result {
            Err(Error::InvalidCid(msg)) => {
                assert!(msg.contains("not found in chain"));
            }
            _ => panic!("Expected InvalidCid error"),
        }
    }

    #[test]
    fn test_chain_with_validation_failure() {
        // Create a chain that will fail validation
        let mut chain = ContentChain::new();
        
        let content1 = TestContent {
            id: "1".to_string(),
            data: "data1".to_string(),
        };
        
        // Add first item normally
        chain.append(content1.clone()).unwrap();
        
        // Manually create and add a corrupted second item
        let mut corrupted = ChainedContent::new(content1, None).unwrap();
        corrupted.sequence = 1;
        corrupted.previous_cid = Some("wrong-cid".to_string());
        
        // Force add the corrupted item
        chain.items.push(corrupted);
        
        // Validation should fail
        let result = chain.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_json_serialization_error_handling() {
        use std::f64;
        
        // Create content that might cause serialization issues
        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct ProblematicContent {
            // Using types that are valid in Rust but might have edge cases in JSON
            float: f64,
            large_num: u64,
        }
        
        impl TypedContent for ProblematicContent {
            const CODEC: u64 = 0x300001;
            const CONTENT_TYPE: ContentType = ContentType::Json;
        }
        
        let content = ProblematicContent {
            float: f64::INFINITY,
            large_num: u64::MAX,
        };
        
        // This should handle the serialization gracefully
        let result = ChainedContent::new(content, None);
        
        // JSON doesn't support Infinity, so this might fail
        // But our code should handle it without panicking
        match result {
            Ok(_) => {} // Implementation handles it
            Err(_) => {} // Or returns an error
        }
    }
}
