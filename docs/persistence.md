# Index Persistence with Encryption at Rest

This document describes the persistence layer for the in-memory indexing system and encryption at rest capabilities in CIM-IPLD.

## Overview

The persistence layer provides:
1. **NATS KV Store Integration**: Durable storage for index data using NATS Key-Value store
2. **Application-Level Encryption**: ChaCha20-Poly1305 and AES-256-GCM encryption for data at rest
3. **Encrypted CID Wrappers**: Unencrypted CIDs with encrypted metadata for efficient retrieval
4. **NATS Native Encryption**: Support for server-side encryption configuration

## Architecture

### Components

- **IndexPersistence**: Main persistence service that manages KV stores and encryption
- **ContentEncryption**: Encryption service supporting multiple algorithms
- **EncryptedCidWrapper**: Structure for storing unencrypted CIDs with encrypted metadata
- **NatsEncryptionConfig**: Configuration for NATS native encryption

### Data Stores

The persistence layer uses separate NATS KV stores for different index types:
- `cim-index-text`: Text search index (inverted word index)
- `cim-index-tags`: Tag-based index
- `cim-index-types`: Content type index
- `cim-index-metadata`: Metadata cache

## Usage

### Basic Setup

```rust
use cim_ipld::content_types::{
    indexing::ContentIndex,
    persistence::IndexPersistence,
    encryption::{ContentEncryption, EncryptionAlgorithm},
};
use async_nats::jetstream;

// Connect to NATS
let client = async_nats::connect("nats://localhost:4222").await?;
let jetstream = jetstream::new(client);

// Generate encryption key
let encryption_key = ContentEncryption::generate_key(EncryptionAlgorithm::ChaCha20Poly1305);

// Create persistence layer with encryption
let persistence = Arc::new(
    IndexPersistence::new(jetstream, Some(encryption_key), false).await?
);

// Create index with persistence
let index = ContentIndex::with_persistence(persistence);
```

### Persisting Index Data

```rust
// Index some content
index.index_document(cid, &metadata, Some("content")).await?;

// Persist to NATS (automatically encrypted)
index.persist().await?;
```

### Loading Persisted Index

```rust
// Create new index instance
let new_index = ContentIndex::with_persistence(persistence);

// Load from persistence
new_index.load_from_persistence().await?;
```

## Encryption

### Application-Level Encryption

The persistence layer supports three encryption algorithms:
- **AES-256-GCM**: Hardware-accelerated on many platforms
- **ChaCha20-Poly1305**: Fast in software implementations
- **XChaCha20-Poly1305**: Extended nonce variant for additional security

### Encrypted CID Wrapper

The `EncryptedCidWrapper` allows storing encrypted metadata while keeping the CID unencrypted for efficient content retrieval:

```rust
// Create encrypted wrapper
let wrapper = persistence.create_encrypted_wrapper(&cid, metadata).await?;

// Wrapper contains:
// - Unencrypted CID for content retrieval
// - Encrypted metadata
// - Nonce/IV for decryption
// - Key hash for rotation detection
```

### Key Rotation

The system supports key rotation:

```rust
use cim_ipld::content_types::encryption::KeyRotation;

let rotation = KeyRotation::new(old_key, new_key, algorithm)?;
let rotated_data = rotation.rotate(&encrypted_data)?;
```

## NATS Native Encryption

For environments requiring server-side encryption:

```rust
let config = NatsEncryptionConfig {
    server_encryption: true,
    algorithm: "AES-256-GCM".to_string(),
    key_rotation_days: 90,
};
```

Note: NATS native encryption requires server configuration in `nats-server.conf`.

## Security Considerations

1. **Key Management**: Store encryption keys securely (e.g., environment variables, key management service)
2. **Key Rotation**: Implement regular key rotation for enhanced security
3. **Backup Encryption**: Ensure backups are also encrypted
4. **Transport Security**: Use TLS for NATS connections in production

## Performance Considerations

1. **Encryption Overhead**: ChaCha20-Poly1305 is recommended for software implementations
2. **Batch Operations**: Persist index changes in batches to reduce overhead
3. **Compression**: Data is compressed before encryption when beneficial
4. **Caching**: In-memory index serves as a cache for fast lookups

## Example Implementation

See `examples/persistence_demo.rs` for a complete working example demonstrating:
- Application-level encryption setup
- Index persistence and restoration
- Encrypted CID wrappers
- NATS native encryption configuration