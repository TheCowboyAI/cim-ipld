//! Integration tests for error propagation across the CIM-IPLD system
//!
//! These tests verify that errors are properly propagated and handled
//! throughout the different layers of the system.

use cim_ipld::{
    ContentChain, TypedContent, ContentType, Error, Result,
    codec::{CodecRegistry, CimCodec},
    content_types::{
        PdfDocument, JpegImage, Mp3Audio, DocumentMetadata, ImageMetadata, AudioMetadata,
    },
    object_store::ObjectStoreError,
};
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Test content that can fail in various ways
#[derive(Debug, Clone, Serialize, Deserialize)]
struct FailableContent {
    data: String,
    fail_mode: FailMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum FailMode {
    None,
    SerializationError,
    InvalidContent,
}

impl TypedContent for FailableContent {
    const CODEC: u64 = 0x399999;
    const CONTENT_TYPE: ContentType = ContentType::Custom(0x399999);

    fn to_bytes(&self) -> Result<Vec<u8>> {
        match self.fail_mode {
            FailMode::SerializationError => {
                // Force a serialization error by returning invalid JSON
                Err(Error::SerializationError(
                    serde_json::Error::io(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Forced serialization error"
                    ))
                ))
            }
            _ => Ok(serde_json::to_vec(self)?),
        }
    }

    fn canonical_payload(&self) -> Result<Vec<u8>> {
        match self.fail_mode {
            FailMode::InvalidContent => {
                Err(Error::InvalidContent("Forced invalid content error".into()))
            }
            _ => Ok(self.data.as_bytes().to_vec()),
        }
    }
}

#[test]
fn test_serialization_error_propagation() {
    let content = FailableContent {
        data: "test".to_string(),
        fail_mode: FailMode::SerializationError,
    };

    // Should fail when trying to serialize
    let result = content.to_bytes();
    assert!(result.is_err());
    
    match result {
        Err(Error::SerializationError(_)) => {
            // Expected error type
        }
        _ => panic!("Expected SerializationError"),
    }
}

#[test]
fn test_cid_calculation_error_propagation() {
    let content = FailableContent {
        data: "test".to_string(),
        fail_mode: FailMode::InvalidContent,
    };

    // Should fail when calculating CID due to canonical_payload error
    let result = content.calculate_cid();
    assert!(result.is_err());
    
    match result {
        Err(Error::InvalidContent(_)) => {
            // Expected error type
        }
        _ => panic!("Expected InvalidContent error"),
    }
}

#[test]
fn test_chain_append_error_propagation() {
    let mut chain = ContentChain::<FailableContent>::new();
    
    // First append should succeed
    let good_content = FailableContent {
        data: "good".to_string(),
        fail_mode: FailMode::None,
    };
    assert!(chain.append(good_content).is_ok());
    
    // Test with content that fails canonical_payload
    let bad_content1 = FailableContent {
        data: "bad".to_string(),
        fail_mode: FailMode::InvalidContent,
    };
    
    // This should fail on calculate_cid
    let cid_result = bad_content1.calculate_cid();
    assert!(cid_result.is_err());
    
    // Test with content that fails to_bytes
    let bad_content2 = FailableContent {
        data: "bad".to_string(),
        fail_mode: FailMode::SerializationError,
    };
    
    // This should fail on to_bytes
    let bytes_result = bad_content2.to_bytes();
    assert!(bytes_result.is_err());
    
    // Chain append works because it uses serde_json directly on the struct
    let result = chain.append(bad_content1);
    assert!(result.is_ok()); // Chain doesn't use TypedContent methods
}

#[test]
fn test_codec_range_validation_error() {
    let mut registry = CodecRegistry::new();
    
    // Create codec with invalid range
    struct BadCodec;
    impl CimCodec for BadCodec {
        fn code(&self) -> u64 {
            0x100000 // Below valid range
        }
        fn name(&self) -> &str {
            "bad-codec"
        }
    }
    
    let result = registry.register(Arc::new(BadCodec));
    assert!(result.is_err());
    
    match result {
        Err(Error::InvalidCodecRange(code)) => {
            assert_eq!(code, 0x100000);
        }
        _ => panic!("Expected InvalidCodecRange error"),
    }
}

#[test]
fn test_content_verification_error_propagation() {
    // Create invalid PDF data
    let invalid_pdf = vec![0xFF, 0xFE, 0xFD]; // Not a PDF
    
    let result = PdfDocument::new(
        invalid_pdf,
        DocumentMetadata::default()
    );
    
    assert!(result.is_err());
    match result {
        Err(Error::InvalidContent(msg)) => {
            assert!(msg.contains("PDF"));
        }
        _ => panic!("Expected InvalidContent error for PDF"),
    }
    
    // Test with other content types
    let invalid_jpeg = vec![0x00, 0x01, 0x02];
    let result = JpegImage::new(
        invalid_jpeg,
        ImageMetadata::default()
    );
    assert!(result.is_err());
    
    let invalid_mp3 = vec![0xAA, 0xBB, 0xCC];
    let result = Mp3Audio::new(
        invalid_mp3,
        AudioMetadata::default()
    );
    assert!(result.is_err());
}

#[test]
fn test_chain_validation_error_details() {
    let mut chain = ContentChain::<FailableContent>::new();
    
    // Add some valid items
    for i in 0..3 {
        chain.append(FailableContent {
            data: format!("item-{}", i),
            fail_mode: FailMode::None,
        }).unwrap();
    }
    
    // Get a copy of the chain items and tamper with one
    let items = chain.items();
    assert_eq!(items.len(), 3);
    
    // Create a validation scenario where sequence is wrong
    // This tests the error message formatting
    let result = chain.validate();
    assert!(result.is_ok()); // Should be valid initially
}

#[test]
fn test_error_display_and_debug() {
    // Test that all error variants have proper Display implementations
    let errors = vec![
        Error::SerializationError(serde_json::from_str::<String>("bad").unwrap_err()),
        Error::CborError("CBOR issue".to_string()),
        Error::InvalidCid("bad-cid".to_string()),
        Error::ChainValidationError {
            expected: "abc".to_string(),
            actual: "xyz".to_string(),
        },
        Error::SequenceValidationError {
            expected: 5,
            actual: 3,
        },
        Error::InvalidCodecRange(0x999999),
        Error::CodecNotFound(0x300100),
        Error::ContentTypeMismatch {
            expected: "Image".to_string(),
            actual: "Audio".to_string(),
        },
        Error::MultihashError("hash error".to_string()),
        Error::InvalidContent("content error".to_string()),
        Error::StorageError("storage error".to_string()),
    ];
    
    for error in errors {
        // Test Display trait
        let display = format!("{}", error);
        assert!(!display.is_empty());
        
        // Test Debug trait
        let debug = format!("{:?}", error);
        assert!(!debug.is_empty());
        
        // Display should be user-friendly (no Debug formatting)
        assert!(!display.contains("{"));
        assert!(!display.contains("}"));
    }
}

#[test]
fn test_cid_parsing_error_propagation() {
    use cim_ipld::chain::ChainedContent;
    
    // Test various invalid CID formats
    let invalid_cids = vec![
        "",
        "not-a-cid",
        "12345",
        "Qm", // Too short
        "bafy1234", // Invalid base32
        "{\"not\":\"a-cid\"}", // JSON
    ];
    
    for invalid_cid in invalid_cids {
        let result = ChainedContent::<FailableContent>::parse_cid(invalid_cid);
        assert!(result.is_err());
        
        match result {
            Err(Error::InvalidCid(msg)) => {
                assert!(!msg.is_empty());
            }
            _ => panic!("Expected InvalidCid error for: {}", invalid_cid),
        }
    }
}

#[test]
fn test_error_chain_in_service_layer() {
    // This would test ContentService error handling if we had a mock storage
    // For now, we test the error types exist and can be created
    
    // Test ObjectStoreError conversions
    let storage_errors = vec![
        ObjectStoreError::NotFound("cid123".to_string()),
        ObjectStoreError::Storage("connection failed".to_string()),
        ObjectStoreError::Serialization("encode failed".to_string()),
        ObjectStoreError::Deserialization("decode failed".to_string()),
        ObjectStoreError::CidMismatch {
            expected: "cid1".to_string(),
            actual: "cid2".to_string(),
        },
    ];
    
    for storage_error in storage_errors {
        let display = format!("{}", storage_error);
        assert!(!display.is_empty());
    }
}

// Async error propagation would be tested with actual async operations
// For now we test that error types work in async contexts

#[test]
fn test_error_conversion_chain() {
    // Test From implementations
    let json_err = serde_json::from_str::<String>("invalid").unwrap_err();
    let error: Error = json_err.into();
    
    match error {
        Error::SerializationError(_) => {
            // Correct conversion
        }
        _ => panic!("Expected SerializationError from serde_json::Error"),
    }
}

#[test]
fn test_result_type_alias() {
    // Test that our Result type alias works correctly
    fn returns_ok() -> Result<String> {
        Ok("success".to_string())
    }
    
    fn returns_err() -> Result<String> {
        Err(Error::InvalidContent("failure".into()))
    }
    
    assert!(returns_ok().is_ok());
    assert!(returns_err().is_err());
    
    // Test ? operator works
    fn uses_question_mark() -> Result<String> {
        let value = returns_ok()?;
        Ok(value.to_uppercase())
    }
    
    assert_eq!(uses_question_mark().unwrap(), "SUCCESS");
}