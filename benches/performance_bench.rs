// Copyright 2025 Cowboy AI, LLC.

//! Performance benchmarks for CIM-IPLD
//!
//! Benchmarks for measuring performance of core operations.
//!
//! Run with: cargo bench

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use cim_ipld::{
    ChainedContent, ContentChain, TypedContent, ContentType,
    DagJsonCodec, DagCborCodec, CodecOperations,
};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

// Test content for benchmarks
#[derive(Debug, Clone, Serialize, Deserialize)]
struct BenchContent {
    id: String,
    data: Vec<u8>,
    metadata: HashMap<String, String>,
}

impl TypedContent for BenchContent {
    const CODEC: u64 = 0x0129; // DAG-JSON
    const CONTENT_TYPE: ContentType = ContentType::Custom(0x400000);
}

/// Generate test data of specified size
fn generate_bench_data(size: usize) -> Vec<u8> {
    vec![0xAB; size]
}

fn create_bench_content(size: usize) -> BenchContent {
    let mut metadata = HashMap::new();
    metadata.insert("size".to_string(), size.to_string());
    metadata.insert("type".to_string(), "benchmark".to_string());
    
    BenchContent {
        id: format!("bench-{}", size),
        data: generate_bench_data(size),
        metadata,
    }
}

fn codec_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("codecs");
    
    // Small content (1KB)
    let small_content = create_bench_content(1024);
    
    group.bench_function("dag_json_encode_small", |b| {
        b.iter(|| {
            let encoded = DagJsonCodec::encode(&small_content).unwrap();
            black_box(encoded);
        })
    });
    
    group.bench_function("dag_cbor_encode_small", |b| {
        b.iter(|| {
            let encoded = DagCborCodec::encode(&small_content).unwrap();
            black_box(encoded);
        })
    });
    
    // Medium content (100KB)
    let medium_content = create_bench_content(102_400);
    
    group.bench_function("dag_json_encode_medium", |b| {
        b.iter(|| {
            let encoded = DagJsonCodec::encode(&medium_content).unwrap();
            black_box(encoded);
        })
    });
    
    group.bench_function("dag_cbor_encode_medium", |b| {
        b.iter(|| {
            let encoded = DagCborCodec::encode(&medium_content).unwrap();
            black_box(encoded);
        })
    });
    
    // Decode benchmarks
    let small_json = DagJsonCodec::encode(&small_content).unwrap();
    let small_cbor = DagCborCodec::encode(&small_content).unwrap();
    
    group.bench_function("dag_json_decode_small", |b| {
        b.iter(|| {
            let decoded: BenchContent = DagJsonCodec::decode(&small_json).unwrap();
            black_box(decoded);
        })
    });
    
    group.bench_function("dag_cbor_decode_small", |b| {
        b.iter(|| {
            let decoded: BenchContent = DagCborCodec::decode(&small_cbor).unwrap();
            black_box(decoded);
        })
    });
    
    group.finish();
}

fn chain_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("chains");
    
    group.bench_function("chain_append", |b| {
        b.iter(|| {
            let mut chain = ContentChain::new();
            for _i in 0..10 {
                let content = create_bench_content(1024);
                chain.append(content).unwrap();
            }
            black_box(chain);
        })
    });
    
    group.bench_function("chained_content_new", |b| {
        let content = create_bench_content(1024);
        let first = ChainedContent::new(content.clone(), None).unwrap();
        
        b.iter(|| {
            let chained = ChainedContent::new(content.clone(), Some(&first)).unwrap();
            black_box(chained);
        })
    });
    
    group.bench_function("chain_validate", |b| {
        let mut chain = ContentChain::new();
        for _i in 0..10 {
            let content = create_bench_content(1024);
            chain.append(content).unwrap();
        }
        
        b.iter(|| {
            let result = chain.validate().unwrap();
            black_box(result);
        })
    });
    
    group.finish();
}

fn cid_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("cids");
    
    group.bench_function("cid_creation", |b| {
        b.iter(|| {
            let content = create_bench_content(1024);
            let chained = ChainedContent::new(content, None).unwrap();
            black_box(&chained.cid);
        })
    });
    
    group.finish();
}

fn serialization_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization");
    
    // Complex nested structure
    let mut complex_content = BenchContent {
        id: "complex".to_string(),
        data: generate_bench_data(1024),
        metadata: HashMap::new(),
    };
    
    for i in 0..100 {
        complex_content.metadata.insert(
            format!("key_{}", i),
            format!("value_{}", i),
        );
    }
    
    group.bench_function("complex_to_dag_json", |b| {
        b.iter(|| {
            let json = complex_content.to_dag_json().unwrap();
            black_box(json);
        })
    });
    
    group.bench_function("complex_to_dag_cbor", |b| {
        b.iter(|| {
            let cbor = complex_content.to_dag_cbor().unwrap();
            black_box(cbor);
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    codec_benchmarks,
    chain_benchmarks,
    cid_benchmarks,
    serialization_benchmarks
);
criterion_main!(benches);