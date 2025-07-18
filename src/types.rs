// Copyright 2025 Cowboy AI, LLC.

//! Core types for CIM-IPLD

use serde::{Deserialize, Serialize};

// Re-export CID for convenience
pub use cid::Cid;

/// Content type identifiers for CIM
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContentType {
    // Core CIM Types (0x300000-0x30FFFF)
    /// Generic event type
    Event,
    /// Graph structure
    Graph,
    /// Graph node
    Node,
    /// Graph edge
    Edge,
    /// Command message
    Command,
    /// Query message
    Query,

    // Document Types (0x310000-0x31FFFF)
    /// Markdown document
    Markdown,
    /// JSON document
    Json,
    /// YAML document
    Yaml,
    /// TOML document
    Toml,

    // Media Types (0x320000-0x32FFFF)
    /// Image file
    Image,
    /// Video file
    Video,
    /// Audio file
    Audio,

    // Extension point for custom types
    /// Custom content type with specific codec
    Custom(u64),
}

impl ContentType {
    /// Get the codec identifier for this content type
    pub fn codec(&self) -> u64 {
        match self {
            // Core types
            ContentType::Event => 0x300000,
            ContentType::Graph => 0x300001,
            ContentType::Node => 0x300002,
            ContentType::Edge => 0x300003,
            ContentType::Command => 0x300004,
            ContentType::Query => 0x300005,

            // Document types
            ContentType::Markdown => 0x310000,
            ContentType::Json => 0x310001,
            ContentType::Yaml => 0x310002,
            ContentType::Toml => 0x310003,

            // Media types
            ContentType::Image => 0x320000,
            ContentType::Video => 0x320001,
            ContentType::Audio => 0x320002,

            // Custom
            ContentType::Custom(codec) => *codec,
        }
    }

    /// Create a ContentType from a codec identifier
    pub fn from_codec(codec: u64) -> Option<Self> {
        match codec {
            0x300000 => Some(ContentType::Event),
            0x300001 => Some(ContentType::Graph),
            0x300002 => Some(ContentType::Node),
            0x300003 => Some(ContentType::Edge),
            0x300004 => Some(ContentType::Command),
            0x300005 => Some(ContentType::Query),

            0x310000 => Some(ContentType::Markdown),
            0x310001 => Some(ContentType::Json),
            0x310002 => Some(ContentType::Yaml),
            0x310003 => Some(ContentType::Toml),

            0x320000 => Some(ContentType::Image),
            0x320001 => Some(ContentType::Video),
            0x320002 => Some(ContentType::Audio),

            // Custom types in valid range
            c if (0x300000..=0x3FFFFF).contains(&c) => Some(ContentType::Custom(c)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_type_codec() {
        // Test core types
        assert_eq!(ContentType::Event.codec(), 0x300000);
        assert_eq!(ContentType::Graph.codec(), 0x300001);
        assert_eq!(ContentType::Node.codec(), 0x300002);
        assert_eq!(ContentType::Edge.codec(), 0x300003);
        assert_eq!(ContentType::Command.codec(), 0x300004);
        assert_eq!(ContentType::Query.codec(), 0x300005);

        // Test document types
        assert_eq!(ContentType::Markdown.codec(), 0x310000);
        assert_eq!(ContentType::Json.codec(), 0x310001);
        assert_eq!(ContentType::Yaml.codec(), 0x310002);
        assert_eq!(ContentType::Toml.codec(), 0x310003);

        // Test media types
        assert_eq!(ContentType::Image.codec(), 0x320000);
        assert_eq!(ContentType::Video.codec(), 0x320001);
        assert_eq!(ContentType::Audio.codec(), 0x320002);

        // Test custom type
        assert_eq!(ContentType::Custom(0x330000).codec(), 0x330000);
        assert_eq!(ContentType::Custom(0x3FFFFF).codec(), 0x3FFFFF);
    }

    #[test]
    fn test_content_type_from_codec() {
        // Test core types
        assert_eq!(ContentType::from_codec(0x300000), Some(ContentType::Event));
        assert_eq!(ContentType::from_codec(0x300001), Some(ContentType::Graph));
        assert_eq!(ContentType::from_codec(0x300002), Some(ContentType::Node));
        assert_eq!(ContentType::from_codec(0x300003), Some(ContentType::Edge));
        assert_eq!(ContentType::from_codec(0x300004), Some(ContentType::Command));
        assert_eq!(ContentType::from_codec(0x300005), Some(ContentType::Query));

        // Test document types
        assert_eq!(ContentType::from_codec(0x310000), Some(ContentType::Markdown));
        assert_eq!(ContentType::from_codec(0x310001), Some(ContentType::Json));
        assert_eq!(ContentType::from_codec(0x310002), Some(ContentType::Yaml));
        assert_eq!(ContentType::from_codec(0x310003), Some(ContentType::Toml));

        // Test media types
        assert_eq!(ContentType::from_codec(0x320000), Some(ContentType::Image));
        assert_eq!(ContentType::from_codec(0x320001), Some(ContentType::Video));
        assert_eq!(ContentType::from_codec(0x320002), Some(ContentType::Audio));

        // Test custom types in valid range
        assert_eq!(ContentType::from_codec(0x330000), Some(ContentType::Custom(0x330000)));
        assert_eq!(ContentType::from_codec(0x3FFFFF), Some(ContentType::Custom(0x3FFFFF)));
        assert_eq!(ContentType::from_codec(0x300006), Some(ContentType::Custom(0x300006)));

        // Test invalid codec values
        assert_eq!(ContentType::from_codec(0x000000), None);
        assert_eq!(ContentType::from_codec(0x2FFFFF), None);
        assert_eq!(ContentType::from_codec(0x400000), None);
        assert_eq!(ContentType::from_codec(u64::MAX), None);
    }

    #[test]
    fn test_content_type_roundtrip() {
        // Test that from_codec(codec()) == identity for all variants
        let types = vec![
            ContentType::Event,
            ContentType::Graph,
            ContentType::Node,
            ContentType::Edge,
            ContentType::Command,
            ContentType::Query,
            ContentType::Markdown,
            ContentType::Json,
            ContentType::Yaml,
            ContentType::Toml,
            ContentType::Image,
            ContentType::Video,
            ContentType::Audio,
            ContentType::Custom(0x330000),
            ContentType::Custom(0x3FFFFF),
        ];

        for content_type in types {
            let codec = content_type.codec();
            let roundtrip = ContentType::from_codec(codec);
            assert_eq!(roundtrip, Some(content_type));
        }
    }

    #[test]
    fn test_content_type_traits() {
        // Test Debug trait
        let event = ContentType::Event;
        assert_eq!(format!("{:?}", event), "Event");

        let custom = ContentType::Custom(0x330000);
        assert_eq!(format!("{:?}", custom), "Custom(3342336)");

        // Test Clone trait
        let original = ContentType::Graph;
        let cloned = original.clone();
        assert_eq!(original, cloned);

        // Test Copy trait (implicitly tested by passing by value)
        let copy_test = ContentType::Node;
        let _copy1 = copy_test;
        let _copy2 = copy_test; // Would fail if Copy wasn't implemented

        // Test PartialEq and Eq
        assert_eq!(ContentType::Edge, ContentType::Edge);
        assert_ne!(ContentType::Edge, ContentType::Node);

        // Test Hash trait
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(ContentType::Command);
        set.insert(ContentType::Query);
        set.insert(ContentType::Command); // Duplicate
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_content_type_serialization() {
        // Test Serialize and Deserialize
        let original = ContentType::Markdown;
        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: ContentType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(original, deserialized);

        // Test custom type serialization
        let custom = ContentType::Custom(0x350000);
        let serialized = serde_json::to_string(&custom).unwrap();
        let deserialized: ContentType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(custom, deserialized);
    }

    #[test]
    fn test_cid_reexport() {
        // Test that Cid is properly re-exported
        use crate::types::Cid as ReexportedCid;
        
        // Create a simple CID to test the re-export works
        let data = b"hello world";
        let hash = blake3::hash(data);
        let multihash = multihash::Multihash::wrap(0x1e, hash.as_bytes()).unwrap();
        let cid = ReexportedCid::new_v1(0x71, multihash); // 0x71 = dag-cbor
        
        // If this compiles and runs, the re-export is working
        assert_eq!(cid.version(), cid::Version::V1);
    }

    #[test]
    fn test_content_type_boundary_values() {
        // Test boundary values for custom codec range
        assert_eq!(ContentType::from_codec(0x300000 - 1), None);
        assert_eq!(ContentType::from_codec(0x300000), Some(ContentType::Event));
        assert_eq!(ContentType::from_codec(0x3FFFFF), Some(ContentType::Custom(0x3FFFFF)));
        assert_eq!(ContentType::from_codec(0x400000), None);
    }

    #[test]
    fn test_content_type_exhaustive_match() {
        // This test ensures all variants are covered in codec() method
        fn exhaustive_codec(ct: ContentType) -> u64 {
            match ct {
                ContentType::Event => 0x300000,
                ContentType::Graph => 0x300001,
                ContentType::Node => 0x300002,
                ContentType::Edge => 0x300003,
                ContentType::Command => 0x300004,
                ContentType::Query => 0x300005,
                ContentType::Markdown => 0x310000,
                ContentType::Json => 0x310001,
                ContentType::Yaml => 0x310002,
                ContentType::Toml => 0x310003,
                ContentType::Image => 0x320000,
                ContentType::Video => 0x320001,
                ContentType::Audio => 0x320002,
                ContentType::Custom(c) => c,
            }
        }

        // Verify our implementation matches
        let types = vec![
            ContentType::Event,
            ContentType::Graph,
            ContentType::Custom(0x350000),
        ];

        for ct in types {
            assert_eq!(ct.codec(), exhaustive_codec(ct));
        }
    }
}
