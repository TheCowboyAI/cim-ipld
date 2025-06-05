//! Core traits for CIM-IPLD

use crate::{ContentType, Result, Cid};
use serde::{Serialize, de::DeserializeOwned};

/// Trait for content that can be stored with a CID
pub trait TypedContent: Serialize + DeserializeOwned + Send + Sync {
    /// The IPLD codec for this content type
    const CODEC: u64;

    /// The content type identifier
    const CONTENT_TYPE: ContentType;

    /// Calculate the CID for this content
    fn calculate_cid(&self) -> Result<Cid> {
        // Serialize to bytes
        let bytes = serde_json::to_vec(self)?;

        // Create hash using BLAKE3
        let hash = blake3::hash(&bytes);
        let hash_bytes = hash.as_bytes();

        // Create multihash manually with BLAKE3 code (0x1e)
        let code = 0x1e; // BLAKE3-256
        let size = hash_bytes.len() as u8;

        // Build multihash: <varint code><varint size><hash>
        let mut multihash_bytes = Vec::new();
        multihash_bytes.push(code);
        multihash_bytes.push(size);
        multihash_bytes.extend_from_slice(hash_bytes);

        // Create CID v1
        let mh = multihash::Multihash::from_bytes(&multihash_bytes)
            .map_err(|e| crate::Error::MultihashError(e.to_string()))?;
        let cid = Cid::new_v1(Self::CODEC, mh);

        Ok(cid)
    }

    /// Convert to bytes for storage
    fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(serde_json::to_vec(self)?)
    }

    /// Create from bytes
    fn from_bytes(bytes: &[u8]) -> Result<Self> where Self: Sized {
        Ok(serde_json::from_slice(bytes)?)
    }
}
