//! # CIM-IPLD
//!
//! IPLD implementation for Composable Information Machines (CIM).
//!
//! This library provides:
//! - Content-addressed storage with CIDs
//! - Type-safe content handling with custom codecs
//! - Cryptographic event chains
//! - Extensible codec registry for domain-specific types
//!
//! ## Example
//!
//! ```rust
//! use cim_ipld::{ChainedContent, TypedContent, ContentType};
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Serialize, Deserialize)]
//! struct MyContent {
//!     data: String,
//! }
//!
//! impl TypedContent for MyContent {
//!     const CODEC: u64 = 0x300000;
//!     const CONTENT_TYPE: ContentType = ContentType::Custom(0x300000);
//!
//!     // ... implement required methods
//! }
//! ```

pub mod chain;
pub mod codec;
pub mod error;
pub mod traits;
pub mod types;

pub use chain::{ChainedContent, ContentChain};
pub use codec::{CodecRegistry, CimCodec};
pub use error::{Error, Result};
pub use traits::TypedContent;
pub use types::{ContentType, Cid};
