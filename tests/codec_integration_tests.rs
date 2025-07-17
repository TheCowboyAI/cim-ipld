//! Codec Integration Tests
//!
//! Tests for IPLD codec functionality and interoperability

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
    id: String,
    value: i32,
    nested: NestedData,
    list: Vec<String>,
    map: HashMap<String, i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct NestedData {
    field1: String,
    field2: bool,
}

impl TestData {
    fn example() -> Self {
        let mut map = HashMap::new();
        map.insert("one".to_string(), 1);
        map.insert("two".to_string(), 2);
        map.insert("three".to_string(), 3);
        
        TestData {
            id: "test-123".to_string(),
            value: 42,
            nested: NestedData {
                field1: "nested value".to_string(),
                field2: true,
            },
            list: vec!["a".to_string(), "b".to_string(), "c".to_string()],
            map,
        }
    }
}

#[test]
fn test_dag_json_codec() {
    let data = TestData::example();
    
    // Encode
    let encoded = DagJsonCodec::encode(&data).unwrap();
    assert!(!encoded.is_empty());
    
    // Decode
    let decoded: TestData = DagJsonCodec::decode(&encoded).unwrap();
    assert_eq!(data, decoded);
    
    // Test with trait methods
    let encoded2 = data.to_dag_json().unwrap();
    assert_eq!(encoded, encoded2);
    
    // Pretty print
    let pretty = data.to_dag_json_pretty().unwrap();
    assert!(pretty.contains("  "));  // Has indentation
    assert!(pretty.len() > encoded.len());  // Pretty is longer
}

#[test]
fn test_dag_cbor_codec() {
    let data = TestData::example();
    
    // Encode
    let encoded = DagCborCodec::encode(&data).unwrap();
    assert!(!encoded.is_empty());
    
    // Decode
    let decoded: TestData = DagCborCodec::decode(&encoded).unwrap();
    assert_eq!(data, decoded);
    
    // Test with trait method
    let encoded2 = data.to_dag_cbor().unwrap();
    assert_eq!(encoded, encoded2);
    
    // CBOR should be more compact than JSON
    let json_encoded = data.to_dag_json().unwrap();
    assert!(encoded.len() < json_encoded.len());
}

#[test]
fn test_raw_codec() {
    // Raw codec doesn't have encode/decode methods
    // It's just a marker for raw binary data
    let raw_codec = RawCodec;
    assert_eq!(raw_codec.code(), 0x55);
    assert_eq!(raw_codec.name(), "raw");
}

#[test]
fn test_standard_json_codec() {
    // JSON codec doesn't have encode/decode methods
    // It's just a marker for JSON data
    let json_codec = JsonCodec;
    assert_eq!(json_codec.code(), 0x0200);
    assert_eq!(json_codec.name(), "json");
}

#[test]
fn test_codec_comparisons() {
    let data = TestData::example();
    
    // DAG-JSON encoding
    let dag_json = DagJsonCodec::encode(&data).unwrap();
    
    // Decode and verify
    let decoded_dj: TestData = DagJsonCodec::decode(&dag_json).unwrap();
    assert_eq!(data, decoded_dj);
}

#[test]
fn test_alchemist_json_codec() {
    use cim_ipld::codec_types::AlchemistConfig;
    
    // Alchemist codec doesn't have encode/decode methods
    // It's just a marker for Alchemist JSON content
    let alchemist = AlchemistJsonCodec;
    assert_eq!(alchemist.code(), 0x340000);
    assert_eq!(alchemist.name(), "cim-alchemist-json");
    
    // To encode Alchemist config, use DAG-JSON
    let config = AlchemistConfig {
        version: "1.0".to_string(),
        domains: vec![],
        infrastructure: cim_ipld::codec_types::InfrastructureConfig {
            nats_url: "nats://localhost:4222".to_string(),
            storage: cim_ipld::codec_types::StorageConfig {
                backend: "memory".to_string(),
                options: Default::default(),
            },
        },
        metadata: Default::default(),
    };
    
    let encoded = DagJsonCodec::encode(&config).unwrap();
    let decoded: AlchemistConfig = DagJsonCodec::decode(&encoded).unwrap();
    assert_eq!(config.version, decoded.version);
}

#[test]
fn test_workflow_graph_json_codec() {
    use cim_ipld::codec_types::{WorkflowGraph, WorkflowNode, WorkflowMetadata, Position};
    
    // Workflow codec doesn't have encode/decode methods
    let workflow_codec = WorkflowGraphJsonCodec;
    assert_eq!(workflow_codec.code(), 0x340001);
    assert_eq!(workflow_codec.name(), "cim-workflow-graph-json");
    
    // Create a workflow using the proper types
    let workflow = WorkflowGraph {
        id: "wf-001".to_string(),
        name: "Data Processing Workflow".to_string(),
        nodes: vec![
            WorkflowNode {
                id: "n1".to_string(),
                node_type: "input".to_string(),
                label: "Input Node".to_string(),
                position: Position { x: 0.0, y: 0.0, z: None },
                data: Default::default(),
            },
        ],
        edges: vec![],
        metadata: WorkflowMetadata {
            created_at: 1234567890,
            updated_at: 1234567890,
            version: "1.0".to_string(),
            tags: vec![],
        },
    };
    
    let encoded = DagJsonCodec::encode(&workflow).unwrap();
    let decoded: WorkflowGraph = DagJsonCodec::decode(&encoded).unwrap();
    assert_eq!(workflow.id, decoded.id);
}

#[test]
fn test_context_graph_json_codec() {
    use cim_ipld::codec_types::{ContextGraph, Entity, ContextMetadata};
    
    // Context codec doesn't have encode/decode methods
    let context_codec = ContextGraphJsonCodec;
    assert_eq!(context_codec.code(), 0x340002);
    assert_eq!(context_codec.name(), "cim-context-graph-json");
    
    // Create a context graph using proper types
    let context = ContextGraph {
        id: "ctx-001".to_string(),
        context: "user-session".to_string(),
        entities: vec![
            Entity {
                id: "e1".to_string(),
                entity_type: "user".to_string(),
                attributes: Default::default(),
            },
        ],
        relationships: vec![],
        metadata: ContextMetadata {
            domain: "session-management".to_string(),
            version: "1.0".to_string(),
            created_at: 1234567890,
            tags: vec![],
        },
    };
    
    let encoded = DagJsonCodec::encode(&context).unwrap();
    let decoded: ContextGraph = DagJsonCodec::decode(&encoded).unwrap();
    assert_eq!(context.id, decoded.id);
}

#[test]
fn test_codec_registry() {
    let registry = CodecRegistry::new();
    
    // Test standard codecs
    assert!(registry.contains(0x71)); // DAG-CBOR
    assert!(registry.contains(0x0129)); // DAG-JSON
    assert!(registry.contains(0x55)); // Raw
    
    // Test CIM codecs
    assert!(registry.contains(0x340000)); // Alchemist
    assert!(registry.contains(0x340001)); // Workflow Graph
    assert!(registry.contains(0x340002)); // Context Graph
    
    // Get codec by code
    let dag_cbor = registry.get(0x71).unwrap();
    assert_eq!(dag_cbor.code(), 0x71);
    assert_eq!(dag_cbor.name(), "dag-cbor");
    
    // List all codec codes
    let codes = registry.codes();
    assert!(codes.len() >= 7); // At least the base codecs
}

#[test]
fn test_error_handling() {
    // Invalid JSON should fail
    let invalid_json = b"{ not valid json";
    let result: Result<serde_json::Value, _> = DagJsonCodec::decode(invalid_json);
    assert!(result.is_err());
    
    // Invalid CBOR should fail
    let invalid_cbor = vec![0xFF, 0xFF, 0xFF];
    let result: Result<serde_json::Value, _> = DagCborCodec::decode(&invalid_cbor);
    assert!(result.is_err());
    
    // Empty data
    let empty = vec![];
    let result: Result<serde_json::Value, _> = DagJsonCodec::decode(&empty);
    assert!(result.is_err());
}

#[test]
fn test_complex_data_structures() {
    use serde_json::json;
    
    // Complex nested data
    let complex_data = json!({
        "users": [
            {"id": 1, "name": "Alice", "roles": ["admin", "user"]},
            {"id": 2, "name": "Bob", "roles": ["user"]}
        ],
        "metadata": {
            "version": "1.0",
            "created": "2024-01-01",
            "tags": ["test", "complex", "data"]
        },
        "settings": {
            "feature_flags": {
                "new_ui": true,
                "beta_features": false
            }
        }
    });
    
    // Test with DAG-JSON
    let json_encoded = DagJsonCodec::encode(&complex_data).unwrap();
    let json_decoded: serde_json::Value = DagJsonCodec::decode(&json_encoded).unwrap();
    assert_eq!(complex_data, json_decoded);
    
    // Test with DAG-CBOR
    let cbor_encoded = DagCborCodec::encode(&complex_data).unwrap();
    let cbor_decoded: serde_json::Value = DagCborCodec::decode(&cbor_encoded).unwrap();
    assert_eq!(complex_data, cbor_decoded);
}

#[test]
fn test_codec_efficiency() {
    let data = TestData::example();
    
    // Compare encoding sizes
    let codecs: Vec<(&str, Box<dyn Fn(&TestData) -> Vec<u8>>)> = vec![
        ("DAG-JSON", Box::new(|d| DagJsonCodec::encode(d).unwrap())),
        ("DAG-CBOR", Box::new(|d| DagCborCodec::encode(d).unwrap())),
    ];
    
    println!("\nCodec efficiency comparison:");
    for (name, encoder) in &codecs {
        let encoded = encoder(&data);
        println!("{}: {} bytes", name, encoded.len());
        
        // Verify round-trip
        let decoded: TestData = match *name {
            "DAG-JSON" => DagJsonCodec::decode(&encoded).unwrap(),
            "DAG-CBOR" => DagCborCodec::decode(&encoded).unwrap(),
            _ => panic!("Unknown codec"),
        };
        assert_eq!(data, decoded);
    }
}

#[test]
fn test_special_characters() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct SpecialData {
        unicode: String,
        emojis: String,
        special_chars: String,
    }
    
    let data = SpecialData {
        unicode: "Hello 世界 🌍".to_string(),
        emojis: "🚀🎉🔥💯".to_string(),
        special_chars: "!@#$%^&*()[]{}|\\:;\"'<>,.?/~`".to_string(),
    };
    
    // Test with DAG-JSON
    let json_encoded = DagJsonCodec::encode(&data).unwrap();
    let json_decoded: SpecialData = DagJsonCodec::decode(&json_encoded).unwrap();
    assert_eq!(data, json_decoded);
    
    // Test with DAG-CBOR
    let cbor_encoded = DagCborCodec::encode(&data).unwrap();
    let cbor_decoded: SpecialData = DagCborCodec::decode(&cbor_encoded).unwrap();
    assert_eq!(data, cbor_decoded);
}