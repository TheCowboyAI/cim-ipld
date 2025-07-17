//! Error types for CIM-IPLD

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to serialize content: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("CBOR serialization error: {0}")]
    CborError(String),

    #[error("Invalid CID: {0}")]
    InvalidCid(String),

    #[error("Chain validation failed: expected {expected}, got {actual}")]
    ChainValidationError { expected: String, actual: String },

    #[error("Sequence validation failed: expected {expected}, got {actual}")]
    SequenceValidationError { expected: u64, actual: u64 },

    #[error("Invalid codec range: {0}. Must be in range 0x300000-0x3FFFFF")]
    InvalidCodecRange(u64),

    #[error("Codec not found: {0}")]
    CodecNotFound(u64),

    #[error("Content type mismatch: expected {expected:?}, got {actual:?}")]
    ContentTypeMismatch { expected: String, actual: String },

    #[error("Multihash error: {0}")]
    MultihashError(String),

    #[error("Invalid content: {0}")]
    InvalidContent(String),
    
    #[error("Storage error: {0}")]
    StorageError(String),
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error as StdError;

    #[test]
    fn test_serialization_error() {
        // Test from serde_json::Error
        let json_err = serde_json::from_str::<String>("invalid json").unwrap_err();
        let err = Error::from(json_err);
        
        match err {
            Error::SerializationError(_) => {
                assert!(err.to_string().contains("Failed to serialize content"));
            }
            _ => panic!("Expected SerializationError"),
        }
    }

    #[test]
    fn test_cbor_error() {
        let err = Error::CborError("CBOR encoding failed".to_string());
        assert_eq!(err.to_string(), "CBOR serialization error: CBOR encoding failed");
        
        // Test Debug trait
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("CborError"));
        assert!(debug_str.contains("CBOR encoding failed"));
    }

    #[test]
    fn test_invalid_cid() {
        let err = Error::InvalidCid("not a valid CID".to_string());
        assert_eq!(err.to_string(), "Invalid CID: not a valid CID");
    }

    #[test]
    fn test_chain_validation_error() {
        let err = Error::ChainValidationError {
            expected: "abc123".to_string(),
            actual: "def456".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Chain validation failed: expected abc123, got def456"
        );
    }

    #[test]
    fn test_sequence_validation_error() {
        let err = Error::SequenceValidationError {
            expected: 5,
            actual: 3,
        };
        assert_eq!(
            err.to_string(),
            "Sequence validation failed: expected 5, got 3"
        );
    }

    #[test]
    fn test_invalid_codec_range() {
        let err = Error::InvalidCodecRange(0x100000);
        assert_eq!(
            err.to_string(),
            "Invalid codec range: 1048576. Must be in range 0x300000-0x3FFFFF"
        );
    }

    #[test]
    fn test_codec_not_found() {
        let err = Error::CodecNotFound(0x300100);
        assert_eq!(err.to_string(), "Codec not found: 3145984");
    }

    #[test]
    fn test_content_type_mismatch() {
        let err = Error::ContentTypeMismatch {
            expected: "Event".to_string(),
            actual: "Graph".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Content type mismatch: expected \"Event\", got \"Graph\""
        );
    }

    #[test]
    fn test_multihash_error() {
        let err = Error::MultihashError("Invalid hash algorithm".to_string());
        assert_eq!(err.to_string(), "Multihash error: Invalid hash algorithm");
    }

    #[test]
    fn test_invalid_content() {
        let err = Error::InvalidContent("Missing required field".to_string());
        assert_eq!(err.to_string(), "Invalid content: Missing required field");
    }

    #[test]
    fn test_storage_error() {
        let err = Error::StorageError("Connection timeout".to_string());
        assert_eq!(err.to_string(), "Storage error: Connection timeout");
    }

    #[test]
    fn test_error_result_type() {
        // Test that Result type alias works correctly
        let success: Result<i32> = Ok(42);
        assert_eq!(success.unwrap(), 42);

        let failure: Result<i32> = Err(Error::InvalidCid("test".to_string()));
        assert!(failure.is_err());
    }

    #[test]
    fn test_error_send_sync() {
        // Ensure Error implements Send + Sync for thread safety
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Error>();
    }

    #[test]
    fn test_error_source_chain() {
        // Test that errors can be chained properly
        let json_err = serde_json::from_str::<String>("invalid").unwrap_err();
        let err = Error::from(json_err);
        
        // The source should be the original serde_json error
        assert!(err.source().is_some());
    }
}
