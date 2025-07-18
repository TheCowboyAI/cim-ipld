# CIM-IPLD

Content-addressed storage for the Composable Information Machine using IPLD (InterPlanetary Linked Data).

# Composabable Information Machine (CIM)
CIM is a new way to realize a distributed system using and Event Driven Arcitecture created by Cowboy AI.

[![Rust](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE-MIT)
[![Tests](https://img.shields.io/badge/tests-206%20passing-green.svg)](docs/testing/test-report.md)

## Overview

CIM-IPLD provides a robust content-addressed storage system with comprehensive support for various content types, IPLD codecs, and distributed storage through NATS JetStream.

When using an Object Store, the "filename" becomes a CID. We can add Metadata by referencing the CID, 
add a replacement, or **linked** data by referencing the previous CID.

This allows us to distribute files into our Object Stores and retrieve them quite easily.

### Key Features

- **🗂️ Content Type Support**: Documents (PDF, DOCX, Markdown, Text), Images (JPEG, PNG, GIF, WebP), Audio (MP3, WAV, FLAC, AAC, OGG), Video (MP4, MOV, MKV, AVI)
- **🔗 IPLD Integration**: DAG-JSON, DAG-CBOR, Raw, and custom CIM-specific codecs with canonical CID generation
- **⛓️ Content Chains**: Cryptographically linked content with integrity verification
- **💾 NATS Object Store**: Distributed storage backend with LRU caching and domain partitioning
- **🔄 Transformation Pipeline**: Convert between formats while preserving CID traceability
- **🔐 Security**: At-rest encryption with AES-256-GCM, ChaCha20-Poly1305, and XChaCha20-Poly1305
- **🔍 Content Indexing**: Full-text search with metadata indexing and persistence

## Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
cim-ipld = "0.5.0"
```

### Basic Usage

```rust
use cim_ipld::{ContentChain, TextDocument, DocumentMetadata};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a content chain
    let mut chain = ContentChain::new();
    
    // Add documents to the chain
    let doc = TextDocument {
        content: "Hello, IPLD!".to_string(),
        metadata: DocumentMetadata {
            title: Some("My Document".to_string()),
            ..Default::default()
        },
    };
    
    let cid = chain.add_content(&doc).await?;
    println!("Content CID: {}", cid);
    
    // Verify chain integrity
    assert!(chain.verify().is_ok());
    
    Ok(())
}
```

## Documentation

### 📚 User Documentation
- [Developer Guide](docs/guides/developer-guide.md) - Comprehensive guide for using CIM-IPLD
- [API Reference](docs/api/api-reference.md) - Complete API documentation
- [Migration Guide](docs/guides/migration-guide.md) - Migrating from other storage systems

### 🏗️ Architecture
- [System Architecture](docs/architecture/system-architecture.md) - Design and architecture overview
- [CID Calculation](docs/architecture/cid-calculation.md) - How CIDs are generated
- [Domain Partitioning](docs/architecture/domain-partitioning.md) - Content routing strategy

### 🚀 Features
- [Content Types](docs/features/content-types.md) - Supported file formats and type detection
- [IPLD Codecs](docs/features/ipld-codecs.md) - Available IPLD codec implementations
- [Persistence & Encryption](docs/features/persistence-encryption.md) - Storage and security features

### 🧪 Testing
- [Test Report](docs/testing/test-report.md) - Comprehensive test coverage report
- [Test Guide](docs/testing/test-guide.md) - Running and writing tests

### 📋 Project
- [Changelog](CHANGELOG.md) - Version history and changes
- [Implementation Status](docs/project/implementation-status.md) - Current feature completion status

## Examples

See the [examples/](examples/) directory for:
- `basic_usage.rs` - Getting started with CIM-IPLD
- `content_types_demo.rs` - Working with different content types
- `persistence_demo.rs` - Using NATS persistence
- `transformation_demo.rs` - Content transformation pipeline

## Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                     Application Layer                    │
├─────────────────────────────────────────────────────────┤
│                    CIM-IPLD Core API                    │
├─────────────┬─────────────┬─────────────┬──────────────┤
│   Content   │    Chain    │    IPLD     │ Transform    │
│   Types     │ Validation  │   Codecs    │ Pipeline     │
├─────────────┴─────────────┴─────────────┴──────────────┤
│                  Storage Abstraction                     │
├─────────────────────────────────────────────────────────┤
│              NATS JetStream Object Store                │
└─────────────────────────────────────────────────────────┘
```

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run with coverage
cargo tarpaulin --out Html

# Run benchmarks
cargo bench
```

### Building Documentation

```bash
# Build and open API docs
cargo doc --open
```

## Contributing

Contributions are welcome! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

## License

This project is licensed under the MIT License - see the [LICENSE-MIT](LICENSE-MIT) file for details.

## Acknowledgments

Built with:
- [IPLD](https://ipld.io/) - InterPlanetary Linked Data
- [NATS](https://nats.io/) - High-performance messaging system
- [Rust](https://www.rust-lang.org/) - Systems programming language


---
Copyright 2025 Cowboy AI, LLC.