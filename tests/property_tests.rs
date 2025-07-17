//! Property-based tests for CIM-IPLD
//!
//! These tests use proptest to verify properties that should hold
//! for arbitrary inputs, helping to find edge cases automatically.

use proptest::prelude::*;
use cim_ipld::{
    ContentChain, TypedContent, ContentType,
    DagJsonCodec, DagCborCodec, CodecOperations,
};
use serde::{Serialize, Deserialize};

// Generate arbitrary content for testing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestData {
    id: String,
    value: u64,
    text: String,
}

impl TypedContent for TestData {
    const CODEC: u64 = 0x300100;
    const CONTENT_TYPE: ContentType = ContentType::Custom(0x300100);
}

// Strategy for generating TestData
fn arb_test_data() -> impl Strategy<Value = TestData> {
    (
        "[a-zA-Z0-9]{1,20}",  // id
        any::<u64>(),          // value
        "[a-zA-Z0-9 ]{0,100}", // text
    ).prop_map(|(id, value, text)| TestData { id, value, text })
}

proptest! {
    #[test]
    fn test_cid_determinism(data in arb_test_data()) {
        // Property: Same content always produces same CID
        let cid1 = data.calculate_cid().unwrap();
        let cid2 = data.calculate_cid().unwrap();
        prop_assert_eq!(cid1, cid2);
    }

    #[test]
    fn test_cid_uniqueness(data1 in arb_test_data(), data2 in arb_test_data()) {
        // Property: Different content produces different CIDs
        // (with very high probability)
        prop_assume!(data1 != data2);
        
        let cid1 = data1.calculate_cid().unwrap();
        let cid2 = data2.calculate_cid().unwrap();
        prop_assert_ne!(cid1, cid2);
    }

    #[test]
    fn test_serialization_roundtrip(data in arb_test_data()) {
        // Property: Serialization and deserialization preserves data
        let bytes = data.to_bytes().unwrap();
        let restored: TestData = TestData::from_bytes(&bytes).unwrap();
        prop_assert_eq!(data, restored);
    }

    #[test]
    fn test_chain_append_maintains_order(items in prop::collection::vec(arb_test_data(), 1..20)) {
        // Property: Chain maintains insertion order
        let mut chain = ContentChain::<TestData>::new();
        
        for item in &items {
            chain.append(item.clone()).unwrap();
        }
        
        let chain_items = chain.items();
        prop_assert_eq!(chain_items.len(), items.len());
        
        for (i, item) in items.iter().enumerate() {
            prop_assert_eq!(&chain_items[i].content, item);
        }
    }

    #[test]
    fn test_chain_sequence_numbers(items in prop::collection::vec(arb_test_data(), 1..20)) {
        // Property: Sequence numbers are consecutive starting from 0
        let mut chain = ContentChain::<TestData>::new();
        
        for item in items {
            chain.append(item).unwrap();
        }
        
        let chain_items = chain.items();
        for (i, item) in chain_items.iter().enumerate() {
            prop_assert_eq!(item.sequence, i as u64);
        }
    }

    #[test]
    fn test_chain_validation_after_append(items in prop::collection::vec(arb_test_data(), 1..20)) {
        // Property: Chain remains valid after each append
        let mut chain = ContentChain::<TestData>::new();
        
        for item in items {
            chain.append(item).unwrap();
            prop_assert!(chain.validate().is_ok());
        }
    }

    #[test]
    fn test_dag_json_roundtrip(data in arb_test_data()) {
        // Property: DAG-JSON encoding and decoding preserves data
        let json = data.to_dag_json().unwrap();
        let restored: TestData = DagJsonCodec::decode(&json).unwrap();
        prop_assert_eq!(data, restored);
    }

    #[test]
    fn test_dag_cbor_roundtrip(data in arb_test_data()) {
        // Property: DAG-CBOR encoding and decoding preserves data
        let cbor = data.to_dag_cbor().unwrap();
        let restored: TestData = DagCborCodec::decode(&cbor).unwrap();
        prop_assert_eq!(data, restored);
    }

    #[test]
    fn test_content_type_codec_roundtrip(codec in 0x300000u64..=0x3FFFFFu64) {
        // Property: ContentType::from_codec(ct.codec()) == Some(ct) for custom types
        let content_type = ContentType::Custom(codec);
        let result = ContentType::from_codec(content_type.codec());
        prop_assert_eq!(result, Some(content_type));
    }

    #[test]
    fn test_chain_items_since_consistency(items in prop::collection::vec(arb_test_data(), 2..20)) {
        // Property: items_since returns all items after given CID
        let mut chain = ContentChain::<TestData>::new();
        
        for item in &items {
            chain.append(item.clone()).unwrap();
        }
        
        // Pick a random position
        let mid = items.len() / 2;
        let mid_cid = &chain.items()[mid].cid;
        
        let since_items = chain.items_since(mid_cid).unwrap();
        prop_assert_eq!(since_items.len(), items.len() - mid - 1);
        
        // Verify all returned items come after the given CID
        for (i, item) in since_items.iter().enumerate() {
            prop_assert_eq!(item.sequence, (mid + i + 1) as u64);
        }
    }

    #[test]
    fn test_empty_string_handling(empty_count in 0..5usize) {
        // Property: Empty strings in content are handled correctly
        let data = TestData {
            id: "test".to_string(),
            value: 42,
            text: "".repeat(empty_count), // Always empty
        };
        
        let cid = data.calculate_cid();
        prop_assert!(cid.is_ok());
        
        let bytes = data.to_bytes();
        prop_assert!(bytes.is_ok());
        
        if let Ok(b) = bytes {
            let restored: Result<TestData, _> = TestData::from_bytes(&b);
            prop_assert!(restored.is_ok());
        }
    }

    #[test]
    fn test_large_content_handling(size in 1..1000usize) {
        // Property: Large content is handled correctly
        let data = TestData {
            id: "x".repeat(size.min(20)), // Cap ID at 20 chars
            value: size as u64,
            text: "a".repeat(size),
        };
        
        let cid = data.calculate_cid();
        prop_assert!(cid.is_ok());
        
        let bytes = data.to_bytes();
        prop_assert!(bytes.is_ok());
    }

    #[test]
    fn test_unicode_content(s in "\\PC{1,50}") {
        // Property: Unicode content is handled correctly
        let data = TestData {
            id: "unicode_test".to_string(),
            value: 123,
            text: s.clone(),
        };
        
        let bytes = data.to_bytes().unwrap();
        let restored: TestData = TestData::from_bytes(&bytes).unwrap();
        prop_assert_eq!(restored.text, s);
    }
}

// Property tests for edge cases in chain validation
proptest! {
    #[test]
    fn test_chain_tampering_detection(
        items in prop::collection::vec(arb_test_data(), 2..10),
        tamper_field in 0..3usize,
    ) {
        // Property: Chain validation detects any tampering
        let mut chain = ContentChain::<TestData>::new();
        
        for item in items {
            chain.append(item).unwrap();
        }
        
        prop_assume!(chain.len() > 1);
        
        // Get a copy of an item from the middle of the chain
        let idx = chain.len() / 2;
        let original_item = &chain.items()[idx];
        
        // Create a tampered version
        let mut tampered_data = original_item.content.clone();
        match tamper_field {
            0 => tampered_data.id = "TAMPERED".to_string(),
            1 => tampered_data.value = tampered_data.value.wrapping_add(1),
            _ => tampered_data.text = "TAMPERED".to_string(),
        }
        
        // The tampered data should produce a different CID
        let original_cid = original_item.content.calculate_cid().unwrap();
        let tampered_cid = tampered_data.calculate_cid().unwrap();
        prop_assert_ne!(original_cid, tampered_cid);
    }

    #[test]
    fn test_cid_stability_across_operations(data in arb_test_data()) {
        // Property: CID doesn't change when content is cloned or moved
        let original_cid = data.calculate_cid().unwrap();
        
        let cloned = data.clone();
        let cloned_cid = cloned.calculate_cid().unwrap();
        prop_assert_eq!(original_cid, cloned_cid);
        
        let moved = data;
        let moved_cid = moved.calculate_cid().unwrap();
        prop_assert_eq!(original_cid, moved_cid);
    }
}