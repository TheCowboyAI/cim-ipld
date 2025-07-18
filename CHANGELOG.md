# Changelog

All notable changes to this project will be documented in this file.

## [0.5.0] - 2025-06-17

### Added
- **Comprehensive Test Coverage**: Achieved 94% test coverage with 206 passing tests
- **Documentation Consolidation**: Reorganized all documentation into a clear structure
- **Copyright Notices**: Added copyright notices to all source files and documentation
- **Encryption Support**: Full encryption at rest with AES-256-GCM, ChaCha20-Poly1305, and XChaCha20-Poly1305
- **Content Indexing**: Full-text search with metadata indexing and persistence
- **Domain Partitioning**: Automatic content routing based on type and patterns

### Changed
- **Version Bump**: Updated to version 0.5.0
- **License**: Changed to MIT License only (removed dual licensing)
- **Documentation Structure**: Consolidated 27 scattered files into organized structure

### Previous Releases

## [0.3.0] - Previous Release

### Added
- **Canonical Payload Support**: Added `canonical_payload()` method to `TypedContent` trait
  - Allows content types to define which fields are included in CID calculation
  - Excludes transient metadata (timestamps, UUIDs, correlation IDs) from CIDs
  - Ensures identical content always produces the same CID
  - Default implementation uses full serialization for backward compatibility
  - See `examples/event_cid_example.rs` for usage patterns

### Changed
- `TypedContent::calculate_cid()` now uses `canonical_payload()` instead of `to_bytes()`
  - This is a breaking change if you rely on CIDs including all fields
  - To maintain old behavior, don't override `canonical_payload()`

## [0.1.0] - Initial Release

- Initial implementation of CIM-IPLD
- Content-addressed storage with NATS Object Store
- CID chain support for event sourcing
- TypedContent trait for type-safe content storage


---
Copyright 2025 Cowboy AI, LLC.
