// Copyright 2025 Cowboy AI, LLC.

//! Example demonstrating index persistence with encryption at rest
//!
//! This example shows how to:
//! 1. Create an in-memory index with persistence backing
//! 2. Use encryption for data at rest
//! 3. Save and restore index state
//! 4. Use encrypted CID wrappers

use cim_ipld::{
    content_types::{
        indexing::{ContentIndex, SearchQuery},
        persistence::{IndexPersistence, NatsEncryptionConfig},
        encryption::{ContentEncryption, EncryptionAlgorithm},
        DocumentMetadata, ImageMetadata,
        codec,
    },
    ContentType,
};
use async_nats::jetstream;
use cid::Cid;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("CIM-IPLD Index Persistence Demo");
    println!("================================\n");

    // Connect to NATS (must be running locally)
    println!("Connecting to NATS...");
    let client = match async_nats::connect("nats://localhost:4222").await {
        Ok(client) => client,
        Err(_) => {
            println!("Error: NATS server not running. Please start NATS with:");
            println!("  docker run -p 4222:4222 nats:latest");
            return Ok(());
        }
    };
    let jetstream = jetstream::new(client);

    // Demonstrate different encryption options
    demonstrate_application_encryption(jetstream.clone()).await?;
    demonstrate_nats_native_encryption().await?;
    demonstrate_encrypted_cid_wrapper(jetstream.clone()).await?;

    Ok(())
}

async fn demonstrate_application_encryption(
    jetstream: jetstream::Context,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n1. Application-Level Encryption");
    println!("-------------------------------");

    // Generate a secure encryption key
    println!("Generating encryption key...");
    let encryption_key = ContentEncryption::generate_key(EncryptionAlgorithm::ChaCha20Poly1305);
    println!("Key generated (32 bytes)");

    // Create persistence layer with encryption
    let persistence = Arc::new(
        IndexPersistence::new(jetstream, Some(encryption_key.clone()), false).await?
    );

    // Create index with persistence
    let index = ContentIndex::with_persistence(persistence.clone());

    // Index some content
    println!("\nIndexing content...");
    
    let doc1 = DocumentMetadata {
        title: Some("Quarterly Report 2024".to_string()),
        author: Some("Finance Team".to_string()),
        tags: vec!["finance".to_string(), "report".to_string(), "2024".to_string()],
        ..Default::default()
    };
    let cid1 = generate_test_cid(1);
    index.index_document(cid1, &doc1, Some("Revenue increased by 25%...")).await?;
    println!("  - Indexed document: {}", doc1.title.as_ref().unwrap());

    let doc2 = DocumentMetadata {
        title: Some("Technical Architecture".to_string()),
        author: Some("Engineering".to_string()),
        tags: vec!["technical".to_string(), "architecture".to_string()],
        ..Default::default()
    };
    let cid2 = generate_test_cid(2);
    index.index_document(cid2, &doc2, Some("Microservices design...")).await?;
    println!("  - Indexed document: {}", doc2.title.as_ref().unwrap());

    let img1 = ImageMetadata {
        tags: vec!["diagram".to_string(), "architecture".to_string()],
        width: Some(1920),
        height: Some(1080),
        format: Some("PNG".to_string()),
        ..Default::default()
    };
    let cid3 = generate_test_cid(3);
    index.index_image(cid3, &img1, ContentType::Custom(codec::PNG)).await?;
    println!("  - Indexed image: architecture diagram");

    // Persist encrypted index
    println!("\nPersisting encrypted index to NATS KV...");
    index.persist().await?;
    println!("Index persisted with ChaCha20-Poly1305 encryption");

    // Simulate application restart - create new index and load
    println!("\nSimulating application restart...");
    let new_index = ContentIndex::with_persistence(persistence);
    new_index.load_from_persistence().await?;
    println!("Index loaded from encrypted storage");

    // Search to verify
    println!("\nSearching restored index...");
    let query = SearchQuery {
        text: Some("architecture".to_string()),
        ..Default::default()
    };
    let results = new_index.search(&query).await?;
    println!("Found {} results for 'architecture'", results.len());
    for result in results {
        println!("  - CID: {} (score: {:.2})", result.cid, result.score);
    }

    // Show stats
    let stats = new_index.stats().await;
    println!("\nIndex Statistics:");
    println!("  - Documents: {}", stats.total_documents);
    println!("  - Images: {}", stats.total_images);
    println!("  - Unique words: {}", stats.unique_words);
    println!("  - Unique tags: {}", stats.unique_tags);

    Ok(())
}

async fn demonstrate_nats_native_encryption() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n2. NATS Native Encryption");
    println!("-------------------------");

    let config = NatsEncryptionConfig::default();
    println!("NATS encryption configuration:");
    println!("  - Server-side encryption: {}", config.server_encryption);
    println!("  - Algorithm: {}", config.algorithm);
    println!("  - Key rotation: {} days", config.key_rotation_days);
    
    println!("\nNote: NATS native encryption requires server configuration:");
    println!("  - Enable encryption in nats-server.conf");
    println!("  - Configure key management");
    println!("  - Set up automatic key rotation");

    Ok(())
}

async fn demonstrate_encrypted_cid_wrapper(
    jetstream: jetstream::Context,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n3. Encrypted CID Wrapper");
    println!("------------------------");

    // Generate encryption key
    let encryption_key = ContentEncryption::generate_key(EncryptionAlgorithm::Aes256Gcm);
    
    // Create persistence with encryption
    let persistence = IndexPersistence::new(jetstream, Some(encryption_key), false).await?;

    // Create some sensitive metadata
    let sensitive_metadata = r#"{
        "classification": "confidential",
        "department": "research",
        "project": "next-gen-ai",
        "clearance_level": 4
    }"#;

    let cid = generate_test_cid(100);
    println!("Original CID: {}", cid);

    // Create encrypted wrapper
    let wrapper = persistence
        .create_encrypted_wrapper(&cid, sensitive_metadata.as_bytes())
        .await?;

    println!("\nEncrypted wrapper created:");
    println!("  - CID (unencrypted): {}", wrapper.cid);
    println!("  - Metadata size: {} bytes", sensitive_metadata.len());
    println!("  - Encrypted size: {} bytes", wrapper.encrypted_metadata.len());
    println!("  - IV size: {} bytes", wrapper.iv.len());
    println!("  - Key hash: {}...", &wrapper.key_hash[..16]);

    // Decrypt metadata
    let decrypted = persistence.decrypt_wrapper_metadata(&wrapper).await?;
    let decrypted_str = String::from_utf8(decrypted)?;
    println!("\nDecrypted metadata:");
    println!("{}", decrypted_str);

    println!("\nBenefits of encrypted CID wrapper:");
    println!("  - CID remains unencrypted for content retrieval");
    println!("  - Metadata is encrypted at rest");
    println!("  - Supports key rotation via key_hash");
    println!("  - Additional authenticated data (AAD) prevents tampering");

    Ok(())
}

// Helper function to generate test CIDs
fn generate_test_cid(n: u8) -> Cid {
    use multihash::Multihash;
    use blake3::hash;
    
    let data = vec![n; 32];
    let hash_bytes = hash(&data);
    
    let multihash_bytes = {
        let mut bytes = Vec::new();
        bytes.push(0x1e); // BLAKE3
        bytes.push(32);   // hash size
        bytes.extend_from_slice(hash_bytes.as_bytes());
        bytes
    };
    
    let mh = Multihash::from_bytes(&multihash_bytes).unwrap();
    Cid::new_v1(0x55, mh) // 0x55 = raw codec
}