# CIM-IPLD

IPLD (InterPlanetary Linked Data) implementation for Composable Information Machines (CIM).

## Overview

CIM-IPLD provides a content-addressed storage foundation for all CIM nodes, enabling:

- **Content-addressed storage** with CIDs (Content Identifiers)
- **Cryptographic integrity** through hash chains
- **Type-safe content handling** with custom codecs
- **Extensible architecture** for domain-specific types

## Features

- 🔗 **Chain Linking**: Create tamper-evident chains of content
- 🎯 **Type Safety**: Strongly typed content with compile-time guarantees
- 🔌 **Extensible**: Register custom codecs for your content types
- 🚀 **Performance**: BLAKE3 hashing for fast content addressing
- 📦 **Codec Registry**: Manage content types with codec identifiers

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
# From GitHub
cim-ipld = { git = "https://github.com/thecowboyai/cim-ipld" }

# From crates.io (once published)
cim-ipld = "0.1"
```

## Quick Start

### Basic Usage

```rust
use cim_ipld::{ChainedContent, ContentChain, TypedContent, ContentType};
use serde::{Serialize, Deserialize};

// Define your content type
#[derive(Serialize, Deserialize)]
struct MyEvent {
    id: String,
    action: String,
    timestamp: u64,
}

// Implement TypedContent
impl TypedContent for MyEvent {
    const CODEC: u64 = 0x300100; // Your codec in range 0x300000-0x3FFFFF
    const CONTENT_TYPE: ContentType = ContentType::Custom(0x300100);
}

// Create a chain
let mut chain = ContentChain::<MyEvent>::new();

// Add events
let event = MyEvent {
    id: "evt-001".to_string(),
    action: "user.login".to_string(),
    timestamp: 1234567890,
};

let chained = chain.append(event)?;
println!("Event CID: {}", chained.cid);
```

### Custom Codecs

```rust
use cim_ipld::{CimCodec, CodecRegistry};
use std::sync::Arc;

struct MyCustomCodec;

impl CimCodec for MyCustomCodec {
    fn code(&self) -> u64 {
        0x330000 // Your codec identifier
    }

    fn name(&self) -> &str {
        "my-custom-codec"
    }

    // Optionally override encode/decode for custom serialization
}

// Register your codec
let mut registry = CodecRegistry::new();
registry.register(Arc::new(MyCustomCodec))?;
```

## Content Types

CIM-IPLD defines standard content types:

| Type | Codec | Range |
|------|-------|-------|
| Event | 0x300000 | Core CIM types |
| Graph | 0x300001 | |
| Node | 0x300002 | |
| Edge | 0x300003 | |
| Command | 0x300004 | |
| Query | 0x300005 | |
| Markdown | 0x310000 | Document types |
| JSON | 0x310001 | |
| YAML | 0x310002 | |
| TOML | 0x310003 | |
| Image | 0x320000 | Media types |
| Video | 0x320001 | |
| Audio | 0x320002 | |
| Custom | 0x330000+ | Your types |

## Chain Validation

CIM-IPLD provides cryptographic chain validation:

```rust
// Validate entire chain
chain.validate()?;

// Get items since a specific CID
let recent = chain.items_since(&previous_cid)?;

// Detect tampering
let mut tampered = chain.items()[0].clone();
tampered.sequence = 999; // Modify
assert!(tampered.validate_chain(None).is_err()); // Validation fails
```

## Architecture

```
cim-ipld/
├── src/
│   ├── chain/      # Chain linking implementation
│   ├── codec/      # Codec registry and traits
│   ├── traits/     # Core traits (TypedContent)
│   ├── types/      # Type definitions
│   └── error.rs    # Error types
```

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Submit a pull request

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Documentation

- [User Stories](docs/USER_STORIES.md) - Detailed user stories and use cases
- [Architecture](docs/ARCHITECTURE.md) - System architecture and design decisions
- [Developer Guide](docs/DEVELOPER_GUIDE.md) - Comprehensive guide for developers
- [API Reference](docs/API.md) - Complete API documentation
- [Migration Guide](docs/MIGRATION_GUIDE.md) - Guide for migrating from other storage systems

## Links

- [CIM Architecture](https://github.com/thecowboyai/alchemist)
- [IPLD Specification](https://ipld.io/)
- [CID Specification](https://github.com/multiformats/cid)
