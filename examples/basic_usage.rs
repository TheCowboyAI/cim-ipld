//! Basic usage examples for CIM-IPLD
//!
//! This example demonstrates the core functionality of CIM-IPLD including:
//! - Creating content with CIDs
//! - Using different codecs
//! - Working with content chains

use cim_ipld::{
    ContentChain, DagJsonCodec, CodecOperations,
    TextDocument, DocumentMetadata, TypedContent, ContentType,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Event {
    id: String,
    action: String,
    timestamp: u64,
}

impl TypedContent for Event {
    const CODEC: u64 = 0x0129; // DAG-JSON
    const CONTENT_TYPE: ContentType = ContentType::Json;
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== CIM-IPLD Basic Usage ===\n");

    // Example 1: Working with text documents
    println!("1. Creating a text document:");
    let doc = TextDocument {
        content: "Hello, CIM-IPLD!".to_string(),
        metadata: DocumentMetadata {
            title: Some("My First Document".to_string()),
            author: Some("Example Author".to_string()),
            tags: vec!["example".to_string(), "demo".to_string()],
            ..Default::default()
        },
    };
    
    // Encode the document
    let encoded = DagJsonCodec::encode(&doc)?;
    println!("  Document encoded to {} bytes", encoded.len());
    
    // Generate CID
    let hash = blake3::hash(&encoded);
    let hash_bytes = hash.as_bytes();
    
    // Create multihash with BLAKE3 code (0x1e)
    let mut multihash_bytes = Vec::new();
    multihash_bytes.push(0x1e); // BLAKE3-256 code
    multihash_bytes.push(hash_bytes.len() as u8);
    multihash_bytes.extend_from_slice(hash_bytes);
    
    let mh = multihash::Multihash::from_bytes(&multihash_bytes)?;
    let cid = cid::Cid::new_v1(0x0129, mh); // 0x0129 is DAG-JSON codec
    println!("  Document CID: {}\n", cid);

    // Example 2: Creating a content chain
    println!("2. Creating a content chain:");
    let mut chain = ContentChain::new();
    
    // Add first event
    let event1 = Event {
        id: "evt-001".to_string(),
        action: "user.created".to_string(),
        timestamp: 1704067200, // 2024-01-01 00:00:00 UTC
    };
    
    let chained1 = chain.append(event1)?;
    println!("  First event:");
    println!("    CID: {}", chained1.cid);
    println!("    Sequence: {}", chained1.sequence);
    println!("    Previous: {:?}", chained1.previous_cid);
    
    // Add second event
    let event2 = Event {
        id: "evt-002".to_string(),
        action: "user.updated".to_string(),
        timestamp: 1704067260,
    };
    
    let chained2 = chain.append(event2)?;
    println!("  Second event:");
    println!("    CID: {}", chained2.cid);
    println!("    Sequence: {}", chained2.sequence);
    println!("    Previous: {:?}\n", chained2.previous_cid);
    
    // Example 3: Chain validation
    println!("3. Chain validation:");
    match chain.validate() {
        Ok(()) => println!("  ✓ Chain is valid"),
        Err(e) => println!("  ✗ Chain validation failed: {}", e),
    }
    
    println!("  Chain length: {}", chain.len());
    
    // Example 4: Using different codecs
    println!("\n4. Codec comparisons:");
    let test_data = serde_json::json!({
        "message": "Test data for codec comparison",
        "values": [1, 2, 3, 4, 5],
        "nested": {
            "key": "value"
        }
    });
    
    // DAG-JSON encoding
    let json_encoded = DagJsonCodec::encode(&test_data)?;
    println!("  DAG-JSON: {} bytes", json_encoded.len());
    
    // DAG-CBOR encoding
    use cim_ipld::DagCborCodec;
    let cbor_encoded = DagCborCodec::encode(&test_data)?;
    println!("  DAG-CBOR: {} bytes", cbor_encoded.len());
    
    // Pretty printing
    let pretty = test_data.to_dag_json_pretty()?;
    println!("\n  Pretty printed:");
    println!("{}", pretty);
    
    Ok(())
}