// Copyright 2025 Cowboy AI, LLC.

//! Content types for common file formats with verification
//!
//! This module provides typed content wrappers for common file formats
//! including documents, images, audio, and video files. Each type includes
//! verification to ensure the content matches the expected format.

pub mod encryption;
pub mod indexing;
pub mod persistence;
pub mod service;
pub mod transformers;

use crate::{TypedContent, ContentType, Error, Result};
use serde::{Deserialize, Serialize};

/// Magic bytes for file format detection
mod magic {
    pub const PDF: &[u8] = b"%PDF-";
    pub const DOCX: &[u8] = b"PK\x03\x04";
    pub const PNG: &[u8] = b"\x89PNG\r\n\x1a\n";
    pub const JPEG: &[u8] = b"\xFF\xD8\xFF";
    pub const GIF: &[u8] = b"GIF8";
    pub const WEBP: &[u8] = b"RIFF";
    pub const MP3_ID3: &[u8] = b"ID3";
    pub const MP3_SYNC: &[u8] = b"\xFF\xFB";
    pub const WAV: &[u8] = b"RIFF";
    pub const FLAC: &[u8] = b"fLaC";
    pub const OGG: &[u8] = b"OggS";
    pub const AAC_ADTS: &[u8] = b"\xFF\xF1";
    // MP4 and MOV are detected by checking ftyp box at offset 4
    // pub const MP4: &[u8] = b"\x00\x00\x00\x20ftypmp4";
    // pub const MOV: &[u8] = b"\x00\x00\x00\x14ftyp";
    pub const MKV: &[u8] = b"\x1A\x45\xDF\xA3";
    pub const AVI: &[u8] = b"RIFF";
}

/// Verified PDF document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfDocument {
    /// The raw PDF data
    pub data: Vec<u8>,
    /// Optional metadata
    pub metadata: DocumentMetadata,
}

/// Verified DOCX document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocxDocument {
    /// The raw DOCX data
    pub data: Vec<u8>,
    /// Optional metadata
    pub metadata: DocumentMetadata,
}

/// Verified Markdown document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownDocument {
    /// The markdown content as UTF-8 text
    pub content: String,
    /// Optional metadata
    pub metadata: DocumentMetadata,
}

/// Verified plain text document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextDocument {
    /// The text content
    pub content: String,
    /// Optional metadata
    pub metadata: DocumentMetadata,
}

/// Common document metadata
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DocumentMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub created_at: Option<u64>,
    pub modified_at: Option<u64>,
    pub tags: Vec<String>,
    pub language: Option<String>,
}

/// Image formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImageContent {
    Jpeg(JpegImage),
    Png(PngImage),
    Gif(GifImage),
    WebP(WebPImage),
}

/// Verified JPEG image
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JpegImage {
    pub data: Vec<u8>,
    pub metadata: ImageMetadata,
}

/// Verified PNG image
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PngImage {
    pub data: Vec<u8>,
    pub metadata: ImageMetadata,
}

/// Verified GIF image
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GifImage {
    pub data: Vec<u8>,
    pub metadata: ImageMetadata,
}

/// Verified WebP image
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebPImage {
    pub data: Vec<u8>,
    pub metadata: ImageMetadata,
}

/// Common image metadata
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImageMetadata {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub format: Option<String>,
    pub color_space: Option<String>,
    pub compression: Option<String>,
    pub tags: Vec<String>,
}

/// Audio formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioContent {
    Mp3(Mp3Audio),
    Wav(WavAudio),
    Flac(FlacAudio),
    Aac(AacAudio),
    Ogg(OggAudio),
}

/// Verified MP3 audio
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mp3Audio {
    pub data: Vec<u8>,
    pub metadata: AudioMetadata,
}

/// Verified WAV audio
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WavAudio {
    pub data: Vec<u8>,
    pub metadata: AudioMetadata,
}

/// Verified FLAC audio
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlacAudio {
    pub data: Vec<u8>,
    pub metadata: AudioMetadata,
}

/// Verified AAC audio
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AacAudio {
    pub data: Vec<u8>,
    pub metadata: AudioMetadata,
}

/// Verified OGG audio
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OggAudio {
    pub data: Vec<u8>,
    pub metadata: AudioMetadata,
}

/// Common audio metadata
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AudioMetadata {
    pub duration_ms: Option<u64>,
    pub bitrate: Option<u32>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u8>,
    pub codec: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub title: Option<String>,
    pub year: Option<u16>,
    pub tags: Vec<String>,
}

/// Video formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VideoContent {
    Mp4(Mp4Video),
    Mov(MovVideo),
    Mkv(MkvVideo),
    Avi(AviVideo),
}

/// Verified MP4 video
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mp4Video {
    pub data: Vec<u8>,
    pub metadata: VideoMetadata,
}

/// Verified MOV video
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovVideo {
    pub data: Vec<u8>,
    pub metadata: VideoMetadata,
}

/// Verified MKV video
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MkvVideo {
    pub data: Vec<u8>,
    pub metadata: VideoMetadata,
}

/// Verified AVI video
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AviVideo {
    pub data: Vec<u8>,
    pub metadata: VideoMetadata,
}

/// Common video metadata
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VideoMetadata {
    pub duration_ms: Option<u64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub frame_rate: Option<f32>,
    pub video_codec: Option<String>,
    pub audio_codec: Option<String>,
    pub bitrate: Option<u32>,
    pub tags: Vec<String>,
}

/// Content type codes for each format
pub mod codec {
    // Document types (0x600000 - 0x60FFFF)
    pub const PDF: u64 = 0x600001;
    pub const DOCX: u64 = 0x600002;
    pub const MARKDOWN: u64 = 0x600003;
    pub const TEXT: u64 = 0x600004;
    
    // Image types (0x610000 - 0x61FFFF)
    pub const JPEG: u64 = 0x610001;
    pub const PNG: u64 = 0x610002;
    pub const GIF: u64 = 0x610003;
    pub const WEBP: u64 = 0x610004;
    
    // Audio types (0x620000 - 0x62FFFF)
    pub const MP3: u64 = 0x620001;
    pub const WAV: u64 = 0x620002;
    pub const FLAC: u64 = 0x620003;
    pub const AAC: u64 = 0x620004;
    pub const OGG: u64 = 0x620005;
    
    // Video types (0x630000 - 0x63FFFF)
    pub const MP4: u64 = 0x630001;
    pub const MOV: u64 = 0x630002;
    pub const MKV: u64 = 0x630003;
    pub const AVI: u64 = 0x630004;
}

// Implement TypedContent for document types

impl TypedContent for PdfDocument {
    const CODEC: u64 = codec::PDF;
    const CONTENT_TYPE: ContentType = ContentType::Custom(codec::PDF);
}

impl TypedContent for DocxDocument {
    const CODEC: u64 = codec::DOCX;
    const CONTENT_TYPE: ContentType = ContentType::Custom(codec::DOCX);
}

impl TypedContent for MarkdownDocument {
    const CODEC: u64 = codec::MARKDOWN;
    const CONTENT_TYPE: ContentType = ContentType::Custom(codec::MARKDOWN);
}

impl TypedContent for TextDocument {
    const CODEC: u64 = codec::TEXT;
    const CONTENT_TYPE: ContentType = ContentType::Custom(codec::TEXT);
}

// Implement TypedContent for image types

impl TypedContent for JpegImage {
    const CODEC: u64 = codec::JPEG;
    const CONTENT_TYPE: ContentType = ContentType::Custom(codec::JPEG);
}

impl TypedContent for PngImage {
    const CODEC: u64 = codec::PNG;
    const CONTENT_TYPE: ContentType = ContentType::Custom(codec::PNG);
}

impl TypedContent for GifImage {
    const CODEC: u64 = codec::GIF;
    const CONTENT_TYPE: ContentType = ContentType::Custom(codec::GIF);
}

impl TypedContent for WebPImage {
    const CODEC: u64 = codec::WEBP;
    const CONTENT_TYPE: ContentType = ContentType::Custom(codec::WEBP);
}

// Implement TypedContent for audio types

impl TypedContent for Mp3Audio {
    const CODEC: u64 = codec::MP3;
    const CONTENT_TYPE: ContentType = ContentType::Custom(codec::MP3);
}

impl TypedContent for WavAudio {
    const CODEC: u64 = codec::WAV;
    const CONTENT_TYPE: ContentType = ContentType::Custom(codec::WAV);
}

impl TypedContent for FlacAudio {
    const CODEC: u64 = codec::FLAC;
    const CONTENT_TYPE: ContentType = ContentType::Custom(codec::FLAC);
}

impl TypedContent for AacAudio {
    const CODEC: u64 = codec::AAC;
    const CONTENT_TYPE: ContentType = ContentType::Custom(codec::AAC);
}

impl TypedContent for OggAudio {
    const CODEC: u64 = codec::OGG;
    const CONTENT_TYPE: ContentType = ContentType::Custom(codec::OGG);
}

// Implement TypedContent for video types

impl TypedContent for Mp4Video {
    const CODEC: u64 = codec::MP4;
    const CONTENT_TYPE: ContentType = ContentType::Custom(codec::MP4);
}

impl TypedContent for MovVideo {
    const CODEC: u64 = codec::MOV;
    const CONTENT_TYPE: ContentType = ContentType::Custom(codec::MOV);
}

impl TypedContent for MkvVideo {
    const CODEC: u64 = codec::MKV;
    const CONTENT_TYPE: ContentType = ContentType::Custom(codec::MKV);
}

impl TypedContent for AviVideo {
    const CODEC: u64 = codec::AVI;
    const CONTENT_TYPE: ContentType = ContentType::Custom(codec::AVI);
}

// Verification functions

impl PdfDocument {
    /// Create a new PDF document with verification
    pub fn new(data: Vec<u8>, metadata: DocumentMetadata) -> Result<Self> {
        if !Self::verify(&data) {
            return Err(Error::InvalidContent("Not a valid PDF file".into()));
        }
        Ok(Self { data, metadata })
    }
    
    /// Verify that the data is a valid PDF
    pub fn verify(data: &[u8]) -> bool {
        data.starts_with(magic::PDF)
    }
}

impl DocxDocument {
    /// Create a new DOCX document with verification
    pub fn new(data: Vec<u8>, metadata: DocumentMetadata) -> Result<Self> {
        if !Self::verify(&data) {
            return Err(Error::InvalidContent("Not a valid DOCX file".into()));
        }
        Ok(Self { data, metadata })
    }
    
    /// Verify that the data is a valid DOCX (ZIP with specific structure)
    pub fn verify(data: &[u8]) -> bool {
        // DOCX files are ZIP files, check for PK header
        data.starts_with(magic::DOCX)
    }
}

impl MarkdownDocument {
    /// Create a new Markdown document
    pub fn new(content: String, metadata: DocumentMetadata) -> Result<Self> {
        // Markdown is plain text, no specific verification needed
        Ok(Self { content, metadata })
    }
    
    /// Create from UTF-8 bytes
    pub fn from_bytes(data: Vec<u8>, metadata: DocumentMetadata) -> Result<Self> {
        let content = String::from_utf8(data)
            .map_err(|_| Error::InvalidContent("Invalid UTF-8 in markdown".into()))?;
        Ok(Self { content, metadata })
    }
}

impl TextDocument {
    /// Create a new text document
    pub fn new(content: String, metadata: DocumentMetadata) -> Result<Self> {
        Ok(Self { content, metadata })
    }
    
    /// Create from UTF-8 bytes
    pub fn from_bytes(data: Vec<u8>, metadata: DocumentMetadata) -> Result<Self> {
        let content = String::from_utf8(data)
            .map_err(|_| Error::InvalidContent("Invalid UTF-8 in text".into()))?;
        Ok(Self { content, metadata })
    }
}

// Image verification

impl JpegImage {
    pub fn new(data: Vec<u8>, metadata: ImageMetadata) -> Result<Self> {
        if !Self::verify(&data) {
            return Err(Error::InvalidContent("Not a valid JPEG file".into()));
        }
        Ok(Self { data, metadata })
    }
    
    pub fn verify(data: &[u8]) -> bool {
        data.starts_with(magic::JPEG)
    }
}

impl PngImage {
    pub fn new(data: Vec<u8>, metadata: ImageMetadata) -> Result<Self> {
        if !Self::verify(&data) {
            return Err(Error::InvalidContent("Not a valid PNG file".into()));
        }
        Ok(Self { data, metadata })
    }
    
    pub fn verify(data: &[u8]) -> bool {
        data.starts_with(magic::PNG)
    }
}

impl GifImage {
    pub fn new(data: Vec<u8>, metadata: ImageMetadata) -> Result<Self> {
        if !Self::verify(&data) {
            return Err(Error::InvalidContent("Not a valid GIF file".into()));
        }
        Ok(Self { data, metadata })
    }
    
    pub fn verify(data: &[u8]) -> bool {
        data.starts_with(magic::GIF)
    }
}

impl WebPImage {
    pub fn new(data: Vec<u8>, metadata: ImageMetadata) -> Result<Self> {
        if !Self::verify(&data) {
            return Err(Error::InvalidContent("Not a valid WebP file".into()));
        }
        Ok(Self { data, metadata })
    }
    
    pub fn verify(data: &[u8]) -> bool {
        data.starts_with(magic::WEBP) && data.len() > 12 && &data[8..12] == b"WEBP"
    }
}

// Audio verification

impl Mp3Audio {
    pub fn new(data: Vec<u8>, metadata: AudioMetadata) -> Result<Self> {
        if !Self::verify(&data) {
            return Err(Error::InvalidContent("Not a valid MP3 file".into()));
        }
        Ok(Self { data, metadata })
    }
    
    pub fn verify(data: &[u8]) -> bool {
        data.starts_with(magic::MP3_ID3) || data.starts_with(magic::MP3_SYNC)
    }
}

impl WavAudio {
    pub fn new(data: Vec<u8>, metadata: AudioMetadata) -> Result<Self> {
        if !Self::verify(&data) {
            return Err(Error::InvalidContent("Not a valid WAV file".into()));
        }
        Ok(Self { data, metadata })
    }
    
    pub fn verify(data: &[u8]) -> bool {
        data.starts_with(magic::WAV) && data.len() > 12 && &data[8..12] == b"WAVE"
    }
}

impl FlacAudio {
    pub fn new(data: Vec<u8>, metadata: AudioMetadata) -> Result<Self> {
        if !Self::verify(&data) {
            return Err(Error::InvalidContent("Not a valid FLAC file".into()));
        }
        Ok(Self { data, metadata })
    }
    
    pub fn verify(data: &[u8]) -> bool {
        data.starts_with(magic::FLAC)
    }
}

impl AacAudio {
    pub fn new(data: Vec<u8>, metadata: AudioMetadata) -> Result<Self> {
        if !Self::verify(&data) {
            return Err(Error::InvalidContent("Not a valid AAC file".into()));
        }
        Ok(Self { data, metadata })
    }
    
    pub fn verify(data: &[u8]) -> bool {
        data.starts_with(magic::AAC_ADTS)
    }
}

impl OggAudio {
    pub fn new(data: Vec<u8>, metadata: AudioMetadata) -> Result<Self> {
        if !Self::verify(&data) {
            return Err(Error::InvalidContent("Not a valid OGG file".into()));
        }
        Ok(Self { data, metadata })
    }
    
    pub fn verify(data: &[u8]) -> bool {
        data.starts_with(magic::OGG)
    }
}

// Video verification

impl Mp4Video {
    pub fn new(data: Vec<u8>, metadata: VideoMetadata) -> Result<Self> {
        if !Self::verify(&data) {
            return Err(Error::InvalidContent("Not a valid MP4 file".into()));
        }
        Ok(Self { data, metadata })
    }
    
    pub fn verify(data: &[u8]) -> bool {
        data.len() >= 8 && &data[4..8] == b"ftyp"
    }
}

impl MovVideo {
    pub fn new(data: Vec<u8>, metadata: VideoMetadata) -> Result<Self> {
        if !Self::verify(&data) {
            return Err(Error::InvalidContent("Not a valid MOV file".into()));
        }
        Ok(Self { data, metadata })
    }
    
    pub fn verify(data: &[u8]) -> bool {
        data.len() >= 12 && &data[4..8] == b"ftyp" && &data[8..12] == b"qt  "
    }
}

impl MkvVideo {
    pub fn new(data: Vec<u8>, metadata: VideoMetadata) -> Result<Self> {
        if !Self::verify(&data) {
            return Err(Error::InvalidContent("Not a valid MKV file".into()));
        }
        Ok(Self { data, metadata })
    }
    
    pub fn verify(data: &[u8]) -> bool {
        data.starts_with(magic::MKV)
    }
}

impl AviVideo {
    pub fn new(data: Vec<u8>, metadata: VideoMetadata) -> Result<Self> {
        if !Self::verify(&data) {
            return Err(Error::InvalidContent("Not a valid AVI file".into()));
        }
        Ok(Self { data, metadata })
    }
    
    pub fn verify(data: &[u8]) -> bool {
        data.starts_with(magic::AVI) && data.len() > 12 && &data[8..12] == b"AVI "
    }
}

/// Helper to detect content type from data
pub fn detect_content_type(data: &[u8]) -> Option<ContentType> {
    // Documents
    if data.starts_with(magic::PDF) {
        return Some(ContentType::Custom(codec::PDF));
    }
    if data.starts_with(magic::DOCX) {
        return Some(ContentType::Custom(codec::DOCX));
    }
    
    // Images
    if data.starts_with(magic::PNG) {
        return Some(ContentType::Custom(codec::PNG));
    }
    if data.starts_with(magic::JPEG) {
        return Some(ContentType::Custom(codec::JPEG));
    }
    if data.starts_with(magic::GIF) {
        return Some(ContentType::Custom(codec::GIF));
    }
    if data.starts_with(magic::WEBP) && data.len() > 12 && &data[8..12] == b"WEBP" {
        return Some(ContentType::Custom(codec::WEBP));
    }
    
    // Audio
    if data.starts_with(magic::MP3_ID3) || data.starts_with(magic::MP3_SYNC) {
        return Some(ContentType::Custom(codec::MP3));
    }
    if data.starts_with(magic::WAV) && data.len() > 12 && &data[8..12] == b"WAVE" {
        return Some(ContentType::Custom(codec::WAV));
    }
    if data.starts_with(magic::FLAC) {
        return Some(ContentType::Custom(codec::FLAC));
    }
    if data.starts_with(magic::AAC_ADTS) {
        return Some(ContentType::Custom(codec::AAC));
    }
    if data.starts_with(magic::OGG) {
        return Some(ContentType::Custom(codec::OGG));
    }
    
    // Video
    if data.len() > 12 && &data[4..8] == b"ftyp" {
        if &data[8..12] == b"qt  " {
            return Some(ContentType::Custom(codec::MOV));
        } else {
            return Some(ContentType::Custom(codec::MP4));
        }
    }
    if data.starts_with(magic::MKV) {
        return Some(ContentType::Custom(codec::MKV));
    }
    if data.starts_with(magic::AVI) && data.len() > 12 && &data[8..12] == b"AVI " {
        return Some(ContentType::Custom(codec::AVI));
    }
    
    None
}

/// Get human-readable name for content type
pub fn content_type_name(content_type: ContentType) -> &'static str {
    match content_type {
        ContentType::Custom(codec::PDF) => "PDF Document",
        ContentType::Custom(codec::DOCX) => "DOCX Document",
        ContentType::Custom(codec::MARKDOWN) => "Markdown Document",
        ContentType::Custom(codec::TEXT) => "Text Document",
        ContentType::Custom(codec::JPEG) => "JPEG Image",
        ContentType::Custom(codec::PNG) => "PNG Image",
        ContentType::Custom(codec::GIF) => "GIF Image",
        ContentType::Custom(codec::WEBP) => "WebP Image",
        ContentType::Custom(codec::MP3) => "MP3 Audio",
        ContentType::Custom(codec::WAV) => "WAV Audio",
        ContentType::Custom(codec::FLAC) => "FLAC Audio",
        ContentType::Custom(codec::AAC) => "AAC Audio",
        ContentType::Custom(codec::OGG) => "OGG Audio",
        ContentType::Custom(codec::MP4) => "MP4 Video",
        ContentType::Custom(codec::MOV) => "MOV Video",
        ContentType::Custom(codec::MKV) => "MKV Video",
        ContentType::Custom(codec::AVI) => "AVI Video",
        _ => "Unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdf_verification() {
        let valid_pdf = b"%PDF-1.4\n%\xE2\xE3\xCF\xD3";
        assert!(PdfDocument::verify(valid_pdf));
        
        let invalid_pdf = b"Not a PDF";
        assert!(!PdfDocument::verify(invalid_pdf));
    }

    #[test]
    fn test_image_detection() {
        let png_data = b"\x89PNG\r\n\x1a\nsome data";
        let detected = detect_content_type(png_data);
        assert_eq!(detected, Some(ContentType::Custom(codec::PNG)));
        
        let jpeg_data = b"\xFF\xD8\xFF\xE0some jpeg data";
        let detected = detect_content_type(jpeg_data);
        assert_eq!(detected, Some(ContentType::Custom(codec::JPEG)));
    }

    #[test]
    fn test_content_type_names() {
        assert_eq!(content_type_name(ContentType::Custom(codec::PDF)), "PDF Document");
        assert_eq!(content_type_name(ContentType::Custom(codec::MP3)), "MP3 Audio");
        assert_eq!(content_type_name(ContentType::Custom(codec::MP4)), "MP4 Video");
    }

    #[test]
    fn test_docx_verification() {
        // DOCX files start with PK (ZIP signature) followed by specific patterns
        let valid_docx = b"PK\x03\x04\x14\x00\x00\x00";
        assert!(DocxDocument::verify(valid_docx));
        
        let invalid_docx = b"Not a DOCX";
        assert!(!DocxDocument::verify(invalid_docx));
    }

    #[test]
    fn test_markdown_document() {
        let doc = MarkdownDocument {
            content: "# Hello\nThis is markdown".to_string(),
            metadata: DocumentMetadata::default(),
        };
        assert_eq!(doc.content, "# Hello\nThis is markdown");
    }

    #[test]
    fn test_text_document() {
        let doc = TextDocument {
            content: "Plain text content".to_string(),
            metadata: DocumentMetadata::default(),
        };
        assert_eq!(doc.content, "Plain text content");
    }

    #[test]
    fn test_jpeg_verification() {
        let valid_jpeg = b"\xFF\xD8\xFF\xE0\x00\x10JFIF";
        assert!(JpegImage::verify(valid_jpeg));
        
        let valid_jpeg2 = b"\xFF\xD8\xFF\xE1\x00\x10Exif";
        assert!(JpegImage::verify(valid_jpeg2));
        
        let invalid_jpeg = b"Not a JPEG";
        assert!(!JpegImage::verify(invalid_jpeg));
    }

    #[test]
    fn test_png_verification() {
        let valid_png = b"\x89PNG\r\n\x1a\n";
        assert!(PngImage::verify(valid_png));
        
        let invalid_png = b"Not a PNG";
        assert!(!PngImage::verify(invalid_png));
    }

    #[test]
    fn test_gif_verification() {
        let valid_gif87 = b"GIF87a";
        assert!(GifImage::verify(valid_gif87));
        
        let valid_gif89 = b"GIF89a";
        assert!(GifImage::verify(valid_gif89));
        
        let invalid_gif = b"Not a GIF";
        assert!(!GifImage::verify(invalid_gif));
    }

    #[test]
    fn test_webp_verification() {
        // WebP has RIFF header with WEBP at bytes 8-12
        let mut valid_webp = b"RIFF\x00\x00\x00\x00WEBP".to_vec();
        valid_webp.push(0); // Make it at least 13 bytes
        assert!(WebPImage::verify(&valid_webp));
        
        // Invalid: too short
        let short_webp = b"RIFF";
        assert!(!WebPImage::verify(short_webp));
        
        // Invalid: wrong format at bytes 8-12
        let invalid_webp = b"RIFF\x00\x00\x00\x00WAVE";
        assert!(!WebPImage::verify(invalid_webp));
    }

    #[test]
    fn test_mp3_verification() {
        // MP3 with ID3v2 tag
        let valid_mp3_id3 = b"ID3\x03\x00\x00\x00";
        assert!(Mp3Audio::verify(valid_mp3_id3));
        
        // MP3 with sync frame
        let valid_mp3_sync = b"\xFF\xFB\x90\x00";
        assert!(Mp3Audio::verify(valid_mp3_sync));
        
        let invalid_mp3 = b"Not an MP3";
        assert!(!Mp3Audio::verify(invalid_mp3));
    }

    #[test]
    fn test_wav_verification() {
        let valid_wav = b"RIFF\x00\x00\x00\x00WAVEfmt ";
        assert!(WavAudio::verify(valid_wav));
        
        let invalid_wav = b"Not a WAV";
        assert!(!WavAudio::verify(invalid_wav));
    }

    #[test]
    fn test_flac_verification() {
        let valid_flac = b"fLaC\x00\x00\x00\x00";
        assert!(FlacAudio::verify(valid_flac));
        
        let invalid_flac = b"Not a FLAC";
        assert!(!FlacAudio::verify(invalid_flac));
    }

    #[test]
    fn test_aac_verification() {
        // AAC with ADTS header
        let valid_aac = b"\xFF\xF1\x00\x00";
        assert!(AacAudio::verify(valid_aac));
        
        let invalid_aac = b"Not an AAC";
        assert!(!AacAudio::verify(invalid_aac));
    }

    #[test]
    fn test_ogg_verification() {
        let valid_ogg = b"OggS\x00\x02\x00\x00";
        assert!(OggAudio::verify(valid_ogg));
        
        let invalid_ogg = b"Not an OGG";
        assert!(!OggAudio::verify(invalid_ogg));
    }

    #[test]
    fn test_mp4_verification() {
        // MP4 with ftyp box
        let valid_mp4 = b"\x00\x00\x00\x20ftypmp42";
        assert!(Mp4Video::verify(valid_mp4));
        
        let invalid_mp4 = b"Not an MP4";
        assert!(!Mp4Video::verify(invalid_mp4));
    }

    #[test]
    fn test_mov_verification() {
        // MOV with ftyp box and qt brand
        let valid_mov = b"\x00\x00\x00\x14ftypqt  ";
        assert!(MovVideo::verify(valid_mov));
        
        let invalid_mov = b"Not a MOV";
        assert!(!MovVideo::verify(invalid_mov));
    }

    #[test]
    fn test_mkv_verification() {
        // MKV with EBML header
        let valid_mkv = b"\x1A\x45\xDF\xA3";
        assert!(MkvVideo::verify(valid_mkv));
        
        let invalid_mkv = b"Not an MKV";
        assert!(!MkvVideo::verify(invalid_mkv));
    }

    #[test]
    fn test_avi_verification() {
        // AVI with RIFF header
        let valid_avi = b"RIFF\x00\x00\x00\x00AVI LIST";
        assert!(AviVideo::verify(valid_avi));
        
        let invalid_avi = b"Not an AVI";
        assert!(!AviVideo::verify(invalid_avi));
    }

    #[test]
    fn test_detect_content_type_comprehensive() {
        // Test all supported formats
        assert_eq!(detect_content_type(b"%PDF-"), Some(ContentType::Custom(codec::PDF)));
        assert_eq!(detect_content_type(b"PK\x03\x04"), Some(ContentType::Custom(codec::DOCX)));
        assert_eq!(detect_content_type(b"\xFF\xD8\xFF"), Some(ContentType::Custom(codec::JPEG)));
        assert_eq!(detect_content_type(b"\x89PNG\r\n\x1a\n"), Some(ContentType::Custom(codec::PNG)));
        assert_eq!(detect_content_type(b"GIF87a"), Some(ContentType::Custom(codec::GIF)));
        assert_eq!(detect_content_type(b"RIFF\x00\x00\x00\x00WEBP\x00"), Some(ContentType::Custom(codec::WEBP)));
        assert_eq!(detect_content_type(b"ID3"), Some(ContentType::Custom(codec::MP3)));
        assert_eq!(detect_content_type(b"\xFF\xFB"), Some(ContentType::Custom(codec::MP3)));
        assert_eq!(detect_content_type(b"RIFF\x00\x00\x00\x00WAVEfmt "), Some(ContentType::Custom(codec::WAV)));
        assert_eq!(detect_content_type(b"fLaC"), Some(ContentType::Custom(codec::FLAC)));
        assert_eq!(detect_content_type(b"\xFF\xF1"), Some(ContentType::Custom(codec::AAC)));
        assert_eq!(detect_content_type(b"OggS"), Some(ContentType::Custom(codec::OGG)));
        assert_eq!(detect_content_type(b"\x00\x00\x00\x20ftypmp42\x00\x00\x00\x00"), Some(ContentType::Custom(codec::MP4)));
        assert_eq!(detect_content_type(b"\x00\x00\x00\x14ftypqt  \x00\x00\x00\x00"), Some(ContentType::Custom(codec::MOV)));
        assert_eq!(detect_content_type(b"\x1A\x45\xDF\xA3"), Some(ContentType::Custom(codec::MKV)));
        assert_eq!(detect_content_type(b"RIFF\x00\x00\x00\x00AVI LIST"), Some(ContentType::Custom(codec::AVI)));
        
        // Test unknown format
        assert_eq!(detect_content_type(b"Unknown format"), None);
    }

    #[test]
    fn test_content_type_names_comprehensive() {
        // Test all content type names
        assert_eq!(content_type_name(ContentType::Custom(codec::PDF)), "PDF Document");
        assert_eq!(content_type_name(ContentType::Custom(codec::DOCX)), "DOCX Document");
        assert_eq!(content_type_name(ContentType::Custom(codec::MARKDOWN)), "Markdown Document");
        assert_eq!(content_type_name(ContentType::Custom(codec::TEXT)), "Text Document");
        assert_eq!(content_type_name(ContentType::Custom(codec::JPEG)), "JPEG Image");
        assert_eq!(content_type_name(ContentType::Custom(codec::PNG)), "PNG Image");
        assert_eq!(content_type_name(ContentType::Custom(codec::GIF)), "GIF Image");
        assert_eq!(content_type_name(ContentType::Custom(codec::WEBP)), "WebP Image");
        assert_eq!(content_type_name(ContentType::Custom(codec::MP3)), "MP3 Audio");
        assert_eq!(content_type_name(ContentType::Custom(codec::WAV)), "WAV Audio");
        assert_eq!(content_type_name(ContentType::Custom(codec::FLAC)), "FLAC Audio");
        assert_eq!(content_type_name(ContentType::Custom(codec::AAC)), "AAC Audio");
        assert_eq!(content_type_name(ContentType::Custom(codec::OGG)), "OGG Audio");
        assert_eq!(content_type_name(ContentType::Custom(codec::MP4)), "MP4 Video");
        assert_eq!(content_type_name(ContentType::Custom(codec::MOV)), "MOV Video");
        assert_eq!(content_type_name(ContentType::Custom(codec::MKV)), "MKV Video");
        assert_eq!(content_type_name(ContentType::Custom(codec::AVI)), "AVI Video");
        assert_eq!(content_type_name(ContentType::Custom(0x999999)), "Unknown");
    }

    #[test]
    fn test_webp_edge_cases() {
        // Test WEBP with exact 12 bytes (minimum required)
        let short_webp = b"RIFF\x00\x00\x00\x00WEBP";
        assert!(!WebPImage::verify(short_webp)); // Should fail - needs > 12 bytes
        
        // Test WEBP with 13 bytes (just enough)
        let min_webp = b"RIFF\x00\x00\x00\x00WEBP\x00";
        assert!(WebPImage::verify(min_webp));
        
        // Test WEBP with wrong RIFF header
        let wrong_riff = b"RIFX\x00\x00\x00\x00WEBP\x00";
        assert!(!WebPImage::verify(wrong_riff));
        
        // Test empty data
        let empty = b"";
        assert!(!WebPImage::verify(empty));
    }

    #[test]
    fn test_wav_edge_cases() {
        // Test WAV with exact 12 bytes (minimum required)
        let short_wav = b"RIFF\x00\x00\x00\x00WAVE";
        assert!(!WavAudio::verify(short_wav)); // Should fail - needs > 12 bytes
        
        // Test WAV with 13 bytes (just enough)
        let min_wav = b"RIFF\x00\x00\x00\x00WAVE\x00";
        assert!(WavAudio::verify(min_wav));
        
        // Test WAV with wrong format chunk
        let wrong_wave = b"RIFF\x00\x00\x00\x00WAXE\x00";
        assert!(!WavAudio::verify(wrong_wave));
    }

    #[test]
    fn test_avi_edge_cases() {
        // Test AVI with exact 12 bytes (minimum required)
        let short_avi = b"RIFF\x00\x00\x00\x00AVI ";
        assert!(!AviVideo::verify(short_avi)); // Should fail - needs > 12 bytes
        
        // Test AVI with 13 bytes (just enough)
        let min_avi = b"RIFF\x00\x00\x00\x00AVI \x00";
        assert!(AviVideo::verify(min_avi));
        
        // Test AVI with wrong AVI chunk
        let wrong_avi = b"RIFF\x00\x00\x00\x00AVIX\x00";
        assert!(!AviVideo::verify(wrong_avi));
    }

    #[test]
    fn test_mov_edge_cases() {
        // Test MOV with less than 12 bytes
        let short_mov = b"\x00\x00\x00\x08ftypqt";
        assert!(!MovVideo::verify(short_mov));
        
        // Test MOV with exactly 12 bytes
        let exact_mov = b"\x00\x00\x00\x0Cftypqt  ";
        assert!(MovVideo::verify(exact_mov));
        
        // Test MOV with wrong brand
        let wrong_brand = b"\x00\x00\x00\x0Cftypmp42";
        assert!(!MovVideo::verify(wrong_brand)); // This is MP4, not MOV
    }

    #[test]
    fn test_mp4_edge_cases() {
        // Test MP4 with less than 8 bytes
        let short_mp4 = b"\x00\x00\x00\x04fty";
        assert!(!Mp4Video::verify(short_mp4));
        
        // Test MP4 with exactly 8 bytes
        let exact_mp4 = b"\x00\x00\x00\x08ftyp";
        assert!(Mp4Video::verify(exact_mp4));
        
        // Test MP4 with wrong box type
        let wrong_box = b"\x00\x00\x00\x08mdat";
        assert!(!Mp4Video::verify(wrong_box));
    }

    #[test]
    fn test_detect_content_type_edge_cases() {
        // Test empty data
        assert_eq!(detect_content_type(b""), None);
        
        // Test single byte
        assert_eq!(detect_content_type(b"X"), None);
        
        // Test data that's too short for format detection
        assert_eq!(detect_content_type(b"RI"), None);
        assert_eq!(detect_content_type(b"RIFF"), None); // Too short for WEBP/WAV/AVI
        
        // Test MP4/MOV differentiation with minimal data
        let mp4_min = b"\x00\x00\x00\x08ftypmp42\x00"; // Need > 12 bytes
        assert_eq!(detect_content_type(mp4_min), Some(ContentType::Custom(codec::MP4)));
        
        let mov_min = b"\x00\x00\x00\x0Cftypqt  \x00"; // Need > 12 bytes
        assert_eq!(detect_content_type(mov_min), Some(ContentType::Custom(codec::MOV)));
        
        // Test data with MP4/MOV signature but too short
        let short_ftyp = b"\x00\x00\x00\x08ftyp";
        assert_eq!(detect_content_type(short_ftyp), None); // Too short, needs > 12 bytes
    }

    #[test]
    fn test_content_new_error_handling() {
        // Test invalid PDF
        let result = PdfDocument::new(b"Not PDF".to_vec(), DocumentMetadata::default());
        assert!(result.is_err());
        match result {
            Err(Error::InvalidContent(msg)) => assert!(msg.contains("Not a valid PDF")),
            _ => panic!("Expected InvalidContent error"),
        }
        
        // Test invalid JPEG
        let result = JpegImage::new(b"Not JPEG".to_vec(), ImageMetadata::default());
        assert!(result.is_err());
        match result {
            Err(Error::InvalidContent(msg)) => assert!(msg.contains("Not a valid JPEG")),
            _ => panic!("Expected InvalidContent error"),
        }
        
        // Test invalid MP3
        let result = Mp3Audio::new(b"Not MP3".to_vec(), AudioMetadata::default());
        assert!(result.is_err());
        match result {
            Err(Error::InvalidContent(msg)) => assert!(msg.contains("Not a valid MP3")),
            _ => panic!("Expected InvalidContent error"),
        }
        
        // Test invalid MP4
        let result = Mp4Video::new(b"Not MP4".to_vec(), VideoMetadata::default());
        assert!(result.is_err());
        match result {
            Err(Error::InvalidContent(msg)) => assert!(msg.contains("Not a valid MP4")),
            _ => panic!("Expected InvalidContent error"),
        }
    }

    #[test]
    fn test_jpeg_edge_cases() {
        // Test JPEG with FFD8 but wrong next bytes
        let bad_jpeg = b"\xFF\xD8\x00\x00";
        assert!(!JpegImage::verify(bad_jpeg));
        
        // Test very short JPEG header
        let short_jpeg = b"\xFF\xD8";
        assert!(!JpegImage::verify(short_jpeg));
        
        // Test JPEG with JFIF marker
        let jfif = b"\xFF\xD8\xFF\xE0";
        assert!(JpegImage::verify(jfif));
        
        // Test JPEG with EXIF marker
        let exif = b"\xFF\xD8\xFF\xE1";
        assert!(JpegImage::verify(exif));
        
        // Test JPEG with other valid markers
        let other = b"\xFF\xD8\xFF\xDB"; // DQT marker
        assert!(JpegImage::verify(other));
    }

    #[test]
    fn test_mp3_edge_cases() {
        // Test MP3 with incomplete ID3 header
        let short_id3 = b"ID";
        assert!(!Mp3Audio::verify(short_id3));
        
        // Test MP3 with incomplete sync header
        let short_sync = b"\xFF";
        assert!(!Mp3Audio::verify(short_sync));
        
        // Test MP3 with sync pattern - only 0xFF 0xFB is checked
        let sync1 = b"\xFF\xFA"; // Not 0xFF 0xFB
        assert!(!Mp3Audio::verify(sync1));
        
        let sync2 = b"\xFF\xF3"; // Not 0xFF 0xFB
        assert!(!Mp3Audio::verify(sync2));
        
        let valid_sync = b"\xFF\xFB"; // Valid MP3 sync
        assert!(Mp3Audio::verify(valid_sync));
        
        // Test MP3 with invalid sync (FF but wrong second byte)
        let bad_sync = b"\xFF\x00";
        assert!(!Mp3Audio::verify(bad_sync));
    }

    #[test]
    fn test_aac_edge_cases() {
        // Test AAC with incomplete ADTS header
        let short_aac = b"\xFF";
        assert!(!AacAudio::verify(short_aac));
        
        // Test AAC with valid ADTS sync
        let valid_aac1 = b"\xFF\xF1"; // No CRC
        assert!(AacAudio::verify(valid_aac1));
        
        // AAC only checks for 0xFF 0xF1 pattern
        let valid_aac2 = b"\xFF\xF9"; // This is not 0xFF 0xF1
        assert!(!AacAudio::verify(valid_aac2));
        
        // Test AAC with invalid sync pattern
        let bad_aac = b"\xFF\xF0";
        assert!(!AacAudio::verify(bad_aac));
    }

    #[test]
    fn test_ogg_edge_cases() {
        // Test OGG with incomplete header
        let short_ogg = b"Og";
        assert!(!OggAudio::verify(short_ogg));
        
        let incomplete_ogg = b"Ogg";
        assert!(!OggAudio::verify(incomplete_ogg));
        
        // Test OGG with wrong magic
        let bad_ogg = b"OgXS";
        assert!(!OggAudio::verify(bad_ogg));
    }

    #[test]
    fn test_gif_edge_cases() {
        // Test GIF with incomplete header
        let short_gif = b"GIF";
        assert!(!GifImage::verify(short_gif));
        
        // GIF8 is a valid prefix for both GIF87a and GIF89a
        let incomplete_gif = b"GIF8";
        assert!(GifImage::verify(incomplete_gif));
        
        // Test GIF with wrong version
        let bad_gif = b"GIF90a";
        assert!(!GifImage::verify(bad_gif));
    }

    #[test]
    fn test_flac_edge_cases() {
        // Test FLAC with incomplete header
        let short_flac = b"fL";
        assert!(!FlacAudio::verify(short_flac));
        
        let incomplete_flac = b"fLa";
        assert!(!FlacAudio::verify(incomplete_flac));
        
        // Test FLAC with wrong magic
        let bad_flac = b"fLaX";
        assert!(!FlacAudio::verify(bad_flac));
    }

    #[test]
    fn test_mkv_edge_cases() {
        // Test MKV with incomplete EBML header
        let short_mkv = b"\x1A";
        assert!(!MkvVideo::verify(short_mkv));
        
        let incomplete_mkv = b"\x1A\x45";
        assert!(!MkvVideo::verify(incomplete_mkv));
        
        // Test MKV with wrong EBML magic
        let bad_mkv = b"\x1A\x45\xDF\xA4";
        assert!(!MkvVideo::verify(bad_mkv));
    }

    #[test]
    fn test_png_edge_cases() {
        // Test PNG with incomplete header
        let short_png = b"\x89PN";
        assert!(!PngImage::verify(short_png));
        
        // Test PNG with wrong magic bytes
        let bad_png = b"\x89PNG\r\n\x1b\n"; // Wrong byte at position 6
        assert!(!PngImage::verify(bad_png));
    }

    #[test]
    fn test_docx_edge_cases() {
        // Test DOCX with incomplete PK header
        let short_docx = b"PK";
        assert!(!DocxDocument::verify(short_docx));
        
        // Test DOCX with wrong PK signature
        let bad_docx = b"PK\x05\x06"; // This is End of Central Directory
        assert!(!DocxDocument::verify(bad_docx));
    }

    #[test]
    fn test_pdf_edge_cases() {
        // Test PDF with incomplete header
        let short_pdf = b"%PD";
        assert!(!PdfDocument::verify(short_pdf));
        
        // Test PDF with wrong header
        let bad_pdf = b"%PDX-";
        assert!(!PdfDocument::verify(bad_pdf));
    }

    #[test]
    fn test_boundary_conditions() {
        // Test formats that check specific byte positions
        
        // WEBP at exactly boundary
        let webp_boundary = b"RIFF\x00\x00\x00\x00WEBP"; // Exactly 12 bytes
        assert_eq!(detect_content_type(webp_boundary), None); // Should fail in detect_content_type
        
        // WAV at exactly boundary  
        let wav_boundary = b"RIFF\x00\x00\x00\x00WAVE"; // Exactly 12 bytes
        assert_eq!(detect_content_type(wav_boundary), None); // Should fail in detect_content_type
        
        // AVI at exactly boundary
        let avi_boundary = b"RIFF\x00\x00\x00\x00AVI "; // Exactly 12 bytes
        assert_eq!(detect_content_type(avi_boundary), None); // Should fail in detect_content_type
        
        // MOV at exactly boundary
        let mov_boundary = b"\x00\x00\x00\x08ftypqt"; // Less than 12 bytes
        assert_eq!(detect_content_type(mov_boundary), None); // Should fail
    }

    #[test]
    fn test_all_verify_methods_with_empty_data() {
        // Ensure all verify methods handle empty data gracefully
        assert!(!PdfDocument::verify(b""));
        assert!(!DocxDocument::verify(b""));
        assert!(!JpegImage::verify(b""));
        assert!(!PngImage::verify(b""));
        assert!(!GifImage::verify(b""));
        assert!(!WebPImage::verify(b""));
        assert!(!Mp3Audio::verify(b""));
        assert!(!WavAudio::verify(b""));
        assert!(!FlacAudio::verify(b""));
        assert!(!AacAudio::verify(b""));
        assert!(!OggAudio::verify(b""));
        assert!(!Mp4Video::verify(b""));
        assert!(!MovVideo::verify(b""));
        assert!(!MkvVideo::verify(b""));
        assert!(!AviVideo::verify(b""));
    }

    #[test]
    fn test_content_type_core_names() {
        // Test core content type names - these return "Unknown" since they're not in the match
        assert_eq!(content_type_name(ContentType::Event), "Unknown");
        assert_eq!(content_type_name(ContentType::Graph), "Unknown");
        assert_eq!(content_type_name(ContentType::Node), "Unknown");
        assert_eq!(content_type_name(ContentType::Edge), "Unknown");
        assert_eq!(content_type_name(ContentType::Command), "Unknown");
        assert_eq!(content_type_name(ContentType::Query), "Unknown");
        assert_eq!(content_type_name(ContentType::Markdown), "Unknown");
        assert_eq!(content_type_name(ContentType::Json), "Unknown");
        assert_eq!(content_type_name(ContentType::Yaml), "Unknown");
        assert_eq!(content_type_name(ContentType::Toml), "Unknown");
        assert_eq!(content_type_name(ContentType::Image), "Unknown");
        assert_eq!(content_type_name(ContentType::Video), "Unknown");
        assert_eq!(content_type_name(ContentType::Audio), "Unknown");
    }

    #[test]
    fn test_markdown_from_bytes_error() {
        // Test invalid UTF-8 in markdown
        let invalid_utf8 = vec![0xFF, 0xFE, 0xFD]; // Invalid UTF-8 sequence
        let result = MarkdownDocument::from_bytes(invalid_utf8, DocumentMetadata::default());
        assert!(result.is_err());
        match result {
            Err(Error::InvalidContent(msg)) => assert!(msg.contains("Invalid UTF-8 in markdown")),
            _ => panic!("Expected InvalidContent error"),
        }
    }

    #[test]
    fn test_text_from_bytes_error() {
        // Test invalid UTF-8 in text
        let invalid_utf8 = vec![0xC0, 0x80]; // Overlong encoding (invalid UTF-8)
        let result = TextDocument::from_bytes(invalid_utf8, DocumentMetadata::default());
        assert!(result.is_err());
        match result {
            Err(Error::InvalidContent(msg)) => assert!(msg.contains("Invalid UTF-8 in text")),
            _ => panic!("Expected InvalidContent error"),
        }
    }

    #[test]
    fn test_valid_utf8_conversion() {
        // Test valid UTF-8 conversion for markdown
        let valid_utf8 = "Hello 世界! 🌍".as_bytes().to_vec();
        let result = MarkdownDocument::from_bytes(valid_utf8.clone(), DocumentMetadata::default());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().content, "Hello 世界! 🌍");
        
        // Test valid UTF-8 conversion for text
        let result = TextDocument::from_bytes(valid_utf8, DocumentMetadata::default());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().content, "Hello 世界! 🌍");
    }

    #[test]
    fn test_metadata_fields() {
        // Test DocumentMetadata with all fields
        let metadata = DocumentMetadata {
            title: Some("Test Document".to_string()),
            author: Some("Test Author".to_string()),
            created_at: Some(1234567890),
            modified_at: Some(1234567900),
            tags: vec!["tag1".to_string(), "tag2".to_string()],
            language: Some("en".to_string()),
        };
        
        assert_eq!(metadata.title.unwrap(), "Test Document");
        assert_eq!(metadata.author.unwrap(), "Test Author");
        assert_eq!(metadata.created_at.unwrap(), 1234567890);
        assert_eq!(metadata.modified_at.unwrap(), 1234567900);
        assert_eq!(metadata.tags.len(), 2);
        assert_eq!(metadata.language.unwrap(), "en");
        
        // Test ImageMetadata with all fields
        let img_metadata = ImageMetadata {
            width: Some(1920),
            height: Some(1080),
            format: Some("JPEG".to_string()),
            color_space: Some("sRGB".to_string()),
            compression: Some("lossy".to_string()),
            tags: vec!["photo".to_string()],
        };
        
        assert_eq!(img_metadata.width.unwrap(), 1920);
        assert_eq!(img_metadata.height.unwrap(), 1080);
        
        // Test AudioMetadata with all fields
        let audio_metadata = AudioMetadata {
            duration_ms: Some(180000),
            bitrate: Some(320000),
            sample_rate: Some(44100),
            channels: Some(2),
            codec: Some("MP3".to_string()),
            artist: Some("Test Artist".to_string()),
            album: Some("Test Album".to_string()),
            title: Some("Test Song".to_string()),
            year: Some(2024),
            tags: vec!["music".to_string()],
        };
        
        assert_eq!(audio_metadata.duration_ms.unwrap(), 180000);
        assert_eq!(audio_metadata.channels.unwrap(), 2);
        
        // Test VideoMetadata with all fields
        let video_metadata = VideoMetadata {
            duration_ms: Some(120000),
            width: Some(1920),
            height: Some(1080),
            frame_rate: Some(30.0),
            video_codec: Some("H.264".to_string()),
            audio_codec: Some("AAC".to_string()),
            bitrate: Some(5000000),
            tags: vec!["movie".to_string()],
        };
        
        assert_eq!(video_metadata.frame_rate.unwrap(), 30.0);
    }

    #[test]
    fn test_all_constructors_with_valid_data() {
        // Test all valid constructors
        let valid_pdf = b"%PDF-1.4\ntest".to_vec();
        let pdf = PdfDocument::new(valid_pdf, DocumentMetadata::default());
        assert!(pdf.is_ok());
        
        let valid_docx = b"PK\x03\x04\x14\x00".to_vec();
        let docx = DocxDocument::new(valid_docx, DocumentMetadata::default());
        assert!(docx.is_ok());
        
        let markdown = MarkdownDocument::new("# Test".to_string(), DocumentMetadata::default());
        assert!(markdown.is_ok());
        
        let text = TextDocument::new("Test".to_string(), DocumentMetadata::default());
        assert!(text.is_ok());
        
        let valid_jpeg = b"\xFF\xD8\xFF\xE0test".to_vec();
        let jpeg = JpegImage::new(valid_jpeg, ImageMetadata::default());
        assert!(jpeg.is_ok());
        
        let valid_png = b"\x89PNG\r\n\x1a\ntest".to_vec();
        let png = PngImage::new(valid_png, ImageMetadata::default());
        assert!(png.is_ok());
        
        let valid_gif = b"GIF89atest".to_vec();
        let gif = GifImage::new(valid_gif, ImageMetadata::default());
        assert!(gif.is_ok());
        
        let valid_webp = b"RIFF\x00\x00\x00\x00WEBPtest".to_vec();
        let webp = WebPImage::new(valid_webp, ImageMetadata::default());
        assert!(webp.is_ok());
        
        let valid_mp3 = b"ID3\x03\x00test".to_vec();
        let mp3 = Mp3Audio::new(valid_mp3, AudioMetadata::default());
        assert!(mp3.is_ok());
        
        let valid_wav = b"RIFF\x00\x00\x00\x00WAVEtest".to_vec();
        let wav = WavAudio::new(valid_wav, AudioMetadata::default());
        assert!(wav.is_ok());
        
        let valid_flac = b"fLaCtest".to_vec();
        let flac = FlacAudio::new(valid_flac, AudioMetadata::default());
        assert!(flac.is_ok());
        
        let valid_aac = b"\xFF\xF1test".to_vec();
        let aac = AacAudio::new(valid_aac, AudioMetadata::default());
        assert!(aac.is_ok());
        
        let valid_ogg = b"OggStest".to_vec();
        let ogg = OggAudio::new(valid_ogg, AudioMetadata::default());
        assert!(ogg.is_ok());
        
        let valid_mp4 = b"\x00\x00\x00\x08ftyptest".to_vec();
        let mp4 = Mp4Video::new(valid_mp4, VideoMetadata::default());
        assert!(mp4.is_ok());
        
        let valid_mov = b"\x00\x00\x00\x0Cftypqt  test".to_vec();
        let mov = MovVideo::new(valid_mov, VideoMetadata::default());
        assert!(mov.is_ok());
        
        let valid_mkv = b"\x1A\x45\xDF\xA3test".to_vec();
        let mkv = MkvVideo::new(valid_mkv, VideoMetadata::default());
        assert!(mkv.is_ok());
        
        let valid_avi = b"RIFF\x00\x00\x00\x00AVI test".to_vec();
        let avi = AviVideo::new(valid_avi, VideoMetadata::default());
        assert!(avi.is_ok());
    }

    #[test]
    fn test_detect_content_type_partial_matches() {
        // Test partial matches that shouldn't be detected
        let partial_pdf = b"%PD"; // Missing "F-"
        assert_eq!(detect_content_type(partial_pdf), None);
        
        let partial_jpeg = b"\xFF\xD8"; // Missing third byte
        assert_eq!(detect_content_type(partial_jpeg), None);
        
        let partial_png = b"\x89PN"; // Missing rest of signature
        assert_eq!(detect_content_type(partial_png), None);
        
        let partial_mp3 = b"ID"; // Incomplete ID3
        assert_eq!(detect_content_type(partial_mp3), None);
        
        let partial_flac = b"fL"; // Incomplete fLaC
        assert_eq!(detect_content_type(partial_flac), None);
    }

    #[test]
    fn test_enum_variants() {
        // Test ImageContent enum
        let jpeg_img = JpegImage {
            data: vec![1, 2, 3],
            metadata: ImageMetadata::default(),
        };
        let img_content = ImageContent::Jpeg(jpeg_img.clone());
        match img_content {
            ImageContent::Jpeg(j) => assert_eq!(j.data, jpeg_img.data),
            _ => panic!("Wrong variant"),
        }
        
        // Test AudioContent enum
        let mp3_audio = Mp3Audio {
            data: vec![4, 5, 6],
            metadata: AudioMetadata::default(),
        };
        let audio_content = AudioContent::Mp3(mp3_audio.clone());
        match audio_content {
            AudioContent::Mp3(m) => assert_eq!(m.data, mp3_audio.data),
            _ => panic!("Wrong variant"),
        }
        
        // Test VideoContent enum
        let mp4_video = Mp4Video {
            data: vec![7, 8, 9],
            metadata: VideoMetadata::default(),
        };
        let video_content = VideoContent::Mp4(mp4_video.clone());
        match video_content {
            VideoContent::Mp4(m) => assert_eq!(m.data, mp4_video.data),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_codec_constants() {
        // Verify all codec constants are in the correct range
        assert!(codec::PDF >= 0x600000 && codec::PDF <= 0x60FFFF);
        assert!(codec::DOCX >= 0x600000 && codec::DOCX <= 0x60FFFF);
        assert!(codec::MARKDOWN >= 0x600000 && codec::MARKDOWN <= 0x60FFFF);
        assert!(codec::TEXT >= 0x600000 && codec::TEXT <= 0x60FFFF);
        
        assert!(codec::JPEG >= 0x610000 && codec::JPEG <= 0x61FFFF);
        assert!(codec::PNG >= 0x610000 && codec::PNG <= 0x61FFFF);
        assert!(codec::GIF >= 0x610000 && codec::GIF <= 0x61FFFF);
        assert!(codec::WEBP >= 0x610000 && codec::WEBP <= 0x61FFFF);
        
        assert!(codec::MP3 >= 0x620000 && codec::MP3 <= 0x62FFFF);
        assert!(codec::WAV >= 0x620000 && codec::WAV <= 0x62FFFF);
        assert!(codec::FLAC >= 0x620000 && codec::FLAC <= 0x62FFFF);
        assert!(codec::AAC >= 0x620000 && codec::AAC <= 0x62FFFF);
        assert!(codec::OGG >= 0x620000 && codec::OGG <= 0x62FFFF);
        
        assert!(codec::MP4 >= 0x630000 && codec::MP4 <= 0x63FFFF);
        assert!(codec::MOV >= 0x630000 && codec::MOV <= 0x63FFFF);
        assert!(codec::MKV >= 0x630000 && codec::MKV <= 0x63FFFF);
        assert!(codec::AVI >= 0x630000 && codec::AVI <= 0x63FFFF);
    }
} 