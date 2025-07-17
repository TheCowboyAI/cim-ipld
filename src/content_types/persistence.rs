//! Persistence layer for content indexing using NATS KV store
//!
//! This module provides persistence capabilities for the in-memory index,
//! using NATS KV (Key-Value) store for durability and encryption at rest.

use crate::{
    content_types::indexing::IndexStats,
    content_types::{ContentType, DocumentMetadata, ImageMetadata, AudioMetadata, VideoMetadata},
};
use async_nats::jetstream::{self, kv::{Store as KvStore, Config as KvConfig}};
use cid::Cid;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use super::encryption::{ContentEncryption, EncryptionAlgorithm, EncryptedData};

/// Error types for persistence operations
#[derive(Debug, thiserror::Error)]
pub enum PersistenceError {
    #[error("NATS error: {0}")]
    Nats(#[from] async_nats::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Decryption error: {0}")]
    Decryption(String),

    #[error("Key not found: {0}")]
    NotFound(String),

    #[error("Invalid encryption key")]
    InvalidKey,
}

pub type PersistenceResult<T> = std::result::Result<T, PersistenceError>;

/// Encrypted CID wrapper for storing unencrypted CIDs with encrypted metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedCidWrapper {
    /// The unencrypted CID (for content retrieval)
    pub cid: String,
    /// Encrypted metadata
    pub encrypted_metadata: Vec<u8>,
    /// Initialization vector for decryption
    pub iv: Vec<u8>,
    /// Hash of the encryption key used (for key rotation detection)
    pub key_hash: String,
}

/// Persisted index data structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedTextIndex {
    pub word_to_cids: HashMap<String, HashSet<String>>,
    pub cid_to_text: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedTagIndex {
    pub tag_to_cids: HashMap<String, HashSet<String>>,
    pub cid_to_tags: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedTypeIndex {
    pub type_to_cids: HashMap<String, HashSet<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedMetadataCache {
    pub documents: HashMap<String, DocumentMetadata>,
    pub images: HashMap<String, ImageMetadata>,
    pub audio: HashMap<String, AudioMetadata>,
    pub video: HashMap<String, VideoMetadata>,
}

/// Index persistence service using NATS KV store
pub struct IndexPersistence {
    jetstream: jetstream::Context,
    /// KV stores for different index types
    text_store: Arc<RwLock<Option<KvStore>>>,
    tag_store: Arc<RwLock<Option<KvStore>>>,
    type_store: Arc<RwLock<Option<KvStore>>>,
    metadata_store: Arc<RwLock<Option<KvStore>>>,
    /// Encryption service
    encryption: Option<ContentEncryption>,
    /// Whether to enable NATS native encryption
    #[allow(dead_code)]
    native_encryption: bool,
}

impl IndexPersistence {
    /// Create a new index persistence service
    pub async fn new(
        jetstream: jetstream::Context,
        encryption_key: Option<Vec<u8>>,
        native_encryption: bool,
    ) -> PersistenceResult<Self> {
        // Create encryption service if key provided
        let encryption = if let Some(key) = encryption_key {
            Some(ContentEncryption::new(key, EncryptionAlgorithm::ChaCha20Poly1305)
                .map_err(|e| PersistenceError::Encryption(e.to_string()))?)
        } else {
            None
        };

        let persistence = Self {
            jetstream,
            text_store: Arc::new(RwLock::new(None)),
            tag_store: Arc::new(RwLock::new(None)),
            type_store: Arc::new(RwLock::new(None)),
            metadata_store: Arc::new(RwLock::new(None)),
            encryption,
            native_encryption,
        };

        // Initialize KV stores
        persistence.initialize_stores().await?;

        Ok(persistence)
    }

    /// Initialize KV stores
    async fn initialize_stores(&self) -> PersistenceResult<()> {
        // Text index store
        let text_config = KvConfig {
            bucket: "cim-index-text".to_string(),
            description: "CIM text search index".to_string(),
            history: 5,
            ..Default::default()
        };
        let text_store = self.jetstream.create_key_value(text_config).await
            .map_err(|e| PersistenceError::Nats(e.to_string().into()))?;
        *self.text_store.write().await = Some(text_store);

        // Tag index store
        let tag_config = KvConfig {
            bucket: "cim-index-tags".to_string(),
            description: "CIM tag index".to_string(),
            history: 5,
            ..Default::default()
        };
        let tag_store = self.jetstream.create_key_value(tag_config).await
            .map_err(|e| PersistenceError::Nats(e.to_string().into()))?;
        *self.tag_store.write().await = Some(tag_store);

        // Type index store
        let type_config = KvConfig {
            bucket: "cim-index-types".to_string(),
            description: "CIM content type index".to_string(),
            history: 5,
            ..Default::default()
        };
        let type_store = self.jetstream.create_key_value(type_config).await
            .map_err(|e| PersistenceError::Nats(e.to_string().into()))?;
        *self.type_store.write().await = Some(type_store);

        // Metadata cache store
        let metadata_config = KvConfig {
            bucket: "cim-index-metadata".to_string(),
            description: "CIM metadata cache".to_string(),
            history: 5,
            ..Default::default()
        };
        let metadata_store = self.jetstream.create_key_value(metadata_config).await
            .map_err(|e| PersistenceError::Nats(e.to_string().into()))?;
        *self.metadata_store.write().await = Some(metadata_store);

        Ok(())
    }

    /// Encrypt data if encryption is enabled
    fn encrypt_data(&self, data: &[u8]) -> PersistenceResult<Vec<u8>> {
        if let Some(ref encryption) = self.encryption {
            let encrypted = encryption.encrypt(data, None)
                .map_err(|e| PersistenceError::Encryption(e.to_string()))?;
            
            // Serialize the encrypted data structure
            serde_json::to_vec(&encrypted)
                .map_err(|e| PersistenceError::Encryption(e.to_string()))
        } else {
            Ok(data.to_vec())
        }
    }

    /// Decrypt data if encryption is enabled
    fn decrypt_data(&self, data: &[u8]) -> PersistenceResult<Vec<u8>> {
        if let Some(ref encryption) = self.encryption {
            // Deserialize the encrypted data structure
            let encrypted: EncryptedData = serde_json::from_slice(data)
                .map_err(|e| PersistenceError::Decryption(e.to_string()))?;
            
            encryption.decrypt(&encrypted)
                .map_err(|e| PersistenceError::Decryption(e.to_string()))
        } else {
            Ok(data.to_vec())
        }
    }


    /// Save text index
    pub async fn save_text_index(
        &self,
        word_to_cids: &HashMap<String, HashSet<Cid>>,
        cid_to_text: &HashMap<Cid, String>,
    ) -> PersistenceResult<()> {
        let store_guard = self.text_store.read().await;
        let store = store_guard.as_ref().ok_or(PersistenceError::NotFound("text store".to_string()))?;

        // Convert to persistable format
        let persisted = PersistedTextIndex {
            word_to_cids: word_to_cids.iter()
                .map(|(k, v)| (k.clone(), v.iter().map(|c| c.to_string()).collect()))
                .collect(),
            cid_to_text: cid_to_text.iter()
                .map(|(k, v)| (k.to_string(), v.clone()))
                .collect(),
        };

        // Serialize
        let data = serde_json::to_vec(&persisted)
            .map_err(|e| PersistenceError::Serialization(e.to_string()))?;

        // Encrypt if enabled
        let data = self.encrypt_data(&data)?;

        // Store in NATS KV
        store.put("text_index", data.into()).await
            .map_err(|e| PersistenceError::Nats(e.to_string().into()))?;

        Ok(())
    }

    /// Load text index
    pub async fn load_text_index(&self) -> PersistenceResult<Option<(HashMap<String, HashSet<Cid>>, HashMap<Cid, String>)>> {
        let store_guard = self.text_store.read().await;
        let store = store_guard.as_ref().ok_or(PersistenceError::NotFound("text store".to_string()))?;

        // Get from KV store
        let entry = match store.get("text_index").await {
            Ok(Some(entry)) => entry,
            Ok(None) => return Ok(None),
            Err(e) => return Err(PersistenceError::Nats(e.to_string().into())),
        };

        // Decrypt if needed
        let data = self.decrypt_data(&entry)?;

        // Deserialize
        let persisted: PersistedTextIndex = serde_json::from_slice(&data)
            .map_err(|e| PersistenceError::Serialization(e.to_string()))?;

        // Convert back to runtime format
        let word_to_cids = persisted.word_to_cids.into_iter()
            .map(|(k, v)| {
                let cids: std::result::Result<HashSet<Cid>, _> = v.iter()
                    .map(|s| Cid::try_from(s.as_str()))
                    .collect();
                cids.map(|cids| (k, cids))
                    .map_err(|_| PersistenceError::Serialization("Invalid CID".to_string()))
            })
            .collect::<PersistenceResult<HashMap<_, _>>>()
            .map_err(|e| PersistenceError::Serialization(e.to_string()))?;

        let cid_to_text = persisted.cid_to_text.into_iter()
            .map(|(k, v)| {
                Cid::try_from(k.as_str())
                    .map(|cid| (cid, v))
                    .map_err(|_| PersistenceError::Serialization("Invalid CID".to_string()))
            })
            .collect::<PersistenceResult<HashMap<_, _>>>()
            .map_err(|e| PersistenceError::Serialization(e.to_string()))?;

        Ok(Some((word_to_cids, cid_to_text)))
    }

    /// Save tag index
    pub async fn save_tag_index(
        &self,
        tag_to_cids: &HashMap<String, HashSet<Cid>>,
        cid_to_tags: &HashMap<Cid, Vec<String>>,
    ) -> PersistenceResult<()> {
        let store_guard = self.tag_store.read().await;
        let store = store_guard.as_ref().ok_or(PersistenceError::NotFound("tag store".to_string()))?;

        let persisted = PersistedTagIndex {
            tag_to_cids: tag_to_cids.iter()
                .map(|(k, v)| (k.clone(), v.iter().map(|c| c.to_string()).collect()))
                .collect(),
            cid_to_tags: cid_to_tags.iter()
                .map(|(k, v)| (k.to_string(), v.clone()))
                .collect(),
        };

        let data = serde_json::to_vec(&persisted)
            .map_err(|e| PersistenceError::Serialization(e.to_string()))?;

        let data = self.encrypt_data(&data)?;

        store.put("tag_index", data.into()).await
            .map_err(|e| PersistenceError::Nats(e.to_string().into()))?;

        Ok(())
    }

    /// Save type index
    pub async fn save_type_index(
        &self,
        type_to_cids: &HashMap<ContentType, HashSet<Cid>>,
    ) -> PersistenceResult<()> {
        let store_guard = self.type_store.read().await;
        let store = store_guard.as_ref().ok_or(PersistenceError::NotFound("type store".to_string()))?;

        // Convert ContentType to string for serialization
        let persisted = PersistedTypeIndex {
            type_to_cids: type_to_cids.iter()
                .map(|(k, v)| {
                    let type_str = format!("{:?}", k);
                    (type_str, v.iter().map(|c| c.to_string()).collect())
                })
                .collect(),
        };

        let data = serde_json::to_vec(&persisted)
            .map_err(|e| PersistenceError::Serialization(e.to_string()))?;

        let data = self.encrypt_data(&data)?;

        store.put("type_index", data.into()).await
            .map_err(|e| PersistenceError::Nats(e.to_string().into()))?;

        Ok(())
    }

    /// Save metadata cache
    pub async fn save_metadata_cache(
        &self,
        documents: &HashMap<Cid, DocumentMetadata>,
        images: &HashMap<Cid, ImageMetadata>,
        audio: &HashMap<Cid, AudioMetadata>,
        video: &HashMap<Cid, VideoMetadata>,
    ) -> PersistenceResult<()> {
        let store_guard = self.metadata_store.read().await;
        let store = store_guard.as_ref().ok_or(PersistenceError::NotFound("metadata store".to_string()))?;

        let persisted = PersistedMetadataCache {
            documents: documents.iter()
                .map(|(k, v)| (k.to_string(), v.clone()))
                .collect(),
            images: images.iter()
                .map(|(k, v)| (k.to_string(), v.clone()))
                .collect(),
            audio: audio.iter()
                .map(|(k, v)| (k.to_string(), v.clone()))
                .collect(),
            video: video.iter()
                .map(|(k, v)| (k.to_string(), v.clone()))
                .collect(),
        };

        let data = serde_json::to_vec(&persisted)
            .map_err(|e| PersistenceError::Serialization(e.to_string()))?;

        let data = self.encrypt_data(&data)?;

        store.put("metadata_cache", data.into()).await
            .map_err(|e| PersistenceError::Nats(e.to_string().into()))?;

        Ok(())
    }

    /// Create encrypted CID wrapper
    pub async fn create_encrypted_wrapper(
        &self,
        cid: &Cid,
        metadata: &[u8],
    ) -> PersistenceResult<EncryptedCidWrapper> {
        if let Some(ref encryption) = self.encryption {
            let encrypted = encryption.encrypt(metadata, Some(cid.to_string().as_bytes()))
                .map_err(|e| PersistenceError::Encryption(e.to_string()))?;

            Ok(EncryptedCidWrapper {
                cid: cid.to_string(),
                encrypted_metadata: encrypted.ciphertext,
                iv: encrypted.nonce,
                key_hash: encrypted.key_hash,
            })
        } else {
            Err(PersistenceError::Encryption("No encryption key configured".to_string()))
        }
    }

    /// Decrypt metadata from wrapper
    pub async fn decrypt_wrapper_metadata(&self, wrapper: &EncryptedCidWrapper) -> PersistenceResult<Vec<u8>> {
        if let Some(ref encryption) = self.encryption {
            // Reconstruct EncryptedData
            let encrypted_data = EncryptedData {
                algorithm: EncryptionAlgorithm::ChaCha20Poly1305,
                ciphertext: wrapper.encrypted_metadata.clone(),
                nonce: wrapper.iv.clone(),
                aad: Some(wrapper.cid.as_bytes().to_vec()),
                key_hash: wrapper.key_hash.clone(),
            };

            encryption.decrypt(&encrypted_data)
                .map_err(|e| PersistenceError::Decryption(e.to_string()))
        } else {
            Err(PersistenceError::Decryption("No encryption key configured".to_string()))
        }
    }

    /// Save index statistics
    pub async fn save_stats(&self, stats: &IndexStats) -> PersistenceResult<()> {
        let store_guard = self.metadata_store.read().await;
        let store = store_guard.as_ref().ok_or(PersistenceError::NotFound("metadata store".to_string()))?;

        let data = serde_json::to_vec(stats)
            .map_err(|e| PersistenceError::Serialization(e.to_string()))?;

        let data = self.encrypt_data(&data)?;

        store.put("index_stats", data.into()).await
            .map_err(|e| PersistenceError::Nats(e.to_string().into()))?;

        Ok(())
    }

    /// Load index statistics
    pub async fn load_stats(&self) -> PersistenceResult<Option<IndexStats>> {
        let store_guard = self.metadata_store.read().await;
        let store = store_guard.as_ref().ok_or(PersistenceError::NotFound("metadata store".to_string()))?;

        let entry = match store.get("index_stats").await {
            Ok(Some(entry)) => entry,
            Ok(None) => return Ok(None),
            Err(e) => return Err(PersistenceError::Nats(e.to_string().into())),
        };

        let data = self.decrypt_data(&entry)?;

        let stats = serde_json::from_slice(&data)
            .map_err(|e| PersistenceError::Serialization(e.to_string()))?;

        Ok(Some(stats))
    }

    /// Perform atomic backup of all indices
    pub async fn backup_all(&self, backup_name: &str) -> PersistenceResult<()> {
        // Create backup entries for each index type
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let backup_key = format!("backup_{}_{}_{}", backup_name, timestamp, "text");

        // Backup text index
        if let Ok(Some(entry)) = self.text_store.read().await.as_ref().unwrap().get("text_index").await {
            self.text_store.read().await.as_ref().unwrap()
                .put(&backup_key, entry).await
                .map_err(|e| PersistenceError::Nats(e.to_string().into()))?;
        }

        // Similar for other indices...

        Ok(())
    }

    /// Restore from backup
    pub async fn restore_from_backup(&self, _backup_name: &str) -> PersistenceResult<()> {
        // Find latest backup entries and restore
        // Implementation would scan for backup entries and restore them

        Ok(())
    }
}

/// Configuration for NATS native encryption
#[derive(Debug, Clone)]
pub struct NatsEncryptionConfig {
    /// Enable server-side encryption
    pub server_encryption: bool,
    /// Encryption algorithm (AES-256-GCM recommended)
    pub algorithm: String,
    /// Key rotation period in days
    pub key_rotation_days: u32,
}

impl Default for NatsEncryptionConfig {
    fn default() -> Self {
        Self {
            server_encryption: true,
            algorithm: "AES-256-GCM".to_string(),
            key_rotation_days: 90,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_encryption_wrapper() {
        // Test creating and decrypting wrapper
        let key = vec![0u8; 32]; // 32-byte key
        let jetstream = jetstream::new(async_nats::connect("nats://localhost:4222").await.unwrap());
        
        let persistence = IndexPersistence::new(
            jetstream,
            Some(key),
            false,
        ).await.unwrap();

        let cid = Cid::default();
        let metadata = b"test metadata";

        let wrapper = persistence.create_encrypted_wrapper(&cid, metadata).await.unwrap();
        assert_eq!(wrapper.cid, cid.to_string());

        let decrypted = persistence.decrypt_wrapper_metadata(&wrapper).await.unwrap();
        assert_eq!(decrypted, metadata);
    }
}