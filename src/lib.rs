//! CIM-IPLD: Content-addressed storage for the Composable Information Machine

pub mod chain;
pub mod codec;
pub mod error;
pub mod traits;
pub mod types;
pub mod object_store;

// Re-exports for convenience
pub use cid::Cid;
pub use multihash::Multihash;

pub use chain::{ChainedContent, ContentChain};
pub use codec::{CimCodec, CodecRegistry};
pub use error::{Error, Result};
pub use traits::TypedContent;
pub use types::ContentType;
