# NATS Persistence and Encryption at Rest

## Overview

This document describes the persistence and encryption capabilities added to CIM-IPLD for storing the in-memory index durably with encryption at rest.

## Features Implemented

### 1. NATS KV Store Persistence

The `IndexPersistence` service provides durable storage for the in-memory content index using NATS Key-Value stores:

- **Text Index**: Full-text search index with inverted word mapping
- **Tag Index**: Tag-based content discovery 
- **Type Index**: Content type categorization
- **Metadata Cache**: Complete metadata storage

Each index is stored in a separate KV bucket for optimal performance and isolation.

### 2. Encryption at Rest

Two encryption approaches are supported:

#### Application-Level Encryption

The `ContentEncryption` module provides:

- **Multiple algorithms**: AES-256-GCM, ChaCha20-Poly1305, XChaCha20-Poly1305
- **Authenticated encryption**: AEAD prevents tampering
- **Key management**: Generation, derivation, and rotation
- **Transparent operation**: Automatic encrypt/decrypt on store/load

#### NATS Native Encryption

Configuration support for server-side encryption:

```rust
let config = NatsEncryption {
    enabled: true,
    key_file: Some("/path/to/key.pem".into()),
    cipher: "AES256".to_string(),
};
```

### 3. Encrypted CID Wrapper

The design supports encrypted CIDs containing unencrypted CIDs:

```rust
#[derive(Serialize, Deserialize)]
pub struct EncryptedCidWrapper {
    /// The unencrypted CID for content retrieval
    pub cid: Cid,
    /// Encrypted metadata
    pub encrypted_metadata: EncryptedData,
    /// Additional authenticated data
    pub aad: Vec<u8>,
}
```

This allows:
- Efficient content retrieval by CID
- Protected metadata and relationships
- Verification of encryption integrity

## Usage

### Basic Persistence

```rust
// Create persistence service
let persistence = IndexPersistence::new(jetstream, None, false).await?;

// Create index with persistence
let index = ContentIndex::with_persistence(Arc::new(persistence));

// Index content (automatically persisted)
index.index_document(cid, &metadata, Some("content")).await?;

// Load from persistence on restart
index.load_from_persistence().await?;
```

### With Encryption

```rust
// Generate encryption key
let key = ContentEncryption::generate_key(EncryptionAlgorithm::ChaCha20Poly1305);

// Create encrypted persistence
let persistence = IndexPersistence::new(jetstream, Some(key), false).await?;

// Use as normal - encryption is transparent
let index = ContentIndex::with_persistence(Arc::new(persistence));
```

### Key Rotation

```rust
// Rotate to new key
let new_key = ContentEncryption::generate_key(EncryptionAlgorithm::AES256GCM);
persistence.rotate_encryption_key(new_key).await?;
```

## Implementation Details

### Persistence Architecture

```
IndexPersistence
├── KV Stores
│   ├── text_index_v1
│   ├── tag_index_v1
│   ├── type_index_v1
│   └── metadata_cache_v1
├── Encryption Layer
│   ├── Algorithm selection
│   ├── Key management
│   └── Transparent enc/dec
└── Backup/Restore
    └── Named snapshots
```

### Security Considerations

1. **Key Storage**: Application is responsible for secure key management
2. **Algorithm Choice**: 
   - AES-256-GCM: Hardware accelerated, FIPS compliant
   - ChaCha20-Poly1305: Fast software implementation
   - XChaCha20-Poly1305: Extended nonce for key reuse safety
3. **Authentication**: All encryption uses AEAD to prevent tampering
4. **Key Rotation**: Supports rotation without data loss

### Performance

- **Async operations**: Non-blocking persistence
- **Batch operations**: Efficient bulk updates
- **Compression**: Optional zstd compression before encryption
- **Caching**: In-memory cache with persistent backing

## Testing

Run the included tests:

```bash
# Unit tests
cargo test persistence
cargo test encryption

# Integration example
cargo run --example persistence_demo
```

## Future Enhancements

1. **Streaming persistence**: For very large indexes
2. **Incremental updates**: Only persist changes
3. **Multi-region replication**: Via NATS clustering
4. **Hardware security modules**: For key management
5. **Searchable encryption**: Allow searching encrypted data