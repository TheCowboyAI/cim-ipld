// Copyright 2025 Cowboy AI, LLC.

//! Event CID Generation Example
//!
//! This example demonstrates how to create events with Content Identifiers (CIDs)
//! and work with event chains in CIM-IPLD.

use cim_ipld::{
    ContentChain, TypedContent, ContentType,
    DagJsonCodec, Cid,
};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// A simple event structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Event {
    id: String,
    event_type: String,
    payload: serde_json::Value,
    timestamp: u64,
}

impl Event {
    fn new(id: &str, event_type: &str, payload: serde_json::Value) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        Event {
            id: id.to_string(),
            event_type: event_type.to_string(),
            payload,
            timestamp,
        }
    }
}

impl TypedContent for Event {
    const CODEC: u64 = 0x0129; // DAG-JSON
    const CONTENT_TYPE: ContentType = ContentType::Json;
}

/// A message wrapper around events
#[derive(Debug, Clone, Serialize, Deserialize)]
struct EventMessage {
    event: Event,
    source: String,
    correlation_id: String,
}

impl TypedContent for EventMessage {
    const CODEC: u64 = 0x0129; // DAG-JSON
    const CONTENT_TYPE: ContentType = ContentType::Json;
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Event CID Generation Example ===\n");
    
    // Example 1: Basic event CID generation
    basic_event_cid()?;
    
    // Example 2: Event chain demonstration
    event_chain_demo()?;
    
    // Example 3: CID comparison
    cid_comparison()?;
    
    // Example 4: Event messages with CIDs
    event_message_demo()?;
    
    Ok(())
}

fn basic_event_cid() -> Result<(), Box<dyn std::error::Error>> {
    println!("1. Basic Event CID Generation:");
    
    let event = Event::new(
        "evt-001",
        "user.created",
        serde_json::json!({
            "user_id": "usr-123",
            "email": "user@example.com",
            "name": "Test User"
        })
    );
    
    // Generate CID for the event
    let cid = generate_cid(&event)?;
    
    println!("  Event: {:?}", event.id);
    println!("  Type: {}", event.event_type);
    println!("  CID: {}", cid);
    
    // Show that same content produces same CID
    let cid2 = generate_cid(&event)?;
    println!("  Same content, same CID: {}\n", cid == cid2);
    
    Ok(())
}

fn event_chain_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("2. Event Chain Demonstration:");
    
    let mut chain = ContentChain::new();
    
    // Create a series of related events
    let events = vec![
        Event::new(
            "evt-001",
            "order.created",
            serde_json::json!({"order_id": "ord-123", "total": 99.99})
        ),
        Event::new(
            "evt-002",
            "order.payment_received",
            serde_json::json!({"order_id": "ord-123", "amount": 99.99})
        ),
        Event::new(
            "evt-003",
            "order.shipped",
            serde_json::json!({"order_id": "ord-123", "tracking": "TRK-456"})
        ),
    ];
    
    println!("  Creating event chain:");
    for event in events {
        let chained = chain.append(event.clone())?;
        println!("    {} -> CID: {}", event.event_type, chained.cid);
        if let Some(prev) = &chained.previous_cid {
            println!("      Previous: {}", prev);
        }
    }
    
    // Validate the chain
    chain.validate()?;
    println!("  ✓ Chain validated successfully");
    println!("  Chain length: {}\n", chain.len());
    
    Ok(())
}

fn cid_comparison() -> Result<(), Box<dyn std::error::Error>> {
    println!("3. CID Comparison:");
    
    // Same content, same CID
    let event1 = Event {
        id: "evt-001".to_string(),
        event_type: "test.event".to_string(),
        payload: serde_json::json!({"value": 42}),
        timestamp: 1704067200,
    };
    
    let event2 = Event {
        id: "evt-001".to_string(),
        event_type: "test.event".to_string(),
        payload: serde_json::json!({"value": 42}),
        timestamp: 1704067200,
    };
    
    let cid1 = generate_cid(&event1)?;
    let cid2 = generate_cid(&event2)?;
    
    println!("  Same content:");
    println!("    CID1: {}", cid1);
    println!("    CID2: {}", cid2);
    println!("    Equal: {}", cid1 == cid2);
    
    // Different content, different CID
    let event3 = Event {
        id: "evt-002".to_string(), // Different ID
        event_type: "test.event".to_string(),
        payload: serde_json::json!({"value": 42}),
        timestamp: 1704067200,
    };
    
    let cid3 = generate_cid(&event3)?;
    
    println!("\n  Different content:");
    println!("    CID1: {}", cid1);
    println!("    CID3: {}", cid3);
    println!("    Equal: {}\n", cid1 == cid3);
    
    Ok(())
}

fn event_message_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("4. Event Messages with CIDs:");
    
    let event = Event::new(
        "evt-001",
        "system.alert",
        serde_json::json!({
            "level": "warning",
            "message": "High memory usage detected"
        })
    );
    
    let message = EventMessage {
        event: event.clone(),
        source: "monitoring-service".to_string(),
        correlation_id: "corr-123".to_string(),
    };
    
    // Generate CIDs for both event and message
    let event_cid = generate_cid(&event)?;
    let message_cid = generate_cid(&message)?;
    
    println!("  Event CID: {}", event_cid);
    println!("  Message CID: {}", message_cid);
    println!("  Different CIDs: {}", event_cid != message_cid);
    
    // Create another message with same event
    let message2 = EventMessage {
        event: event.clone(),
        source: "monitoring-service".to_string(),
        correlation_id: "corr-456".to_string(), // Different correlation ID
    };
    
    let message_cid2 = generate_cid(&message2)?;
    
    println!("\n  Same event, different correlation:");
    println!("    Message CID 1: {}", message_cid);
    println!("    Message CID 2: {}", message_cid2);
    println!("    Different: {}", message_cid != message_cid2);
    
    Ok(())
}

/// Helper function to generate CID for any serializable content
fn generate_cid<T: Serialize>(content: &T) -> Result<Cid, Box<dyn std::error::Error>> {
    // Encode with DAG-JSON
    let encoded = DagJsonCodec::encode(content)?;
    
    // Hash with BLAKE3
    let hash = blake3::hash(&encoded);
    let hash_bytes = hash.as_bytes();
    
    // Create multihash with BLAKE3 code (0x1e)
    let mut multihash_bytes = Vec::new();
    multihash_bytes.push(0x1e); // BLAKE3-256 code
    multihash_bytes.push(hash_bytes.len() as u8);
    multihash_bytes.extend_from_slice(hash_bytes);
    
    let mh = multihash::Multihash::from_bytes(&multihash_bytes)?;
    let cid = Cid::new_v1(0x0129, mh); // 0x0129 is DAG-JSON codec
    
    Ok(cid)
}