// Copyright 2025 Cowboy AI, LLC.

//! Content Types Demonstration
//!
//! This example showcases the various content types supported by CIM-IPLD

use cim_ipld::{
    // Document types
    MarkdownDocument, TextDocument, DocumentMetadata,
    
    // Image types
    JpegImage, PngImage, ImageMetadata,
    
    // Audio types
    Mp3Audio, AudioMetadata,
    
    // Detection utilities
    detect_content_type, content_type_name,
    
    // Codec operations
    DagJsonCodec,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Content Types Demonstration ===\n");
    
    // Document types
    demo_documents()?;
    
    // Image types
    demo_images()?;
    
    // Audio types
    demo_audio()?;
    
    // Content detection
    demo_content_detection()?;
    
    Ok(())
}

fn demo_documents() -> Result<(), Box<dyn std::error::Error>> {
    println!("1. Document Types:");
    
    // Text document
    let text_doc = TextDocument {
        content: "This is a plain text document.".to_string(),
        metadata: DocumentMetadata {
            title: Some("Plain Text Example".to_string()),
            author: Some("Demo Author".to_string()),
            tags: vec!["example".to_string(), "text".to_string()],
            ..Default::default()
        },
    };
    
    println!("  Text Document:");
    println!("    Title: {:?}", text_doc.metadata.title);
    println!("    Content: {}", text_doc.content);
    
    // Markdown document
    let markdown_doc = MarkdownDocument {
        content: r#"# Markdown Example

This is a **markdown** document with:
- Lists
- *Emphasis*
- [Links](https://example.com)
"#.to_string(),
        metadata: DocumentMetadata {
            title: Some("Markdown Guide".to_string()),
            author: Some("Doc Writer".to_string()),
            ..Default::default()
        },
    };
    
    println!("\n  Markdown Document:");
    println!("    Title: {:?}", markdown_doc.metadata.title);
    println!("    Author: {:?}", markdown_doc.metadata.author);
    println!("    Content preview: {}", &markdown_doc.content[0..50]);
    
    // Encode and show size
    let encoded = DagJsonCodec::encode(&markdown_doc)?;
    println!("    Encoded size: {} bytes\n", encoded.len());
    
    Ok(())
}

fn demo_images() -> Result<(), Box<dyn std::error::Error>> {
    println!("2. Image Types:");
    
    // JPEG image (simulated)
    let jpeg = JpegImage {
        data: vec![0xFF, 0xD8, 0xFF, 0xE0], // JPEG header bytes
        metadata: ImageMetadata {
            width: Some(1920),
            height: Some(1080),
            format: Some("jpeg".to_string()),
            color_space: Some("sRGB".to_string()),
            ..Default::default()
        },
    };
    
    println!("  JPEG Image:");
    println!("    Dimensions: {}x{}", 
        jpeg.metadata.width.unwrap_or(0),
        jpeg.metadata.height.unwrap_or(0)
    );
    println!("    Format: {:?}", jpeg.metadata.format);
    println!("    Color space: {:?}", jpeg.metadata.color_space);
    
    // PNG image (simulated)
    let png = PngImage {
        data: vec![0x89, 0x50, 0x4E, 0x47], // PNG header bytes
        metadata: ImageMetadata {
            width: Some(800),
            height: Some(600),
            format: Some("png".to_string()),
            ..Default::default()
        },
    };
    
    println!("\n  PNG Image:");
    println!("    Dimensions: {}x{}", 
        png.metadata.width.unwrap_or(0),
        png.metadata.height.unwrap_or(0)
    );
    println!("    Format: {:?}", png.metadata.format);
    println!();
    
    Ok(())
}

fn demo_audio() -> Result<(), Box<dyn std::error::Error>> {
    println!("3. Audio Types:");
    
    // MP3 audio (simulated)
    let mp3 = Mp3Audio {
        data: vec![0xFF, 0xFB], // MP3 frame header
        metadata: AudioMetadata {
            duration_ms: Some(180000), // 3 minutes
            bitrate: Some(320000), // 320 kbps
            sample_rate: Some(44100),
            channels: Some(2), // Stereo
            codec: Some("mp3".to_string()),
            title: Some("Example Song".to_string()),
            artist: Some("Demo Artist".to_string()),
            album: Some("Test Album".to_string()),
            tags: vec!["demo".to_string(), "example".to_string()],
            ..Default::default()
        },
    };
    
    println!("  MP3 Audio:");
    println!("    Title: {:?}", mp3.metadata.title);
    println!("    Artist: {:?}", mp3.metadata.artist);
    println!("    Album: {:?}", mp3.metadata.album);
    println!("    Duration: {} seconds", mp3.metadata.duration_ms.unwrap_or(0) / 1000);
    println!("    Bitrate: {} kbps", mp3.metadata.bitrate.unwrap_or(0) / 1000);
    println!("    Sample rate: {} Hz", mp3.metadata.sample_rate.unwrap_or(0));
    println!("    Channels: {}", mp3.metadata.channels.unwrap_or(0));
    println!();
    
    Ok(())
}

fn demo_content_detection() -> Result<(), Box<dyn std::error::Error>> {
    println!("4. Content Type Detection:");
    
    println!("  Content header detection:");
    
    // Test with actual file headers
    let headers = vec![
        (vec![0xFF, 0xD8, 0xFF], "JPEG"),
        (vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A], "PNG"),
        (vec![0x25, 0x50, 0x44, 0x46], "PDF"),
        (vec![0xFF, 0xFB], "MP3"),
    ];
    
    for (header, expected) in headers {
        let content_type = detect_content_type(&header);
        let type_name = if let Some(ct) = content_type {
            content_type_name(ct)
        } else {
            "Unknown"
        };
        println!("    {:?} -> {} (expected: {})", 
            &header[..4.min(header.len())], 
            type_name, 
            expected
        );
    }
    
    Ok(())
}