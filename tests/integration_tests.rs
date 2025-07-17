//! Integration tests for CIM-IPLD
//!
//! These tests verify the integration of various components working together.

use cim_ipld::{
    ChainedContent, ContentChain, TypedContent, ContentType,
    DagJsonCodec, DagCborCodec, CodecOperations,
    TextDocument, MarkdownDocument, DocumentMetadata,
    detect_content_type, content_type_name,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestEvent {
    id: String,
    action: String,
    data: serde_json::Value,
}

impl TypedContent for TestEvent {
    const CODEC: u64 = 0x0129; // DAG-JSON
    const CONTENT_TYPE: ContentType = ContentType::Json;
}

#[test]
fn test_end_to_end_content_chain() {
    // Create a chain
    let mut chain = ContentChain::new();
    
    // Add multiple events
    for i in 0..5 {
        let event = TestEvent {
            id: format!("evt-{:03}", i),
            action: format!("action.{}", i),
            data: serde_json::json!({
                "index": i,
                "message": format!("Event number {}", i)
            }),
        };
        
        let result = chain.append(event);
        assert!(result.is_ok());
        
        let chained = result.unwrap();
        assert_eq!(chained.sequence, i as u64);
        
        if i > 0 {
            assert!(chained.previous_cid.is_some());
        } else {
            assert!(chained.previous_cid.is_none());
        }
    }
    
    // Validate the chain
    assert!(chain.validate().is_ok());
    assert_eq!(chain.len(), 5);
    
    // Get head
    let head = chain.head().unwrap();
    assert_eq!(head.sequence, 4);
    assert_eq!(head.content.id, "evt-004");
}

#[test]
fn test_content_type_roundtrip() {
    // Test document types
    let doc = TextDocument {
        content: "Test document content".to_string(),
        metadata: DocumentMetadata {
            title: Some("Test Doc".to_string()),
            author: Some("Test Author".to_string()),
            tags: vec!["test".to_string()],
            ..Default::default()
        },
    };
    
    // Encode to DAG-JSON
    let encoded = doc.to_dag_json().unwrap();
    let decoded: TextDocument = DagJsonCodec::decode(&encoded).unwrap();
    assert_eq!(doc.content, decoded.content);
    assert_eq!(doc.metadata.title, decoded.metadata.title);
    
    // Encode to DAG-CBOR
    let cbor_encoded = doc.to_dag_cbor().unwrap();
    let cbor_decoded: TextDocument = DagCborCodec::decode(&cbor_encoded).unwrap();
    assert_eq!(doc.content, cbor_decoded.content);
    
    // Verify CBOR is smaller
    assert!(cbor_encoded.len() < encoded.len());
}

#[test]
fn test_content_detection() {
    // Test content header detection
    let pdf_header = b"%PDF-1.4";
    let detected = detect_content_type(pdf_header);
    assert!(detected.is_some());
    let pdf_type = detected.unwrap();
    assert_eq!(content_type_name(pdf_type), "PDF Document");
    
    let jpeg_header = vec![0xFF, 0xD8, 0xFF];
    let detected = detect_content_type(&jpeg_header);
    assert!(detected.is_some());
    let jpeg_type = detected.unwrap();
    assert_eq!(content_type_name(jpeg_type), "JPEG Image");
    
    let png_header = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    let detected = detect_content_type(&png_header);
    assert!(detected.is_some());
    let png_type = detected.unwrap();
    assert_eq!(content_type_name(png_type), "PNG Image");
}

#[test]
fn test_mixed_content_chain() {
    // Create a chain that can hold different content types
    #[derive(Debug, Clone, Serialize, Deserialize)]
    enum MixedContent {
        Event(TestEvent),
        Document(TextDocument),
    }
    
    impl TypedContent for MixedContent {
        const CODEC: u64 = 0x0129;
        const CONTENT_TYPE: ContentType = ContentType::Json;
    }
    
    let mut chain = ContentChain::new();
    
    // Add an event
    let event = MixedContent::Event(TestEvent {
        id: "evt-001".to_string(),
        action: "created".to_string(),
        data: serde_json::json!({"type": "event"}),
    });
    chain.append(event).unwrap();
    
    // Add a document
    let doc = MixedContent::Document(TextDocument {
        content: "Document content".to_string(),
        metadata: DocumentMetadata {
            title: Some("Mixed Doc".to_string()),
            ..Default::default()
        },
    });
    chain.append(doc).unwrap();
    
    // Validate
    assert!(chain.validate().is_ok());
    assert_eq!(chain.len(), 2);
}

#[test]
fn test_cid_determinism() {
    // Same content should produce same CID
    let content1 = TestEvent {
        id: "evt-123".to_string(),
        action: "test.action".to_string(),
        data: serde_json::json!({"value": 42}),
    };
    
    let content2 = TestEvent {
        id: "evt-123".to_string(),
        action: "test.action".to_string(),
        data: serde_json::json!({"value": 42}),
    };
    
    let chained1 = ChainedContent::new(content1, None).unwrap();
    let chained2 = ChainedContent::new(content2, None).unwrap();
    
    assert_eq!(chained1.cid, chained2.cid);
}

#[test]
fn test_chain_iteration() {
    let mut chain = ContentChain::new();
    
    // Add items
    for i in 0..10 {
        let event = TestEvent {
            id: format!("evt-{}", i),
            action: "test".to_string(),
            data: serde_json::json!({"index": i}),
        };
        chain.append(event).unwrap();
    }
    
    // Iterate over items
    let items = chain.items();
    assert_eq!(items.len(), 10);
    
    for (i, item) in items.iter().enumerate() {
        assert_eq!(item.sequence, i as u64);
        assert_eq!(item.content.id, format!("evt-{}", i));
    }
    
    // Get specific item
    let item_5 = &items[5];
    assert_eq!(item_5.content.data["index"], 5);
}

#[test]
fn test_metadata_handling() {
    let mut metadata = DocumentMetadata::default();
    assert!(metadata.title.is_none());
    
    // Set various fields
    metadata.title = Some("Test Title".to_string());
    metadata.author = Some("Test Author".to_string());
    metadata.tags = vec!["tag1".to_string(), "tag2".to_string()];
    metadata.language = Some("en".to_string());
    
    // Create document with metadata
    let doc = MarkdownDocument {
        content: "# Test\n\nContent".to_string(),
        metadata,
    };
    
    // Encode and decode
    let encoded = doc.to_dag_json().unwrap();
    let decoded: MarkdownDocument = DagJsonCodec::decode(&encoded).unwrap();
    
    assert_eq!(decoded.metadata.title, Some("Test Title".to_string()));
    assert_eq!(decoded.metadata.tags.len(), 2);
}

#[test]
fn test_content_type_codec_values() {
    // Test that content types have expected codec values
    assert_eq!(ContentType::Event.codec(), 0x300000);
    assert_eq!(ContentType::Graph.codec(), 0x300001);
    assert_eq!(ContentType::Json.codec(), 0x310001);
    assert_eq!(ContentType::Markdown.codec(), 0x310000);
    assert_eq!(ContentType::Image.codec(), 0x320000);
    assert_eq!(ContentType::Video.codec(), 0x320001);
    assert_eq!(ContentType::Audio.codec(), 0x320002);
}

#[test]
fn test_chain_error_handling() {
    let mut chain = ContentChain::<TestEvent>::new();
    
    // Chain should be empty
    assert_eq!(chain.len(), 0);
    assert!(chain.head().is_none());
    
    // Add first item
    let event = TestEvent {
        id: "evt-001".to_string(),
        action: "test".to_string(),
        data: serde_json::json!({}),
    };
    
    let chained = chain.append(event).unwrap();
    assert_eq!(chained.sequence, 0);
    
    // Validation should pass
    assert!(chain.validate().is_ok());
}