//! IPLD Codecs Demonstration
//!
//! This example demonstrates the various IPLD codecs available in CIM-IPLD,
//! including standard IPLD codecs and CIM-specific JSON types.

use cim_ipld::{
    CimCodec, CodecRegistry,
    DagCborCodec, DagJsonCodec, RawCodec, JsonCodec,
    AlchemistJsonCodec, WorkflowGraphJsonCodec, ContextGraphJsonCodec,
    CodecOperations,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestData {
    name: String,
    value: i32,
    active: bool,
    tags: Vec<String>,
    metadata: HashMap<String, String>,
}

impl TestData {
    fn example() -> Self {
        let mut metadata = HashMap::new();
        metadata.insert("version".to_string(), "1.0".to_string());
        metadata.insert("author".to_string(), "demo".to_string());
        
        TestData {
            name: "test-object".to_string(),
            value: 42,
            active: true,
            tags: vec!["demo".to_string(), "test".to_string()],
            metadata,
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== IPLD Codecs Demonstration ===\n");
    
    // Demonstrate different codecs
    demo_dag_json()?;
    demo_dag_cbor()?;
    demo_raw()?;
    demo_json()?;
    demo_cim_json_codecs()?;
    demo_codec_registry()?;
    
    Ok(())
}

fn demo_dag_json() -> Result<(), Box<dyn std::error::Error>> {
    println!("1. DAG-JSON Codec (0x0129):");
    
    let data = TestData::example();
    println!("  Original: {:?}", data);
    
    // Encode to DAG-JSON
    let encoded = DagJsonCodec::encode(&data)?;
    println!("  Encoded: {} bytes", encoded.len());
    
    // Show the JSON string
    let json_str = std::str::from_utf8(&encoded)?;
    println!("  JSON: {}", json_str);
    
    // Decode back
    let decoded: TestData = DagJsonCodec::decode(&encoded)?;
    assert_eq!(data, decoded);
    
    // Also available through trait
    let encoded2 = data.to_dag_json()?;
    assert_eq!(encoded, encoded2);
    
    // Pretty print version
    let pretty = data.to_dag_json_pretty()?;
    println!("\n  Pretty JSON:\n{}", pretty);
    
    println!("  ✓ Round-trip successful\n");
    
    Ok(())
}

fn demo_dag_cbor() -> Result<(), Box<dyn std::error::Error>> {
    println!("2. DAG-CBOR Codec (0x71):");
    
    let data = TestData::example();
    
    // Encode to DAG-CBOR
    let encoded = DagCborCodec::encode(&data)?;
    println!("  Encoded: {} bytes (binary format)", encoded.len());
    
    // Show hex representation
    println!("  Hex: {}", hex::encode(&encoded[..20.min(encoded.len())]));
    
    // Decode back
    let decoded: TestData = DagCborCodec::decode(&encoded)?;
    assert_eq!(data, decoded);
    
    // Also available through trait
    let encoded2 = data.to_dag_cbor()?;
    assert_eq!(encoded, encoded2);
    
    println!("  ✓ Round-trip successful\n");
    
    Ok(())
}

fn demo_raw() -> Result<(), Box<dyn std::error::Error>> {
    println!("3. Raw Codec (0x55):");
    
    // Raw codec doesn't have encode/decode methods
    // It's just a marker for raw binary data
    let raw_codec = RawCodec;
    println!("  Codec code: 0x{:02X}", raw_codec.code());
    println!("  Codec name: {}", raw_codec.name());
    println!("  Purpose: Marks raw binary data in IPLD");
    println!("  Note: Does not transform data\n");
    
    Ok(())
}

fn demo_json() -> Result<(), Box<dyn std::error::Error>> {
    println!("4. Standard JSON Codec (0x0200):");
    
    // JSON codec doesn't have encode/decode methods
    // It's just a marker for JSON data
    let json_codec = JsonCodec;
    println!("  Codec code: 0x{:04X}", json_codec.code());
    println!("  Codec name: {}", json_codec.name());
    println!("  Purpose: Marks standard JSON data");
    println!("  Note: Use DAG-JSON for IPLD-compatible JSON\n");
    
    Ok(())
}

fn demo_cim_json_codecs() -> Result<(), Box<dyn std::error::Error>> {
    println!("5. CIM-specific JSON Codecs:");
    
    // Alchemist JSON
    println!("\n  a) Alchemist JSON:");
    let alchemist = AlchemistJsonCodec;
    println!("     Code: 0x{:06X}", alchemist.code());
    println!("     Name: {}", alchemist.name());
    println!("     Purpose: Alchemist configuration format");
    
    // Workflow Graph JSON
    println!("\n  b) Workflow Graph JSON:");
    let workflow = WorkflowGraphJsonCodec;
    println!("     Code: 0x{:06X}", workflow.code());
    println!("     Name: {}", workflow.name());
    println!("     Purpose: Workflow graph structures");
    
    // Context Graph JSON
    println!("\n  c) Context Graph JSON:");
    let context = ContextGraphJsonCodec;
    println!("     Code: 0x{:06X}", context.code());
    println!("     Name: {}", context.name());
    println!("     Purpose: Context graph structures");
    
    println!("\n  Note: These are content type markers for specific JSON formats.");
    println!("        Use DAG-JSON for actual encoding/decoding.\n");
    
    // Example: Using proper types with DAG-JSON
    use cim_ipld::codec_types::{WorkflowGraph, WorkflowNode, WorkflowMetadata, Position};
    
    let workflow_data = WorkflowGraph {
        id: "wf-001".to_string(),
        name: "Example Workflow".to_string(),
        nodes: vec![
            WorkflowNode {
                id: "node-1".to_string(),
                node_type: "process".to_string(),
                label: "Process Data".to_string(),
                position: Position { x: 0.0, y: 0.0, z: None },
                data: Default::default(),
            },
        ],
        edges: vec![],
        metadata: WorkflowMetadata {
            created_at: 1234567890,
            updated_at: 1234567890,
            version: "1.0".to_string(),
            tags: vec!["example".to_string()],
        },
    };
    
    // Encode workflow data using DAG-JSON
    let encoded = DagJsonCodec::encode(&workflow_data)?;
    println!("  Example workflow encoded: {} bytes", encoded.len());
    
    Ok(())
}

fn demo_codec_registry() -> Result<(), Box<dyn std::error::Error>> {
    println!("6. Codec Registry:");
    
    let registry = CodecRegistry::new();
    
    // Check standard codecs
    println!("  Standard IPLD codecs:");
    let standard_codes = vec![
        (0x55, "raw"),
        (0x71, "dag-cbor"),
        (0x0129, "dag-json"),
        (0x0200, "json"),
    ];
    
    for (code, name) in standard_codes {
        if registry.contains(code) {
            let codec = registry.get(code).unwrap();
            println!("    ✓ 0x{:04X}: {} ({})", code, name, codec.name());
        }
    }
    
    // Check CIM-specific codecs
    println!("\n  CIM-specific codecs:");
    let cim_codes = vec![
        (0x340000, "alchemist"),
        (0x340001, "workflow-graph"),
        (0x340002, "context-graph"),
    ];
    
    for (code, desc) in cim_codes {
        if registry.contains(code) {
            let codec = registry.get(code).unwrap();
            println!("    ✓ 0x{:06X}: {} ({})", code, desc, codec.name());
        }
    }
    
    // List all registered codes
    let all_codes = registry.codes();
    println!("\n  Total registered codecs: {}", all_codes.len());
    
    Ok(())
}

// Utility function for hex encoding
mod hex {
    pub fn encode(data: &[u8]) -> String {
        data.iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>()
    }
}