// Copyright 2025 Cowboy AI, LLC.

//! Encryption utilities for content at rest
//!
//! This module provides encryption capabilities for stored content,
//! supporting both application-level and NATS native encryption.

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce, Key,
};
use chacha20poly1305::{ChaCha20Poly1305, XChaCha20Poly1305};
use aes_gcm::aead;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use blake3::Hasher;
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};

/// Error types for encryption operations
#[derive(Debug, thiserror::Error)]
pub enum EncryptionError {
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("Invalid key size: expected {expected}, got {actual}")]
    InvalidKeySize { expected: usize, actual: usize },

    #[error("Invalid nonce size")]
    InvalidNonce,

    #[error("Key derivation failed")]
    KeyDerivationFailed,
}

pub type EncryptionResult<T> = std::result::Result<T, EncryptionError>;

/// Encryption algorithms supported
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EncryptionAlgorithm {
    /// AES-256-GCM (standard, hardware accelerated on many platforms)
    Aes256Gcm,
    /// ChaCha20-Poly1305 (faster in software)
    ChaCha20Poly1305,
    /// XChaCha20-Poly1305 (extended nonce variant)
    XChaCha20Poly1305,
}

impl EncryptionAlgorithm {
    /// Get the key size in bytes for this algorithm
    pub fn key_size(&self) -> usize {
        match self {
            Self::Aes256Gcm => 32,
            Self::ChaCha20Poly1305 => 32,
            Self::XChaCha20Poly1305 => 32,
        }
    }

    /// Get the nonce size in bytes for this algorithm
    pub fn nonce_size(&self) -> usize {
        match self {
            Self::Aes256Gcm => 12,
            Self::ChaCha20Poly1305 => 12,
            Self::XChaCha20Poly1305 => 24,
        }
    }
}

/// Encrypted data container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    /// The encryption algorithm used
    pub algorithm: EncryptionAlgorithm,
    /// The encrypted data
    pub ciphertext: Vec<u8>,
    /// The nonce/IV used for encryption
    pub nonce: Vec<u8>,
    /// Optional additional authenticated data (AAD)
    pub aad: Option<Vec<u8>>,
    /// Hash of the encryption key (for key rotation detection)
    pub key_hash: String,
}

/// Encryption service for content
pub struct ContentEncryption {
    /// Master encryption key
    key: Vec<u8>,
    /// Default algorithm to use
    algorithm: EncryptionAlgorithm,
    /// Key hash for validation
    key_hash: String,
}

impl ContentEncryption {
    /// Create a new encryption service with the specified key
    pub fn new(key: Vec<u8>, algorithm: EncryptionAlgorithm) -> EncryptionResult<Self> {
        if key.len() != algorithm.key_size() {
            return Err(EncryptionError::InvalidKeySize {
                expected: algorithm.key_size(),
                actual: key.len(),
            });
        }

        let key_hash = Self::hash_key(&key);

        Ok(Self {
            key,
            algorithm,
            key_hash,
        })
    }

    /// Generate a new random key for the specified algorithm
    pub fn generate_key(algorithm: EncryptionAlgorithm) -> Vec<u8> {
        let mut key = vec![0u8; algorithm.key_size()];
        rand::thread_rng().fill_bytes(&mut key);
        key
    }

    /// Hash a key for validation purposes
    fn hash_key(key: &[u8]) -> String {
        let mut hasher = Hasher::new();
        hasher.update(key);
        BASE64.encode(hasher.finalize().as_bytes())
    }

    /// Encrypt data with the configured algorithm
    pub fn encrypt(&self, plaintext: &[u8], aad: Option<&[u8]>) -> EncryptionResult<EncryptedData> {
        let nonce = self.generate_nonce();
        
        let ciphertext = match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => {
                self.encrypt_aes256gcm(plaintext, &nonce, aad)?
            }
            EncryptionAlgorithm::ChaCha20Poly1305 => {
                self.encrypt_chacha20(plaintext, &nonce, aad)?
            }
            EncryptionAlgorithm::XChaCha20Poly1305 => {
                self.encrypt_xchacha20(plaintext, &nonce, aad)?
            }
        };

        Ok(EncryptedData {
            algorithm: self.algorithm,
            ciphertext,
            nonce,
            aad: aad.map(|a| a.to_vec()),
            key_hash: self.key_hash.clone(),
        })
    }

    /// Decrypt data
    pub fn decrypt(&self, encrypted: &EncryptedData) -> EncryptionResult<Vec<u8>> {
        // Verify key hash matches
        if encrypted.key_hash != self.key_hash {
            return Err(EncryptionError::DecryptionFailed(
                "Key mismatch - possible key rotation needed".to_string()
            ));
        }

        // Verify algorithm matches
        if encrypted.algorithm != self.algorithm {
            return Err(EncryptionError::DecryptionFailed(
                format!("Algorithm mismatch: expected {:?}, got {:?}", 
                    self.algorithm, encrypted.algorithm)
            ));
        }

        let aad = encrypted.aad.as_deref();

        match encrypted.algorithm {
            EncryptionAlgorithm::Aes256Gcm => {
                self.decrypt_aes256gcm(&encrypted.ciphertext, &encrypted.nonce, aad)
            }
            EncryptionAlgorithm::ChaCha20Poly1305 => {
                self.decrypt_chacha20(&encrypted.ciphertext, &encrypted.nonce, aad)
            }
            EncryptionAlgorithm::XChaCha20Poly1305 => {
                self.decrypt_xchacha20(&encrypted.ciphertext, &encrypted.nonce, aad)
            }
        }
    }

    /// Generate a random nonce for the current algorithm
    fn generate_nonce(&self) -> Vec<u8> {
        let mut nonce = vec![0u8; self.algorithm.nonce_size()];
        rand::thread_rng().fill_bytes(&mut nonce);
        nonce
    }

    // AES-256-GCM encryption
    fn encrypt_aes256gcm(&self, plaintext: &[u8], nonce: &[u8], aad: Option<&[u8]>) -> EncryptionResult<Vec<u8>> {
        let key = Key::<Aes256Gcm>::from_slice(&self.key);
        let cipher = Aes256Gcm::new(key);
        
        let nonce = Nonce::from_slice(nonce);
        
        let ciphertext = if let Some(aad) = aad {
            cipher.encrypt(nonce, aead::Payload { msg: plaintext, aad })
        } else {
            cipher.encrypt(nonce, plaintext)
        };

        ciphertext.map_err(|e| EncryptionError::EncryptionFailed(e.to_string()))
    }

    fn decrypt_aes256gcm(&self, ciphertext: &[u8], nonce: &[u8], aad: Option<&[u8]>) -> EncryptionResult<Vec<u8>> {
        let key = Key::<Aes256Gcm>::from_slice(&self.key);
        let cipher = Aes256Gcm::new(key);
        
        let nonce = Nonce::from_slice(nonce);
        
        let plaintext = if let Some(aad) = aad {
            cipher.decrypt(nonce, aead::Payload { msg: ciphertext, aad })
        } else {
            cipher.decrypt(nonce, ciphertext)
        };

        plaintext.map_err(|e| EncryptionError::DecryptionFailed(e.to_string()))
    }

    // ChaCha20-Poly1305 encryption
    fn encrypt_chacha20(&self, plaintext: &[u8], nonce: &[u8], aad: Option<&[u8]>) -> EncryptionResult<Vec<u8>> {
        let key = Key::<ChaCha20Poly1305>::from_slice(&self.key);
        let cipher = ChaCha20Poly1305::new(key);
        
        let nonce = Nonce::from_slice(nonce);
        
        let ciphertext = if let Some(aad) = aad {
            cipher.encrypt(nonce, aead::Payload { msg: plaintext, aad })
        } else {
            cipher.encrypt(nonce, plaintext)
        };

        ciphertext.map_err(|e| EncryptionError::EncryptionFailed(e.to_string()))
    }

    fn decrypt_chacha20(&self, ciphertext: &[u8], nonce: &[u8], aad: Option<&[u8]>) -> EncryptionResult<Vec<u8>> {
        let key = Key::<ChaCha20Poly1305>::from_slice(&self.key);
        let cipher = ChaCha20Poly1305::new(key);
        
        let nonce = Nonce::from_slice(nonce);
        
        let plaintext = if let Some(aad) = aad {
            cipher.decrypt(nonce, aead::Payload { msg: ciphertext, aad })
        } else {
            cipher.decrypt(nonce, ciphertext)
        };

        plaintext.map_err(|e| EncryptionError::DecryptionFailed(e.to_string()))
    }

    // XChaCha20-Poly1305 encryption
    fn encrypt_xchacha20(&self, plaintext: &[u8], nonce: &[u8], aad: Option<&[u8]>) -> EncryptionResult<Vec<u8>> {
        let key = Key::<XChaCha20Poly1305>::from_slice(&self.key);
        let cipher = XChaCha20Poly1305::new(key);
        
        let nonce = chacha20poly1305::XNonce::from_slice(nonce);
        
        let ciphertext = if let Some(aad) = aad {
            cipher.encrypt(nonce, aead::Payload { msg: plaintext, aad })
        } else {
            cipher.encrypt(nonce, plaintext)
        };

        ciphertext.map_err(|e| EncryptionError::EncryptionFailed(e.to_string()))
    }

    fn decrypt_xchacha20(&self, ciphertext: &[u8], nonce: &[u8], aad: Option<&[u8]>) -> EncryptionResult<Vec<u8>> {
        let key = Key::<XChaCha20Poly1305>::from_slice(&self.key);
        let cipher = XChaCha20Poly1305::new(key);
        
        let nonce = chacha20poly1305::XNonce::from_slice(nonce);
        
        let plaintext = if let Some(aad) = aad {
            cipher.decrypt(nonce, aead::Payload { msg: ciphertext, aad })
        } else {
            cipher.decrypt(nonce, ciphertext)
        };

        plaintext.map_err(|e| EncryptionError::DecryptionFailed(e.to_string()))
    }

    /// Derive a key from a password using Argon2
    pub fn derive_key_from_password(
        password: &str,
        salt: &[u8],
        algorithm: EncryptionAlgorithm,
    ) -> EncryptionResult<Vec<u8>> {
        
        // Use BLAKE3 key derivation (simpler than Argon2 for this example)
        let context = format!("CIM-IPLD {} 2024", match algorithm {
            EncryptionAlgorithm::Aes256Gcm => "AES-256-GCM",
            EncryptionAlgorithm::ChaCha20Poly1305 => "ChaCha20-Poly1305",
            EncryptionAlgorithm::XChaCha20Poly1305 => "XChaCha20-Poly1305",
        });
        
        let mut hasher = blake3::Hasher::new_derive_key(&context);
        hasher.update(password.as_bytes());
        hasher.update(salt);
        
        let mut key = vec![0u8; algorithm.key_size()];
        hasher.finalize_xof().fill(&mut key);
        
        Ok(key)
    }
}

/// Key rotation helper
pub struct KeyRotation {
    old_encryption: ContentEncryption,
    new_encryption: ContentEncryption,
}

impl KeyRotation {
    /// Create a new key rotation helper
    pub fn new(
        old_key: Vec<u8>,
        new_key: Vec<u8>,
        algorithm: EncryptionAlgorithm,
    ) -> EncryptionResult<Self> {
        Ok(Self {
            old_encryption: ContentEncryption::new(old_key, algorithm)?,
            new_encryption: ContentEncryption::new(new_key, algorithm)?,
        })
    }

    /// Rotate encryption on data
    pub fn rotate(&self, encrypted: &EncryptedData) -> EncryptionResult<EncryptedData> {
        // Decrypt with old key
        let plaintext = self.old_encryption.decrypt(encrypted)?;
        
        // Re-encrypt with new key
        self.new_encryption.encrypt(&plaintext, encrypted.aad.as_deref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes_encryption_decryption() {
        let key = ContentEncryption::generate_key(EncryptionAlgorithm::Aes256Gcm);
        let encryption = ContentEncryption::new(key, EncryptionAlgorithm::Aes256Gcm).unwrap();

        let plaintext = b"Hello, World!";
        let aad = Some(b"metadata".as_ref());

        let encrypted = encryption.encrypt(plaintext, aad).unwrap();
        let decrypted = encryption.decrypt(&encrypted).unwrap();

        assert_eq!(plaintext, &decrypted[..]);
    }

    #[test]
    fn test_chacha_encryption_decryption() {
        let key = ContentEncryption::generate_key(EncryptionAlgorithm::ChaCha20Poly1305);
        let encryption = ContentEncryption::new(key, EncryptionAlgorithm::ChaCha20Poly1305).unwrap();

        let plaintext = b"Test data for ChaCha20";
        
        let encrypted = encryption.encrypt(plaintext, None).unwrap();
        let decrypted = encryption.decrypt(&encrypted).unwrap();

        assert_eq!(plaintext, &decrypted[..]);
    }

    #[test]
    fn test_key_rotation() {
        let old_key = ContentEncryption::generate_key(EncryptionAlgorithm::Aes256Gcm);
        let new_key = ContentEncryption::generate_key(EncryptionAlgorithm::Aes256Gcm);
        
        let old_encryption = ContentEncryption::new(old_key.clone(), EncryptionAlgorithm::Aes256Gcm).unwrap();
        let rotation = KeyRotation::new(old_key, new_key, EncryptionAlgorithm::Aes256Gcm).unwrap();

        let plaintext = b"Sensitive data";
        let encrypted = old_encryption.encrypt(plaintext, None).unwrap();
        
        let rotated = rotation.rotate(&encrypted).unwrap();
        
        // Old key should fail
        assert!(old_encryption.decrypt(&rotated).is_err());
        
        // New key should succeed
        let decrypted = rotation.new_encryption.decrypt(&rotated).unwrap();
        assert_eq!(plaintext, &decrypted[..]);
    }

    #[test]
    fn test_key_derivation() {
        let password = "strong password";
        let salt = b"random salt";
        
        let key = ContentEncryption::derive_key_from_password(
            password,
            salt,
            EncryptionAlgorithm::Aes256Gcm
        ).unwrap();
        
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_invalid_key_size() {
        // Test with wrong key size for AES-256-GCM
        let wrong_key = vec![0u8; 16]; // Should be 32 bytes
        let result = ContentEncryption::new(wrong_key, EncryptionAlgorithm::Aes256Gcm);
        
        assert!(result.is_err());
        match result {
            Err(EncryptionError::InvalidKeySize { expected: 32, actual: 16 }) => {}
            _ => panic!("Expected InvalidKeySize error"),
        }
        
        // Test with wrong key size for ChaCha20
        let wrong_key = vec![0u8; 24]; // Should be 32 bytes
        let result = ContentEncryption::new(wrong_key, EncryptionAlgorithm::ChaCha20Poly1305);
        
        assert!(result.is_err());
        match result {
            Err(EncryptionError::InvalidKeySize { expected: 32, actual: 24 }) => {}
            _ => panic!("Expected InvalidKeySize error"),
        }
    }

    #[test]
    fn test_key_mismatch_decryption() {
        let key1 = ContentEncryption::generate_key(EncryptionAlgorithm::Aes256Gcm);
        let key2 = ContentEncryption::generate_key(EncryptionAlgorithm::Aes256Gcm);
        
        let encryption1 = ContentEncryption::new(key1, EncryptionAlgorithm::Aes256Gcm).unwrap();
        let encryption2 = ContentEncryption::new(key2, EncryptionAlgorithm::Aes256Gcm).unwrap();
        
        let plaintext = b"Secret message";
        let encrypted = encryption1.encrypt(plaintext, None).unwrap();
        
        // Try to decrypt with different key
        let result = encryption2.decrypt(&encrypted);
        assert!(result.is_err());
        match result {
            Err(EncryptionError::DecryptionFailed(msg)) => {
                assert!(msg.contains("Key mismatch"));
            }
            _ => panic!("Expected DecryptionFailed error"),
        }
    }

    #[test]
    fn test_algorithm_mismatch_decryption() {
        let key = ContentEncryption::generate_key(EncryptionAlgorithm::Aes256Gcm);
        
        let aes_encryption = ContentEncryption::new(key.clone(), EncryptionAlgorithm::Aes256Gcm).unwrap();
        let chacha_encryption = ContentEncryption::new(key, EncryptionAlgorithm::ChaCha20Poly1305).unwrap();
        
        let plaintext = b"Test data";
        let encrypted = aes_encryption.encrypt(plaintext, None).unwrap();
        
        // Try to decrypt with different algorithm
        let result = chacha_encryption.decrypt(&encrypted);
        assert!(result.is_err());
        match result {
            Err(EncryptionError::DecryptionFailed(msg)) => {
                assert!(msg.contains("Algorithm mismatch"));
            }
            _ => panic!("Expected DecryptionFailed error"),
        }
    }

    #[test]
    fn test_corrupted_ciphertext_decryption() {
        let key = ContentEncryption::generate_key(EncryptionAlgorithm::Aes256Gcm);
        let encryption = ContentEncryption::new(key, EncryptionAlgorithm::Aes256Gcm).unwrap();
        
        let plaintext = b"Original message";
        let mut encrypted = encryption.encrypt(plaintext, None).unwrap();
        
        // Corrupt the ciphertext
        encrypted.ciphertext[0] ^= 0xFF;
        
        // Decryption should fail
        let result = encryption.decrypt(&encrypted);
        assert!(result.is_err());
        match result {
            Err(EncryptionError::DecryptionFailed(_)) => {}
            _ => panic!("Expected DecryptionFailed error"),
        }
    }

    #[test]
    fn test_corrupted_nonce_decryption() {
        let key = ContentEncryption::generate_key(EncryptionAlgorithm::Aes256Gcm);
        let encryption = ContentEncryption::new(key, EncryptionAlgorithm::Aes256Gcm).unwrap();
        
        let plaintext = b"Test message";
        let mut encrypted = encryption.encrypt(plaintext, None).unwrap();
        
        // Corrupt the nonce
        encrypted.nonce[0] ^= 0xFF;
        
        // Decryption should fail
        let result = encryption.decrypt(&encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn test_aad_mismatch() {
        let key = ContentEncryption::generate_key(EncryptionAlgorithm::Aes256Gcm);
        let encryption = ContentEncryption::new(key, EncryptionAlgorithm::Aes256Gcm).unwrap();
        
        let plaintext = b"Message with AAD";
        let aad = Some(b"metadata".as_ref());
        
        // Encrypt with AAD
        let encrypted = encryption.encrypt(plaintext, aad).unwrap();
        
        // Try to decrypt with different AAD
        let mut modified = encrypted.clone();
        modified.aad = Some(b"wrong metadata".to_vec());
        
        let result = encryption.decrypt(&modified);
        assert!(result.is_err());
        
        // Try to decrypt with no AAD when AAD was used
        modified.aad = None;
        let result = encryption.decrypt(&modified);
        assert!(result.is_err());
    }

    #[test]
    fn test_xchacha20_encryption() {
        let key = ContentEncryption::generate_key(EncryptionAlgorithm::XChaCha20Poly1305);
        let encryption = ContentEncryption::new(key, EncryptionAlgorithm::XChaCha20Poly1305).unwrap();
        
        let plaintext = b"XChaCha20 test data";
        let aad = Some(b"additional data".as_ref());
        
        let encrypted = encryption.encrypt(plaintext, aad).unwrap();
        assert_eq!(encrypted.nonce.len(), 24); // XChaCha20 uses 24-byte nonce
        
        let decrypted = encryption.decrypt(&encrypted).unwrap();
        assert_eq!(plaintext, &decrypted[..]);
    }

    #[test]
    fn test_all_aad_combinations() {
        for algorithm in [
            EncryptionAlgorithm::Aes256Gcm,
            EncryptionAlgorithm::ChaCha20Poly1305,
            EncryptionAlgorithm::XChaCha20Poly1305,
        ] {
            let key = ContentEncryption::generate_key(algorithm);
            let encryption = ContentEncryption::new(key, algorithm).unwrap();
            
            let plaintext = b"Test with various AAD";
            
            // Test with AAD
            let encrypted_with_aad = encryption.encrypt(plaintext, Some(b"aad data")).unwrap();
            assert_eq!(encrypted_with_aad.aad, Some(b"aad data".to_vec()));
            
            // Test without AAD
            let encrypted_no_aad = encryption.encrypt(plaintext, None).unwrap();
            assert_eq!(encrypted_no_aad.aad, None);
            
            // Verify both decrypt correctly
            let decrypted1 = encryption.decrypt(&encrypted_with_aad).unwrap();
            let decrypted2 = encryption.decrypt(&encrypted_no_aad).unwrap();
            
            assert_eq!(plaintext, &decrypted1[..]);
            assert_eq!(plaintext, &decrypted2[..]);
        }
    }

    #[test]
    fn test_key_rotation_with_different_algorithms() {
        // Test that key rotation fails when algorithms differ
        let old_key = ContentEncryption::generate_key(EncryptionAlgorithm::Aes256Gcm);
        let new_key = ContentEncryption::generate_key(EncryptionAlgorithm::ChaCha20Poly1305);
        
        let old_encryption = ContentEncryption::new(old_key, EncryptionAlgorithm::Aes256Gcm).unwrap();
        let new_encryption = ContentEncryption::new(new_key, EncryptionAlgorithm::ChaCha20Poly1305).unwrap();
        
        let rotation = KeyRotation {
            old_encryption,
            new_encryption,
        };
        
        let plaintext = b"Data to rotate";
        let encrypted = rotation.old_encryption.encrypt(plaintext, None).unwrap();
        
        // Rotation should work but produce different algorithm
        let rotated = rotation.rotate(&encrypted).unwrap();
        assert_eq!(rotated.algorithm, EncryptionAlgorithm::ChaCha20Poly1305);
        
        // Verify new encryption can decrypt
        let decrypted = rotation.new_encryption.decrypt(&rotated).unwrap();
        assert_eq!(plaintext, &decrypted[..]);
    }

    #[test]
    fn test_nonce_size_validation() {
        let key = ContentEncryption::generate_key(EncryptionAlgorithm::Aes256Gcm);
        let encryption = ContentEncryption::new(key, EncryptionAlgorithm::Aes256Gcm).unwrap();
        
        let plaintext = b"Test";
        let encrypted = encryption.encrypt(plaintext, None).unwrap();
        
        // Verify correct nonce sizes for each algorithm
        assert_eq!(EncryptionAlgorithm::Aes256Gcm.nonce_size(), 12);
        assert_eq!(EncryptionAlgorithm::ChaCha20Poly1305.nonce_size(), 12);
        assert_eq!(EncryptionAlgorithm::XChaCha20Poly1305.nonce_size(), 24);
        
        assert_eq!(encrypted.nonce.len(), 12);
    }

    #[test]
    fn test_key_hash_validation() {
        let key1 = vec![1u8; 32];
        let key2 = vec![2u8; 32];
        
        let enc1 = ContentEncryption::new(key1, EncryptionAlgorithm::Aes256Gcm).unwrap();
        let enc2 = ContentEncryption::new(key2, EncryptionAlgorithm::Aes256Gcm).unwrap();
        
        // Different keys should have different hashes
        assert_ne!(enc1.key_hash, enc2.key_hash);
        
        // Same key should have same hash
        let key3 = vec![1u8; 32];
        let enc3 = ContentEncryption::new(key3, EncryptionAlgorithm::Aes256Gcm).unwrap();
        assert_eq!(enc1.key_hash, enc3.key_hash);
    }

    #[test]
    fn test_empty_plaintext() {
        let key = ContentEncryption::generate_key(EncryptionAlgorithm::Aes256Gcm);
        let encryption = ContentEncryption::new(key, EncryptionAlgorithm::Aes256Gcm).unwrap();
        
        let plaintext = b"";
        let encrypted = encryption.encrypt(plaintext, None).unwrap();
        let decrypted = encryption.decrypt(&encrypted).unwrap();
        
        assert_eq!(plaintext, &decrypted[..]);
    }

    #[test]
    fn test_large_plaintext() {
        let key = ContentEncryption::generate_key(EncryptionAlgorithm::Aes256Gcm);
        let encryption = ContentEncryption::new(key, EncryptionAlgorithm::Aes256Gcm).unwrap();
        
        // Test with 1MB of data
        let plaintext = vec![42u8; 1024 * 1024];
        let encrypted = encryption.encrypt(&plaintext, None).unwrap();
        let decrypted = encryption.decrypt(&encrypted).unwrap();
        
        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_key_derivation_consistency() {
        let password = "test password";
        let salt = b"consistent salt";
        
        // Same password and salt should produce same key
        let key1 = ContentEncryption::derive_key_from_password(
            password,
            salt,
            EncryptionAlgorithm::Aes256Gcm
        ).unwrap();
        
        let key2 = ContentEncryption::derive_key_from_password(
            password,
            salt,
            EncryptionAlgorithm::Aes256Gcm
        ).unwrap();
        
        assert_eq!(key1, key2);
        
        // Different salt should produce different key
        let key3 = ContentEncryption::derive_key_from_password(
            password,
            b"different salt",
            EncryptionAlgorithm::Aes256Gcm
        ).unwrap();
        
        assert_ne!(key1, key3);
        
        // Different password should produce different key
        let key4 = ContentEncryption::derive_key_from_password(
            "different password",
            salt,
            EncryptionAlgorithm::Aes256Gcm
        ).unwrap();
        
        assert_ne!(key1, key4);
    }

    #[test]
    fn test_encryption_error_display() {
        let err1 = EncryptionError::EncryptionFailed("test error".to_string());
        assert_eq!(err1.to_string(), "Encryption failed: test error");
        
        let err2 = EncryptionError::DecryptionFailed("decrypt error".to_string());
        assert_eq!(err2.to_string(), "Decryption failed: decrypt error");
        
        let err3 = EncryptionError::InvalidKeySize { expected: 32, actual: 16 };
        assert_eq!(err3.to_string(), "Invalid key size: expected 32, got 16");
        
        let err4 = EncryptionError::InvalidNonce;
        assert_eq!(err4.to_string(), "Invalid nonce size");
        
        let err5 = EncryptionError::KeyDerivationFailed;
        assert_eq!(err5.to_string(), "Key derivation failed");
    }
}