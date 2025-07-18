// Copyright 2025 Cowboy AI, LLC.

//! Tests for index persistence with encryption

use cim_ipld::{
    content_types::{
        indexing::{ContentIndex, SearchQuery},
        persistence::IndexPersistence,
        encryption::ContentEncryption,
        DocumentMetadata, ImageMetadata,
        codec,
    },
    ContentType,
};
use async_nats::jetstream;
use cid::Cid;
use std::sync::Arc;

#[tokio::test]
#[ignore] // Requires NATS server running
async fn test_index_persistence_with_encryption() {
    // Connect to NATS
    let client = async_nats::connect("nats://localhost:4222").await.unwrap();
    let jetstream = jetstream::new(client);

    // Generate encryption key
    let encryption_key = ContentEncryption::generate_key(
        cim_ipld::content_types::encryption::EncryptionAlgorithm::ChaCha20Poly1305
    );

    // Create persistence layer with encryption
    let persistence = Arc::new(
        IndexPersistence::new(jetstream.clone(), Some(encryption_key), false)
            .await
            .unwrap()
    );

    // Create index with persistence
    let index = ContentIndex::with_persistence(persistence.clone());

    // Index some documents
    let doc_metadata = DocumentMetadata {
        title: Some("Test Document".to_string()),
        author: Some("Test Author".to_string()),
        tags: vec!["test".to_string(), "document".to_string()],
        ..Default::default()
    };

    let cid1 = Cid::default();
    index.index_document(cid1, &doc_metadata, Some("This is test content")).await.unwrap();

    // Index an image
    let img_metadata = ImageMetadata {
        tags: vec!["test".to_string(), "image".to_string()],
        width: Some(1920),
        height: Some(1080),
        ..Default::default()
    };

    let cid2 = Cid::default();
    index.index_image(cid2, &img_metadata, ContentType::Custom(codec::PNG)).await.unwrap();

    // Search to verify index
    let query = SearchQuery {
        text: Some("test".to_string()),
        ..Default::default()
    };

    let results = index.search(&query).await.unwrap();
    assert!(!results.is_empty());

    // Persist the index
    index.persist().await.unwrap();

    // Create a new index and load from persistence
    let new_index = ContentIndex::with_persistence(persistence);
    new_index.load_from_persistence().await.unwrap();

    // Verify data was loaded correctly
    let results2 = new_index.search(&query).await.unwrap();
    assert_eq!(results.len(), results2.len());
}

#[tokio::test]
#[ignore] // Requires NATS server running
async fn test_encrypted_cid_wrapper() {
    // Connect to NATS
    let client = async_nats::connect("nats://localhost:4222").await.unwrap();
    let jetstream = jetstream::new(client);

    // Generate encryption key
    let encryption_key = ContentEncryption::generate_key(
        cim_ipld::content_types::encryption::EncryptionAlgorithm::Aes256Gcm
    );

    // Create persistence layer with encryption
    let persistence = IndexPersistence::new(jetstream, Some(encryption_key), false)
        .await
        .unwrap();

    // Create encrypted wrapper
    let cid = Cid::default();
    let metadata = b"sensitive metadata";

    let wrapper = persistence.create_encrypted_wrapper(&cid, metadata).await.unwrap();
    
    // Verify CID is not encrypted
    assert_eq!(wrapper.cid, cid.to_string());

    // Decrypt metadata
    let decrypted = persistence.decrypt_wrapper_metadata(&wrapper).await.unwrap();
    assert_eq!(decrypted, metadata);
}

#[test]
fn test_nats_encryption_config() {
    use cim_ipld::content_types::persistence::NatsEncryptionConfig;

    let config = NatsEncryptionConfig::default();
    assert!(config.server_encryption);
    assert_eq!(config.algorithm, "AES-256-GCM");
    assert_eq!(config.key_rotation_days, 90);
}