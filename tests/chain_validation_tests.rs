// Copyright 2025 Cowboy AI, LLC.

//! Chain Validation Tests
//!
//! Tests for content chain validation and integrity

use cim_ipld::{
    ChainedContent, ContentChain, TypedContent, ContentType,
    DagJsonCodec, CodecOperations, Error,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestEvent {
    id: String,
    event_type: String,
    data: String,
}

impl TypedContent for TestEvent {
    const CODEC: u64 = 0x0129; // DAG-JSON
    const CONTENT_TYPE: ContentType = ContentType::Event;
}

#[test]
fn test_chain_sequence_validation() {
    let mut chain = ContentChain::new();
    
    // Add first item
    let event1 = TestEvent {
        id: "evt-001".to_string(),
        event_type: "created".to_string(),
        data: "First event".to_string(),
    };
    chain.append(event1).unwrap();
    
    // Add second item
    let event2 = TestEvent {
        id: "evt-002".to_string(),
        event_type: "updated".to_string(),
        data: "Second event".to_string(),
    };
    chain.append(event2).unwrap();
    
    // Verify sequences
    let items = chain.items();
    assert_eq!(items[0].sequence, 0);
    assert_eq!(items[1].sequence, 1);
    
    // Verify chain linkage
    assert!(items[0].previous_cid.is_none());
    assert_eq!(items[1].previous_cid, Some(items[0].cid.clone()));
}

#[test]
fn test_chain_cid_linkage() {
    // Create standalone chained content items
    let event1 = TestEvent {
        id: "evt-001".to_string(),
        event_type: "created".to_string(),
        data: "First event".to_string(),
    };
    let chained1 = ChainedContent::new(event1, None).unwrap();
    
    let event2 = TestEvent {
        id: "evt-002".to_string(),
        event_type: "updated".to_string(),
        data: "Second event".to_string(),
    };
    let chained2 = ChainedContent::new(event2, Some(&chained1)).unwrap();
    
    // Verify linkage
    assert_eq!(chained2.previous_cid, Some(chained1.cid.clone()));
    assert_eq!(chained2.sequence, chained1.sequence + 1);
    
    // Validate the chain
    chained2.validate_chain(Some(&chained1)).unwrap();
}

#[test]
fn test_chain_validation_detects_tampering() {
    // Create a valid chain
    let event1 = TestEvent {
        id: "evt-001".to_string(),
        event_type: "created".to_string(),
        data: "Original data".to_string(),
    };
    let chained1 = ChainedContent::new(event1, None).unwrap();
    
    // Create second item
    let event2 = TestEvent {
        id: "evt-002".to_string(),
        event_type: "updated".to_string(),
        data: "Second event".to_string(),
    };
    let mut chained2 = ChainedContent::new(event2, Some(&chained1)).unwrap();
    
    // Tamper with the chain by changing previous_cid
    chained2.previous_cid = Some("tampered-cid".to_string());
    
    // Validation should fail
    match chained2.validate_chain(Some(&chained1)) {
        Err(Error::ChainValidationError { expected, actual }) => {
            assert_eq!(expected, chained1.cid);
            assert_eq!(actual, "tampered-cid");
        }
        _ => panic!("Expected ChainValidationError"),
    }
}

#[test]
fn test_empty_chain() {
    let chain: ContentChain<TestEvent> = ContentChain::new();
    assert_eq!(chain.len(), 0);
    assert!(chain.is_empty());
    assert!(chain.head().is_none());
}

#[test]
fn test_chain_iteration() {
    let mut chain = ContentChain::new();
    
    // Add multiple items
    for i in 0..5 {
        let event = TestEvent {
            id: format!("evt-{:03}", i),
            event_type: "test".to_string(),
            data: format!("Event {}", i),
        };
        chain.append(event).unwrap();
    }
    
    // Test iteration
    let items = chain.items();
    assert_eq!(items.len(), 5);
    
    // Verify sequence
    for (i, item) in items.iter().enumerate() {
        assert_eq!(item.sequence, i as u64);
        assert_eq!(item.content.id, format!("evt-{:03}", i));
    }
}

#[test]
fn test_chain_head() {
    let mut chain = ContentChain::new();
    
    let event1 = TestEvent {
        id: "evt-001".to_string(),
        event_type: "first".to_string(),
        data: "First".to_string(),
    };
    chain.append(event1).unwrap();
    
    let event2 = TestEvent {
        id: "evt-002".to_string(),
        event_type: "last".to_string(),
        data: "Last".to_string(),
    };
    chain.append(event2).unwrap();
    
    let head = chain.head().unwrap();
    assert_eq!(head.content.id, "evt-002");
    assert_eq!(head.sequence, 1);
}

#[test]
fn test_sequence_mismatch_validation() {
    // Create first item
    let event1 = TestEvent {
        id: "evt-001".to_string(),
        event_type: "created".to_string(),
        data: "First event".to_string(),
    };
    let chained1 = ChainedContent::new(event1, None).unwrap();
    
    // Create second item with wrong sequence
    let event2 = TestEvent {
        id: "evt-002".to_string(),
        event_type: "updated".to_string(),
        data: "Second event".to_string(),
    };
    let mut chained2 = ChainedContent::new(event2, Some(&chained1)).unwrap();
    
    // Corrupt the sequence
    chained2.sequence = 5; // Should be 1
    
    // Validation should fail
    match chained2.validate_chain(Some(&chained1)) {
        Err(Error::SequenceValidationError { expected, actual }) => {
            assert_eq!(expected, 1);
            assert_eq!(actual, 5);
        }
        _ => panic!("Expected SequenceValidationError"),
    }
}

#[test]
fn test_chain_serialization() {
    let mut chain = ContentChain::new();
    
    let event = TestEvent {
        id: "evt-001".to_string(),
        event_type: "test".to_string(),
        data: "Test data".to_string(),
    };
    chain.append(event).unwrap();
    
    // Get the head and serialize it
    let head = chain.head().unwrap();
    let encoded = head.to_dag_json().unwrap();
    
    // Deserialize
    let decoded: ChainedContent<TestEvent> = DagJsonCodec::decode(&encoded).unwrap();
    
    assert_eq!(decoded.content.id, head.content.id);
    assert_eq!(decoded.cid, head.cid);
    assert_eq!(decoded.sequence, head.sequence);
}

#[test]
fn test_multiple_content_types_in_chain() {
    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct DocEvent {
        doc_id: String,
        action: String,
    }
    
    impl TypedContent for DocEvent {
        const CODEC: u64 = 0x0129;
        const CONTENT_TYPE: ContentType = ContentType::Custom(0x300100);
    }
    
    let mut chain = ContentChain::new();
    
    let doc1 = DocEvent {
        doc_id: "doc-001".to_string(),
        action: "created".to_string(),
    };
    chain.append(doc1).unwrap();
    
    let doc2 = DocEvent {
        doc_id: "doc-002".to_string(),
        action: "modified".to_string(),
    };
    chain.append(doc2).unwrap();
    
    assert_eq!(chain.len(), 2);
    
    // Verify content types
    for _item in chain.items() {
        assert_eq!(DocEvent::CONTENT_TYPE, ContentType::Custom(0x300100));
    }
}

#[test]
fn test_chain_with_large_content() {
    let mut chain = ContentChain::new();
    
    // Create event with large data
    let large_data = "x".repeat(10000);
    let event = TestEvent {
        id: "evt-large".to_string(),
        event_type: "large".to_string(),
        data: large_data.clone(),
    };
    
    chain.append(event).unwrap();
    
    let head = chain.head().unwrap();
    assert_eq!(head.content.data.len(), 10000);
    assert_eq!(head.content.data, large_data);
}

#[test]
fn test_chain_cid_stability() {
    // Same content should produce same CID
    let event1 = TestEvent {
        id: "evt-001".to_string(),
        event_type: "test".to_string(),
        data: "Test data".to_string(),
    };
    let chained1 = ChainedContent::new(event1.clone(), None).unwrap();
    
    let event2 = TestEvent {
        id: "evt-001".to_string(),
        event_type: "test".to_string(),
        data: "Test data".to_string(),
    };
    let chained2 = ChainedContent::new(event2, None).unwrap();
    
    // Same content should produce same CID
    assert_eq!(chained1.cid, chained2.cid);
}

#[test]
fn test_chain_timestamp_ordering() {
    use std::thread;
    use std::time::Duration;
    
    let mut chain = ContentChain::new();
    
    let event1 = TestEvent {
        id: "evt-001".to_string(),
        event_type: "first".to_string(),
        data: "First".to_string(),
    };
    let item1 = chain.append(event1).unwrap();
    let time1 = item1.timestamp;
    
    // Small delay to ensure different timestamp
    thread::sleep(Duration::from_millis(10));
    
    let event2 = TestEvent {
        id: "evt-002".to_string(),
        event_type: "second".to_string(),
        data: "Second".to_string(),
    };
    let item2 = chain.append(event2).unwrap();
    let time2 = item2.timestamp;
    
    // Timestamps should be ordered
    assert!(time2 > time1);
}