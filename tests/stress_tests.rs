//! Stress tests for CIM-IPLD
//!
//! These tests push the system to its limits to ensure it performs
//! correctly under heavy load and extreme conditions.

use cim_ipld::{
    ContentChain, TypedContent, ContentType,
    DagJsonCodec, DagCborCodec, CodecOperations,
};
use serde::{Serialize, Deserialize};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct StressTestData {
    id: u64,
    data: Vec<u8>,
    text: String,
}

impl TypedContent for StressTestData {
    const CODEC: u64 = 0x300200;
    const CONTENT_TYPE: ContentType = ContentType::Custom(0x300200);
}

#[test]
fn test_large_chain_stress() {
    const CHAIN_SIZE: usize = 1000;
    let mut chain = ContentChain::<StressTestData>::new();
    
    let start = Instant::now();
    
    // Build a large chain
    for i in 0..CHAIN_SIZE {
        let content = StressTestData {
            id: i as u64,
            data: vec![i as u8; 100], // 100 bytes of data
            text: format!("Item {} with some text content", i),
        };
        
        chain.append(content).unwrap();
    }
    
    let build_time = start.elapsed();
    println!("Built chain of {} items in {:?}", CHAIN_SIZE, build_time);
    
    // Validate the entire chain
    let validate_start = Instant::now();
    chain.validate().unwrap();
    let validate_time = validate_start.elapsed();
    println!("Validated chain in {:?}", validate_time);
    
    // Test items_since performance
    let mid_cid = &chain.items()[CHAIN_SIZE / 2].cid;
    let query_start = Instant::now();
    let items = chain.items_since(mid_cid).unwrap();
    let query_time = query_start.elapsed();
    println!("Retrieved {} items_since in {:?}", items.len(), query_time);
    
    assert_eq!(chain.len(), CHAIN_SIZE);
    assert_eq!(items.len(), CHAIN_SIZE / 2 - 1);
}

#[test]
fn test_large_content_stress() {
    // Test with increasingly large content
    let sizes = vec![1_000, 10_000, 100_000, 1_000_000]; // Up to 1MB
    
    for size in sizes {
        let content = StressTestData {
            id: 1,
            data: vec![0xFF; size],
            text: "x".repeat(size / 10),
        };
        
        let start = Instant::now();
        
        // Test serialization
        let bytes = content.to_bytes().unwrap();
        let ser_time = start.elapsed();
        
        // Test CID calculation
        let cid_start = Instant::now();
        let cid = content.calculate_cid().unwrap();
        let cid_time = cid_start.elapsed();
        
        // Test deserialization
        let deser_start = Instant::now();
        let _restored: StressTestData = StressTestData::from_bytes(&bytes).unwrap();
        let deser_time = deser_start.elapsed();
        
        println!(
            "Size {}: ser {:?}, cid {:?}, deser {:?}",
            size, ser_time, cid_time, deser_time
        );
        
        assert!(!cid.to_string().is_empty());
    }
}

#[test]
fn test_concurrent_chain_operations() {
    use std::sync::{Arc, Mutex};
    use std::thread;
    
    const THREADS: usize = 4;
    const ITEMS_PER_THREAD: usize = 250;
    
    let chain = Arc::new(Mutex::new(ContentChain::<StressTestData>::new()));
    let mut handles = vec![];
    
    let start = Instant::now();
    
    for thread_id in 0..THREADS {
        let chain_clone = Arc::clone(&chain);
        
        let handle = thread::spawn(move || {
            for i in 0..ITEMS_PER_THREAD {
                let content = StressTestData {
                    id: (thread_id * ITEMS_PER_THREAD + i) as u64,
                    data: vec![thread_id as u8; 50],
                    text: format!("Thread {} item {}", thread_id, i),
                };
                
                let mut chain = chain_clone.lock().unwrap();
                chain.append(content).unwrap();
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }
    
    let elapsed = start.elapsed();
    
    let chain = chain.lock().unwrap();
    assert_eq!(chain.len(), THREADS * ITEMS_PER_THREAD);
    assert!(chain.validate().is_ok());
    
    println!(
        "Concurrent append: {} items from {} threads in {:?}",
        chain.len(), THREADS, elapsed
    );
}

#[test]
fn test_codec_performance_comparison() {
    let test_data = StressTestData {
        id: 12345,
        data: vec![0xAB; 10_000], // 10KB of data
        text: "Performance test data with some meaningful content that could be compressed well".repeat(10),
    };
    
    const ITERATIONS: usize = 100;
    
    // Test DAG-JSON performance
    let json_start = Instant::now();
    for _ in 0..ITERATIONS {
        let encoded = test_data.to_dag_json().unwrap();
        let _decoded: StressTestData = DagJsonCodec::decode(&encoded).unwrap();
    }
    let json_time = json_start.elapsed();
    
    // Test DAG-CBOR performance
    let cbor_start = Instant::now();
    for _ in 0..ITERATIONS {
        let encoded = test_data.to_dag_cbor().unwrap();
        let _decoded: StressTestData = DagCborCodec::decode(&encoded).unwrap();
    }
    let cbor_time = cbor_start.elapsed();
    
    // Test sizes
    let json_size = test_data.to_dag_json().unwrap().len();
    let cbor_size = test_data.to_dag_cbor().unwrap().len();
    
    println!("Codec performance ({} iterations):", ITERATIONS);
    println!("  DAG-JSON: {:?}, size: {} bytes", json_time, json_size);
    println!("  DAG-CBOR: {:?}, size: {} bytes", cbor_time, cbor_size);
    println!("  Size ratio: {:.2}x", json_size as f64 / cbor_size as f64);
    
    // CBOR should generally be smaller
    assert!(cbor_size < json_size);
}

#[test]
fn test_memory_stress() {
    // Test that we can handle many small chains without issues
    let mut chains = Vec::new();
    const NUM_CHAINS: usize = 100;
    const ITEMS_PER_CHAIN: usize = 10;
    
    let start = Instant::now();
    
    for chain_id in 0..NUM_CHAINS {
        let mut chain = ContentChain::<StressTestData>::new();
        
        for item_id in 0..ITEMS_PER_CHAIN {
            let content = StressTestData {
                id: (chain_id * ITEMS_PER_CHAIN + item_id) as u64,
                data: vec![chain_id as u8; 100],
                text: format!("Chain {} item {}", chain_id, item_id),
            };
            
            chain.append(content).unwrap();
        }
        
        chains.push(chain);
    }
    
    let elapsed = start.elapsed();
    
    // Validate all chains
    for chain in &chains {
        assert!(chain.validate().is_ok());
        assert_eq!(chain.len(), ITEMS_PER_CHAIN);
    }
    
    println!(
        "Created {} chains with {} items each in {:?}",
        NUM_CHAINS, ITEMS_PER_CHAIN, elapsed
    );
}

#[test]
fn test_pathological_content() {
    let mut chain = ContentChain::<StressTestData>::new();
    
    // Test with various pathological cases
    let test_cases = vec![
        // Empty content
        StressTestData {
            id: 0,
            data: vec![],
            text: String::new(),
        },
        // Very repetitive content (good for compression)
        StressTestData {
            id: 1,
            data: vec![0x00; 1000],
            text: "a".repeat(1000),
        },
        // Random-like content (bad for compression)
        StressTestData {
            id: 2,
            data: (0..255).cycle().take(1000).map(|b| b as u8).collect(),
            text: format!("{:x}", rand::random::<u128>()).repeat(50),
        },
        // Unicode stress
        StressTestData {
            id: 3,
            data: vec![0xFF; 100],
            text: "🔥💯🚀".repeat(100) + &"नमस्ते".repeat(50) + &"您好".repeat(50),
        },
        // Nested JSON-like structure in text
        StressTestData {
            id: 4,
            data: vec![123; 100],
            text: r#"{"a":{"b":{"c":{"d":{"e":"deep"}}}}} \\n \r\n \t \0"#.repeat(10),
        },
    ];
    
    for content in test_cases {
        let result = chain.append(content.clone());
        assert!(result.is_ok(), "Failed to append pathological content: {:?}", content);
        
        // Verify CID calculation works
        let cid = content.calculate_cid().unwrap();
        assert!(!cid.to_string().is_empty());
        
        // Verify serialization roundtrip
        let bytes = content.to_bytes().unwrap();
        let restored: StressTestData = StressTestData::from_bytes(&bytes).unwrap();
        assert_eq!(content, restored);
    }
    
    assert!(chain.validate().is_ok());
    println!("Successfully handled {} pathological content cases", chain.len());
}

#[test]
#[ignore] // Only run when specifically requested due to time
fn test_extreme_chain_stress() {
    const EXTREME_SIZE: usize = 10_000;
    let mut chain = ContentChain::<StressTestData>::new();
    
    println!("Building extreme chain of {} items...", EXTREME_SIZE);
    let start = Instant::now();
    
    for i in 0..EXTREME_SIZE {
        if i % 1000 == 0 {
            println!("Progress: {}/{}", i, EXTREME_SIZE);
        }
        
        let content = StressTestData {
            id: i as u64,
            data: vec![(i % 256) as u8; 50],
            text: format!("Item {}", i),
        };
        
        chain.append(content).unwrap();
    }
    
    let build_time = start.elapsed();
    println!("Built extreme chain in {:?}", build_time);
    
    // Validate
    let validate_start = Instant::now();
    chain.validate().unwrap();
    let validate_time = validate_start.elapsed();
    println!("Validated extreme chain in {:?}", validate_time);
    
    // Performance should scale reasonably
    assert!(build_time < Duration::from_secs(60), "Chain building took too long");
    assert!(validate_time < Duration::from_secs(10), "Chain validation took too long");
}