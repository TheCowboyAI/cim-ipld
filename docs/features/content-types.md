# Content Types in CIM-IPLD

## Overview

CIM-IPLD provides comprehensive support for various content types with type-safe handling, automatic format detection, and rich metadata support. Each content type implements the `TypedContent` trait, ensuring consistent behavior across the system.

## Supported Content Types

### Documents

#### Text Documents

**Type**: `TextDocument`  
**Extensions**: `.txt`, `.text`  
**MIME**: `text/plain`

```rust
use cim_ipld::content_types::{TextDocument, DocumentMetadata};

let doc = TextDocument {
    content: "Hello, World!".to_string(),
    metadata: DocumentMetadata {
        title: Some("My Document".to_string()),
        author: Some("Alice".to_string()),
        ..Default::default()
    },
};
```

#### Markdown Documents

**Type**: `MarkdownDocument`  
**Extensions**: `.md`, `.markdown`  
**MIME**: `text/markdown`

```rust
let markdown = MarkdownDocument {
    content: "# Title\n\nContent with **bold** text".to_string(),
    metadata: DocumentMetadata {
        title: Some("README".to_string()),
        ..Default::default()
    },
};
```

#### PDF Documents

**Type**: `PdfDocument`  
**Extensions**: `.pdf`  
**MIME**: `application/pdf`

```rust
let pdf = PdfDocument {
    data: pdf_bytes,
    metadata: DocumentMetadata {
        title: Some("Report.pdf".to_string()),
        page_count: Some(10),
        ..Default::default()
    },
};

// Verify PDF format
if PdfDocument::verify(&pdf_bytes) {
    println!("Valid PDF document");
}
```

#### Word Documents

**Type**: `WordDocument`  
**Extensions**: `.docx`  
**MIME**: `application/vnd.openxmlformats-officedocument.wordprocessingml.document`  
**Feature**: Requires `office` feature flag

### Images

#### JPEG Images

**Type**: `JpegImage`  
**Extensions**: `.jpg`, `.jpeg`  
**MIME**: `image/jpeg`

```rust
use cim_ipld::content_types::{JpegImage, ImageMetadata};

let image = JpegImage {
    data: jpeg_bytes,
    metadata: ImageMetadata {
        width: Some(1920),
        height: Some(1080),
        format: Some("JPEG".to_string()),
        ..Default::default()
    },
};
```

#### PNG Images

**Type**: `PngImage`  
**Extensions**: `.png`  
**MIME**: `image/png`

#### GIF Images

**Type**: `GifImage`  
**Extensions**: `.gif`  
**MIME**: `image/gif`

#### WebP Images

**Type**: `WebPImage`  
**Extensions**: `.webp`  
**MIME**: `image/webp`

### Audio

#### MP3 Audio

**Type**: `Mp3Audio`  
**Extensions**: `.mp3`  
**MIME**: `audio/mpeg`

```rust
use cim_ipld::content_types::{Mp3Audio, AudioMetadata};

let audio = Mp3Audio {
    data: mp3_bytes,
    metadata: AudioMetadata {
        duration: Some(180.5), // seconds
        bitrate: Some(320),    // kbps
        artist: Some("Artist Name".to_string()),
        title: Some("Song Title".to_string()),
        album: Some("Album Name".to_string()),
        ..Default::default()
    },
};
```

#### WAV Audio

**Type**: `WavAudio`  
**Extensions**: `.wav`  
**MIME**: `audio/wav`

#### FLAC Audio

**Type**: `FlacAudio`  
**Extensions**: `.flac`  
**MIME**: `audio/flac`

#### AAC Audio

**Type**: `AacAudio`  
**Extensions**: `.aac`, `.m4a`  
**MIME**: `audio/aac`

#### Ogg Vorbis

**Type**: `OggAudio`  
**Extensions**: `.ogg`, `.oga`  
**MIME**: `audio/ogg`

### Video

#### MP4 Video

**Type**: `Mp4Video`  
**Extensions**: `.mp4`, `.m4v`  
**MIME**: `video/mp4`

```rust
use cim_ipld::content_types::{Mp4Video, VideoMetadata};

let video = Mp4Video {
    data: mp4_bytes,
    metadata: VideoMetadata {
        duration: Some(3600.0), // seconds
        width: Some(1920),
        height: Some(1080),
        codec: Some("h264".to_string()),
        bitrate: Some(5000),    // kbps
        ..Default::default()
    },
};
```

#### QuickTime Video

**Type**: `MovVideo`  
**Extensions**: `.mov`  
**MIME**: `video/quicktime`

#### Matroska Video

**Type**: `MkvVideo`  
**Extensions**: `.mkv`  
**MIME**: `video/x-matroska`

#### AVI Video

**Type**: `AviVideo`  
**Extensions**: `.avi`  
**MIME**: `video/x-msvideo`

### Special Types

#### Events

**Type**: `Event`  
**Purpose**: Event sourcing and audit logs

```rust
#[derive(Serialize, Deserialize)]
struct CustomEvent {
    event_type: String,
    aggregate_id: String,
    payload: serde_json::Value,
    timestamp: u64,
}

impl TypedContent for CustomEvent {
    const CODEC: u64 = 0x300000;
    const CONTENT_TYPE: ContentType = ContentType::Event;
}
```

## Content Type Detection

### Automatic Detection

```rust
use cim_ipld::content_types::ContentType;

// Detect from file bytes
let content_type = ContentType::detect_from_bytes(&file_bytes);

match content_type {
    Some(ContentType::JpegImage) => println!("JPEG image detected"),
    Some(ContentType::PdfDocument) => println!("PDF document detected"),
    Some(ct) => println!("Detected: {:?}", ct),
    None => println!("Unknown content type"),
}
```

### Detection with Filename

```rust
// Use filename hint for better accuracy
let content_type = ContentType::detect_with_name(&file_bytes, "document.pdf");
```

### Magic Byte Detection

CIM-IPLD uses magic bytes (file signatures) for accurate content detection:

| Format | Magic Bytes | Description |
|--------|-------------|-------------|
| JPEG | `FF D8 FF` | JPEG image start |
| PNG | `89 50 4E 47 0D 0A 1A 0A` | PNG signature |
| PDF | `25 50 44 46` (`%PDF`) | PDF header |
| GIF | `47 49 46 38` (`GIF8`) | GIF87a/GIF89a |
| MP3 | `FF FB` or `49 44 33` | MPEG audio or ID3 |
| WebP | `52 49 46 46 ... 57 45 42 50` | RIFF WebP |

## Metadata Structures

### DocumentMetadata

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub created: Option<DateTime<Utc>>,
    pub modified: Option<DateTime<Utc>>,
    pub tags: Option<Vec<String>>,
    pub description: Option<String>,
    pub language: Option<String>,
    pub word_count: Option<usize>,
    pub page_count: Option<usize>,
    pub custom: Option<HashMap<String, String>>,
}
```

### ImageMetadata

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ImageMetadata {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub format: Option<String>,
    pub color_mode: Option<String>,  // RGB, CMYK, Grayscale
    pub dpi: Option<u32>,
    pub created: Option<DateTime<Utc>>,
    pub camera: Option<CameraInfo>,
    pub location: Option<GeoLocation>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraInfo {
    pub make: Option<String>,
    pub model: Option<String>,
    pub lens: Option<String>,
    pub exposure: Option<String>,
    pub aperture: Option<String>,
    pub iso: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoLocation {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: Option<f64>,
}
```

### AudioMetadata

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AudioMetadata {
    pub duration: Option<f64>,      // seconds
    pub bitrate: Option<u32>,       // kbps
    pub sample_rate: Option<u32>,   // Hz
    pub channels: Option<u8>,       // mono=1, stereo=2
    pub codec: Option<String>,
    pub artist: Option<String>,
    pub title: Option<String>,
    pub album: Option<String>,
    pub year: Option<u16>,
    pub genre: Option<String>,
    pub track: Option<u16>,
}
```

### VideoMetadata

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VideoMetadata {
    pub duration: Option<f64>,      // seconds
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub frame_rate: Option<f32>,    // fps
    pub bitrate: Option<u32>,       // kbps
    pub codec: Option<String>,
    pub audio_codec: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub created: Option<DateTime<Utc>>,
}
```

## Custom Content Types

### Creating Custom Types

```rust
use cim_ipld::{TypedContent, ContentType};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InvoiceDocument {
    invoice_number: String,
    customer: String,
    items: Vec<LineItem>,
    total: f64,
    metadata: InvoiceMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InvoiceMetadata {
    created: DateTime<Utc>,
    due_date: DateTime<Utc>,
    status: String,
}

impl TypedContent for InvoiceDocument {
    const CODEC: u64 = 0x400001; // Custom codec
    const CONTENT_TYPE: ContentType = ContentType::Custom(0x400001);
    
    fn verify(data: &[u8]) -> bool {
        // Custom verification logic
        serde_json::from_slice::<Self>(data).is_ok()
    }
}
```

### Registering Custom Detection

```rust
// Add custom MIME type mapping
store.register_mime_type("application/x-invoice", ContentType::Custom(0x400001));

// Add custom extension mapping
store.register_extension("inv", ContentType::Custom(0x400001));
```

## Content Validation

### Format Verification

```rust
// Verify content matches expected format
if !JpegImage::verify(&data) {
    return Err("Not a valid JPEG image".into());
}

// Type-safe retrieval with automatic verification
let image: JpegImage = store.get_typed(&cid).await?;
```

### Size Limits

```rust
const MAX_IMAGE_SIZE: usize = 50 * 1024 * 1024; // 50MB

if image_data.len() > MAX_IMAGE_SIZE {
    return Err("Image too large".into());
}
```

## Performance Considerations

### Lazy Loading

```rust
// Load metadata without full content
let metadata = store.get_metadata(&cid).await?;

// Stream large content
let stream = store.get_stream(&cid).await?;
```

### Content Chunking

```rust
// For large files, use chunked storage
pub struct ChunkedContent {
    chunks: Vec<Cid>,
    total_size: usize,
    chunk_size: usize,
}
```

## Best Practices

1. **Always Verify**: Use type-specific `verify()` methods before storing
2. **Rich Metadata**: Populate metadata fields for better searchability
3. **Appropriate Types**: Use the most specific content type available
4. **Size Awareness**: Consider chunking for large content
5. **Custom Types**: Create custom types for domain-specific content

## Content Type Migration

When upgrading content types:

```rust
// Old version
#[derive(Serialize, Deserialize)]
struct DocumentV1 {
    content: String,
}

// New version with migration
#[derive(Serialize, Deserialize)]
struct DocumentV2 {
    content: String,
    version: u32,
    metadata: DocumentMetadata,
}

impl From<DocumentV1> for DocumentV2 {
    fn from(v1: DocumentV1) -> Self {
        DocumentV2 {
            content: v1.content,
            version: 2,
            metadata: Default::default(),
        }
    }
}
```

## Future Content Types

Planned support for:
- **3D Models**: `.obj`, `.gltf`, `.fbx`
- **Archives**: `.zip`, `.tar`, `.7z`
- **Spreadsheets**: `.xlsx`, `.csv`
- **Presentations**: `.pptx`
- **Scientific Data**: `.hdf5`, `.netcdf`


---
Copyright 2025 Cowboy AI, LLC.
