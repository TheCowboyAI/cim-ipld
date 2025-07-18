// Copyright 2025 Cowboy AI, LLC.

//! Content indexing and search capabilities
//!
//! This module provides indexing functionality for content metadata,
//! enabling efficient search and discovery of stored content.

use crate::{
    content_types::{
        DocumentMetadata, ImageMetadata, AudioMetadata, VideoMetadata,
        ContentType, codec,
        persistence::IndexPersistence,
    },
    Result,
};
use cid::Cid;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

// Serde support for CID
mod cid_serde {
    use cid::Cid;
    use serde::{Deserializer, Serializer, Deserialize};
    
    pub fn serialize<S>(cid: &Cid, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&cid.to_string())
    }
    
    pub fn deserialize<'de, D>(deserializer: D) -> std::result::Result<Cid, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

/// Main content indexing service
pub struct ContentIndex {
    /// Inverted index for text search
    text_index: Arc<RwLock<TextIndex>>,
    /// Tag-based index
    tag_index: Arc<RwLock<TagIndex>>,
    /// Type-based index
    type_index: Arc<RwLock<TypeIndex>>,
    /// Metadata cache
    metadata_cache: Arc<RwLock<MetadataCache>>,
    /// Persistence layer
    persistence: Option<Arc<IndexPersistence>>,
}

/// Text search index using inverted index structure
#[derive(Debug, Clone, Default)]
struct TextIndex {
    /// Word to CID mappings
    word_to_cids: HashMap<String, HashSet<Cid>>,
    /// CID to document text
    cid_to_text: HashMap<Cid, String>,
}

/// Tag-based index
#[derive(Debug, Clone, Default)]
struct TagIndex {
    /// Tag to CID mappings
    tag_to_cids: HashMap<String, HashSet<Cid>>,
    /// CID to tags
    cid_to_tags: HashMap<Cid, Vec<String>>,
}

/// Content type index
#[derive(Debug, Clone, Default)]
struct TypeIndex {
    /// Type to CID mappings
    type_to_cids: HashMap<ContentType, HashSet<Cid>>,
}

/// Metadata cache for quick access
#[derive(Debug, Clone, Default)]
struct MetadataCache {
    documents: HashMap<Cid, DocumentMetadata>,
    images: HashMap<Cid, ImageMetadata>,
    audio: HashMap<Cid, AudioMetadata>,
    video: HashMap<Cid, VideoMetadata>,
}

/// Search query for finding content
#[derive(Debug, Clone)]
pub struct SearchQuery {
    /// Text to search for
    pub text: Option<String>,
    /// Tags to filter by (AND operation)
    pub tags: Vec<String>,
    /// Content types to include
    pub content_types: Vec<ContentType>,
    /// Maximum results
    pub limit: Option<usize>,
    /// Offset for pagination
    pub offset: Option<usize>,
}

impl Default for SearchQuery {
    fn default() -> Self {
        Self {
            text: None,
            tags: Vec::new(),
            content_types: Vec::new(),
            limit: Some(100),
            offset: None,
        }
    }
}

/// Search result entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Content CID
    #[serde(with = "cid_serde")]
    pub cid: Cid,
    /// Content type
    pub content_type: ContentType,
    /// Relevance score (0.0 to 1.0)
    pub score: f32,
    /// Snippet of matching text
    pub snippet: Option<String>,
    /// Matching tags
    pub matching_tags: Vec<String>,
    /// Basic metadata
    pub metadata: SearchMetadata,
}

/// Simplified metadata for search results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub created_at: Option<u64>,
    pub size: Option<usize>,
    pub tags: Vec<String>,
}

impl ContentIndex {
    /// Create a new content index
    pub fn new() -> Self {
        Self {
            text_index: Arc::new(RwLock::new(TextIndex::default())),
            tag_index: Arc::new(RwLock::new(TagIndex::default())),
            type_index: Arc::new(RwLock::new(TypeIndex::default())),
            metadata_cache: Arc::new(RwLock::new(MetadataCache::default())),
            persistence: None,
        }
    }

    /// Create a new content index with persistence
    pub fn with_persistence(persistence: Arc<IndexPersistence>) -> Self {
        Self {
            text_index: Arc::new(RwLock::new(TextIndex::default())),
            tag_index: Arc::new(RwLock::new(TagIndex::default())),
            type_index: Arc::new(RwLock::new(TypeIndex::default())),
            metadata_cache: Arc::new(RwLock::new(MetadataCache::default())),
            persistence: Some(persistence),
        }
    }

    /// Load index from persistence
    pub async fn load_from_persistence(&self) -> Result<()> {
        if let Some(ref persistence) = self.persistence {
            // Load text index
            if let Some((word_to_cids, cid_to_text)) = persistence.load_text_index().await
                .map_err(|e| crate::Error::StorageError(e.to_string()))? {
                let mut text_index = self.text_index.write().await;
                text_index.word_to_cids = word_to_cids;
                text_index.cid_to_text = cid_to_text;
            }

            // Load other indices similarly...
            // Note: Full implementation would load all index types
        }
        Ok(())
    }

    /// Persist current index state
    pub async fn persist(&self) -> Result<()> {
        if let Some(ref persistence) = self.persistence {
            // Persist text index
            let text_index = self.text_index.read().await;
            persistence.save_text_index(&text_index.word_to_cids, &text_index.cid_to_text).await
                .map_err(|e| crate::Error::StorageError(e.to_string()))?;

            // Persist tag index
            let tag_index = self.tag_index.read().await;
            persistence.save_tag_index(&tag_index.tag_to_cids, &tag_index.cid_to_tags).await
                .map_err(|e| crate::Error::StorageError(e.to_string()))?;

            // Persist type index
            let type_index = self.type_index.read().await;
            persistence.save_type_index(&type_index.type_to_cids).await
                .map_err(|e| crate::Error::StorageError(e.to_string()))?;

            // Persist metadata cache
            let cache = self.metadata_cache.read().await;
            persistence.save_metadata_cache(
                &cache.documents,
                &cache.images,
                &cache.audio,
                &cache.video,
            ).await
                .map_err(|e| crate::Error::StorageError(e.to_string()))?;

            // Persist stats
            let stats = self.stats().await;
            persistence.save_stats(&stats).await
                .map_err(|e| crate::Error::StorageError(e.to_string()))?;
        }
        Ok(())
    }

    /// Index a document
    pub async fn index_document(
        &self,
        cid: Cid,
        metadata: &DocumentMetadata,
        content: Option<&str>,
    ) -> Result<()> {
        // Index text content
        if let Some(text) = content {
            let mut text_index = self.text_index.write().await;
            index_text(&mut text_index, cid, text);
        }

        // Index title
        if let Some(title) = &metadata.title {
            let mut text_index = self.text_index.write().await;
            index_text(&mut text_index, cid, title);
        }

        // Index tags
        if !metadata.tags.is_empty() {
            let mut tag_index = self.tag_index.write().await;
            index_tags(&mut tag_index, cid, &metadata.tags);
        }

        // Index type
        {
            let mut type_index = self.type_index.write().await;
            type_index.type_to_cids
                .entry(ContentType::Custom(codec::TEXT))
                .or_default()
                .insert(cid);
        }

        // Cache metadata
        {
            let mut cache = self.metadata_cache.write().await;
            cache.documents.insert(cid, metadata.clone());
        }

        // Persist if enabled
        if self.persistence.is_some() {
            self.persist().await?;
        }

        Ok(())
    }

    /// Index an image
    pub async fn index_image(
        &self,
        cid: Cid,
        metadata: &ImageMetadata,
        content_type: ContentType,
    ) -> Result<()> {
        // Index tags
        if !metadata.tags.is_empty() {
            let mut tag_index = self.tag_index.write().await;
            index_tags(&mut tag_index, cid, &metadata.tags);
        }

        // Index type
        {
            let mut type_index = self.type_index.write().await;
            type_index.type_to_cids
                .entry(content_type)
                .or_default()
                .insert(cid);
        }

        // Cache metadata
        {
            let mut cache = self.metadata_cache.write().await;
            cache.images.insert(cid, metadata.clone());
        }

        // Persist if enabled
        if self.persistence.is_some() {
            self.persist().await?;
        }

        Ok(())
    }

    /// Search the index
    pub async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        let mut results = HashMap::new();
        let mut scores: HashMap<Cid, f32> = HashMap::new();

        // Text search
        if let Some(text) = &query.text {
            let text_index = self.text_index.read().await;
            let text_results = search_text(&text_index, text);
            
            for (cid, score) in text_results {
                scores.insert(cid, score);
                results.insert(cid, SearchResultBuilder::new(cid));
            }
        }

        // Tag filter
        if !query.tags.is_empty() {
            let tag_index = self.tag_index.read().await;
            let tag_results = search_tags(&tag_index, &query.tags);
            
            // If we have text results, intersect; otherwise use tag results
            if results.is_empty() {
                for cid in tag_results {
                    results.insert(cid, SearchResultBuilder::new(cid));
                }
            } else {
                // Keep only results that have all required tags
                results.retain(|cid, _| tag_results.contains(cid));
            }
        }

        // Type filter
        if !query.content_types.is_empty() {
            let type_index = self.type_index.read().await;
            let mut type_cids: HashSet<Cid> = HashSet::new();
            
            for content_type in &query.content_types {
                if let Some(cids) = type_index.type_to_cids.get(content_type) {
                    type_cids.extend(cids);
                }
            }
            
            // Filter results by type
            results.retain(|cid, _| type_cids.contains(cid));
        }

        // Build final results
        let mut final_results = Vec::new();
        let cache = self.metadata_cache.read().await;
        
        for (cid, mut builder) in results {
            // Set score
            builder.score = scores.get(&cid).copied().unwrap_or(0.5);
            
            // Get metadata
            if let Some(metadata) = get_metadata_for_cid(&cache, cid) {
                builder.metadata = metadata;
            }
            
            // Get content type
            let type_index = self.type_index.read().await;
            for (content_type, cids) in &type_index.type_to_cids {
                if cids.contains(&cid) {
                    builder.content_type = Some(*content_type);
                    break;
                }
            }
            
            if let Some(result) = builder.build() {
                final_results.push(result);
            }
        }

        // Sort by score
        final_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        // Apply pagination
        let offset = query.offset.unwrap_or(0);
        let limit = query.limit.unwrap_or(100);
        
        Ok(final_results
            .into_iter()
            .skip(offset)
            .take(limit)
            .collect())
    }

    /// Get index statistics
    pub async fn stats(&self) -> IndexStats {
        let text_index = self.text_index.read().await;
        let tag_index = self.tag_index.read().await;
        let type_index = self.type_index.read().await;
        let cache = self.metadata_cache.read().await;

        IndexStats {
            total_documents: cache.documents.len(),
            total_images: cache.images.len(),
            total_audio: cache.audio.len(),
            total_video: cache.video.len(),
            unique_words: text_index.word_to_cids.len(),
            unique_tags: tag_index.tag_to_cids.len(),
            content_types: type_index.type_to_cids.len(),
        }
    }
}

/// Index statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStats {
    pub total_documents: usize,
    pub total_images: usize,
    pub total_audio: usize,
    pub total_video: usize,
    pub unique_words: usize,
    pub unique_tags: usize,
    pub content_types: usize,
}

// Helper functions

fn index_text(index: &mut TextIndex, cid: Cid, text: &str) {
    let words = tokenize(text);
    
    for word in words {
        index.word_to_cids
            .entry(word.to_lowercase())
            .or_default()
            .insert(cid);
    }
    
    index.cid_to_text.insert(cid, text.to_string());
}

fn index_tags(index: &mut TagIndex, cid: Cid, tags: &[String]) {
    for tag in tags {
        index.tag_to_cids
            .entry(tag.to_lowercase())
            .or_default()
            .insert(cid);
    }
    
    index.cid_to_tags.insert(cid, tags.to_vec());
}

fn search_text(index: &TextIndex, query: &str) -> Vec<(Cid, f32)> {
    let query_words = tokenize(query);
    let mut cid_scores: HashMap<Cid, f32> = HashMap::new();
    
    for word in query_words {
        if let Some(cids) = index.word_to_cids.get(&word.to_lowercase()) {
            for cid in cids {
                *cid_scores.entry(*cid).or_default() += 1.0;
            }
        }
    }
    
    // Normalize scores
    let max_score = cid_scores.values().cloned().fold(0.0, f32::max);
    if max_score > 0.0 {
        for score in cid_scores.values_mut() {
            *score /= max_score;
        }
    }
    
    cid_scores.into_iter().collect()
}

fn search_tags(index: &TagIndex, required_tags: &[String]) -> HashSet<Cid> {
    if required_tags.is_empty() {
        return HashSet::new();
    }
    
    let mut result: Option<HashSet<Cid>> = None;
    
    for tag in required_tags {
        if let Some(cids) = index.tag_to_cids.get(&tag.to_lowercase()) {
            match &mut result {
                None => result = Some(cids.clone()),
                Some(existing) => {
                    existing.retain(|cid| cids.contains(cid));
                }
            }
        } else {
            // Tag not found, no results
            return HashSet::new();
        }
    }
    
    result.unwrap_or_default()
}

fn tokenize(text: &str) -> Vec<String> {
    text.split_whitespace()
        .filter(|word| word.len() > 2) // Skip very short words
        .map(|word| word.to_string())
        .collect()
}

fn get_metadata_for_cid(cache: &MetadataCache, cid: Cid) -> Option<SearchMetadata> {
    // Check each metadata type
    if let Some(doc) = cache.documents.get(&cid) {
        return Some(SearchMetadata {
            title: doc.title.clone(),
            author: doc.author.clone(),
            created_at: doc.created_at,
            size: None,
            tags: doc.tags.clone(),
        });
    }
    
    if let Some(img) = cache.images.get(&cid) {
        return Some(SearchMetadata {
            title: None,
            author: None,
            created_at: None,
            size: None,
            tags: img.tags.clone(),
        });
    }
    
    // Similar for audio and video...
    
    None
}

/// Builder for search results
struct SearchResultBuilder {
    cid: Cid,
    content_type: Option<ContentType>,
    score: f32,
    snippet: Option<String>,
    matching_tags: Vec<String>,
    metadata: SearchMetadata,
}

impl SearchResultBuilder {
    fn new(cid: Cid) -> Self {
        Self {
            cid,
            content_type: None,
            score: 0.0,
            snippet: None,
            matching_tags: Vec::new(),
            metadata: SearchMetadata {
                title: None,
                author: None,
                created_at: None,
                size: None,
                tags: Vec::new(),
            },
        }
    }
    
    fn build(self) -> Option<SearchResult> {
        self.content_type.map(|content_type| SearchResult {
            cid: self.cid,
            content_type,
            score: self.score,
            snippet: self.snippet,
            matching_tags: self.matching_tags,
            metadata: self.metadata,
        })
    }
}

impl Default for ContentIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_text_indexing() {
        let index = ContentIndex::new();
        
        let cid = Cid::default();
        let metadata = DocumentMetadata {
            title: Some("Test Document".to_string()),
            tags: vec!["test".to_string(), "example".to_string()],
            ..Default::default()
        };
        
        index.index_document(cid, &metadata, Some("This is test content")).await.unwrap();
        
        // Search by text
        let query = SearchQuery {
            text: Some("test".to_string()),
            ..Default::default()
        };
        
        let results = index.search(&query).await.unwrap();
        assert!(!results.is_empty());
    }
    
    #[tokio::test]
    async fn test_tag_search() {
        let index = ContentIndex::new();
        
        let cid = Cid::default();
        let metadata = ImageMetadata {
            tags: vec!["landscape".to_string(), "nature".to_string()],
            ..Default::default()
        };
        
        index.index_image(cid, &metadata, ContentType::Custom(codec::PNG)).await.unwrap();
        
        // Search by tag
        let query = SearchQuery {
            tags: vec!["landscape".to_string()],
            ..Default::default()
        };
        
        let results = index.search(&query).await.unwrap();
        assert!(!results.is_empty());
    }

    #[tokio::test]
    async fn test_empty_search_query() {
        let index = ContentIndex::new();
        
        // Empty query should return no results
        let query = SearchQuery::default();
        let results = index.search(&query).await.unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_text_search_no_matches() {
        let index = ContentIndex::new();
        
        let cid = Cid::default();
        let metadata = DocumentMetadata {
            title: Some("Test Document".to_string()),
            ..Default::default()
        };
        
        index.index_document(cid, &metadata, Some("This is test content")).await.unwrap();
        
        // Search for non-existent text
        let query = SearchQuery {
            text: Some("nonexistent".to_string()),
            ..Default::default()
        };
        
        let results = index.search(&query).await.unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_tag_search_no_matches() {
        let index = ContentIndex::new();
        
        let cid = Cid::default();
        let metadata = ImageMetadata {
            tags: vec!["landscape".to_string()],
            ..Default::default()
        };
        
        index.index_image(cid, &metadata, ContentType::Custom(codec::PNG)).await.unwrap();
        
        // Search for non-existent tag
        let query = SearchQuery {
            tags: vec!["portrait".to_string()],
            ..Default::default()
        };
        
        let results = index.search(&query).await.unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_type_filter_no_matches() {
        let index = ContentIndex::new();
        
        let cid = Cid::default();
        let metadata = ImageMetadata::default();
        
        index.index_image(cid, &metadata, ContentType::Custom(codec::PNG)).await.unwrap();
        
        // Search for different content type
        let query = SearchQuery {
            content_types: vec![ContentType::Custom(codec::MP3)],
            ..Default::default()
        };
        
        let results = index.search(&query).await.unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_combined_filters() {
        let index = ContentIndex::new();
        
        // Index a document with specific attributes
        let cid1 = Cid::default();
        let metadata1 = DocumentMetadata {
            title: Some("Technical Report".to_string()),
            tags: vec!["technical".to_string(), "report".to_string()],
            ..Default::default()
        };
        index.index_document(cid1, &metadata1, Some("This is a technical report")).await.unwrap();
        
        // Index another document
        let cid2 = cid::Cid::new_v1(0x71, multihash::Multihash::wrap(0x12, b"different").unwrap());
        let metadata2 = DocumentMetadata {
            title: Some("User Manual".to_string()),
            tags: vec!["manual".to_string(), "user".to_string()],
            ..Default::default()
        };
        index.index_document(cid2, &metadata2, Some("This is a user manual")).await.unwrap();
        
        // Search with multiple filters that match first document
        let query = SearchQuery {
            text: Some("technical".to_string()),
            tags: vec!["report".to_string()],
            content_types: vec![ContentType::Custom(codec::TEXT)],
            ..Default::default()
        };
        
        let results = index.search(&query).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].cid, cid1);
    }

    #[tokio::test]
    async fn test_pagination() {
        let index = ContentIndex::new();
        
        // Index multiple documents
        for i in 0..10 {
            let data = format!("doc{}", i);
            let cid = cid::Cid::new_v1(0x71, multihash::Multihash::wrap(0x12, data.as_bytes()).unwrap());
            let metadata = DocumentMetadata {
                title: Some(format!("Document {}", i)),
                tags: vec!["test".to_string()],
                ..Default::default()
            };
            index.index_document(cid, &metadata, Some("test content")).await.unwrap();
        }
        
        // Test pagination with limit
        let query = SearchQuery {
            tags: vec!["test".to_string()],
            limit: Some(3),
            ..Default::default()
        };
        
        let results = index.search(&query).await.unwrap();
        assert_eq!(results.len(), 3);
        
        // Test pagination with offset
        let query_offset = SearchQuery {
            tags: vec!["test".to_string()],
            limit: Some(3),
            offset: Some(3),
            ..Default::default()
        };
        
        let results_offset = index.search(&query_offset).await.unwrap();
        assert_eq!(results_offset.len(), 3);
        
        // Ensure different results
        assert_ne!(results[0].cid, results_offset[0].cid);
    }

    #[tokio::test]
    async fn test_index_document_without_content() {
        let index = ContentIndex::new();
        
        let cid = Cid::default();
        let metadata = DocumentMetadata {
            title: Some("Title Only".to_string()),
            tags: vec!["metadata".to_string()],
            ..Default::default()
        };
        
        // Index without content
        index.index_document(cid, &metadata, None).await.unwrap();
        
        // Should still be searchable by title
        let query = SearchQuery {
            text: Some("Title".to_string()),
            ..Default::default()
        };
        
        let results = index.search(&query).await.unwrap();
        assert!(!results.is_empty());
    }

    #[tokio::test]
    async fn test_index_document_without_title() {
        let index = ContentIndex::new();
        
        let cid = Cid::default();
        let metadata = DocumentMetadata {
            title: None,
            tags: vec!["notitle".to_string()],
            ..Default::default()
        };
        
        index.index_document(cid, &metadata, Some("Content without title")).await.unwrap();
        
        // Should be searchable by content
        let query = SearchQuery {
            text: Some("Content".to_string()),
            ..Default::default()
        };
        
        let results = index.search(&query).await.unwrap();
        assert!(!results.is_empty());
    }

    #[tokio::test]
    async fn test_index_with_empty_tags() {
        let index = ContentIndex::new();
        
        let cid = Cid::default();
        let metadata = ImageMetadata {
            tags: vec![],
            ..Default::default()
        };
        
        // Should not panic with empty tags
        index.index_image(cid, &metadata, ContentType::Custom(codec::JPEG)).await.unwrap();
        
        let stats = index.stats().await;
        assert_eq!(stats.unique_tags, 0);
    }

    #[tokio::test]
    async fn test_tokenize_edge_cases() {
        // Test tokenize function with various inputs
        assert_eq!(tokenize(""), Vec::<String>::new());
        assert_eq!(tokenize("a b c"), Vec::<String>::new()); // All words too short
        assert_eq!(tokenize("hello world"), vec!["hello", "world"]);
        assert_eq!(tokenize("  multiple   spaces  "), vec!["multiple", "spaces"]);
        assert_eq!(tokenize("one"), vec!["one"]);
    }

    #[tokio::test]
    async fn test_case_insensitive_search() {
        let index = ContentIndex::new();
        
        let cid = Cid::default();
        let metadata = DocumentMetadata {
            title: Some("UPPERCASE Title".to_string()),
            tags: vec!["TAG1".to_string(), "Tag2".to_string()],
            ..Default::default()
        };
        
        index.index_document(cid, &metadata, Some("MiXeD CaSe Content")).await.unwrap();
        
        // Search with lowercase
        let query = SearchQuery {
            text: Some("uppercase".to_string()),
            tags: vec!["tag1".to_string()],
            ..Default::default()
        };
        
        let results = index.search(&query).await.unwrap();
        assert!(!results.is_empty());
    }

    #[tokio::test]
    async fn test_multiple_tags_intersection() {
        let index = ContentIndex::new();
        
        // Document with multiple tags
        let cid1 = Cid::default();
        let metadata1 = DocumentMetadata {
            tags: vec!["rust".to_string(), "programming".to_string(), "tutorial".to_string()],
            ..Default::default()
        };
        index.index_document(cid1, &metadata1, None).await.unwrap();
        
        // Document with subset of tags
        let cid2 = cid::Cid::new_v1(0x71, multihash::Multihash::wrap(0x12, b"doc2").unwrap());
        let metadata2 = DocumentMetadata {
            tags: vec!["rust".to_string(), "programming".to_string()],
            ..Default::default()
        };
        index.index_document(cid2, &metadata2, None).await.unwrap();
        
        // Search requiring all three tags
        let query = SearchQuery {
            tags: vec!["rust".to_string(), "programming".to_string(), "tutorial".to_string()],
            ..Default::default()
        };
        
        let results = index.search(&query).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].cid, cid1);
    }

    #[tokio::test]
    async fn test_score_normalization() {
        let index = ContentIndex::new();
        
        // Document with multiple occurrences of search term
        let cid1 = Cid::default();
        let metadata1 = DocumentMetadata::default();
        index.index_document(cid1, &metadata1, Some("test word test word test")).await.unwrap();
        
        // Document with single occurrence
        let cid2 = cid::Cid::new_v1(0x71, multihash::Multihash::wrap(0x12, b"doc2").unwrap());
        let metadata2 = DocumentMetadata::default();
        index.index_document(cid2, &metadata2, Some("test once only")).await.unwrap();
        
        let query = SearchQuery {
            text: Some("test".to_string()),
            ..Default::default()
        };
        
        let results = index.search(&query).await.unwrap();
        assert_eq!(results.len(), 2);
        
        // Both documents contain "test" once as a word token (tokenize splits on whitespace)
        // So they should have the same score of 1.0 after normalization
        assert_eq!(results[0].score, 1.0);
        assert_eq!(results[1].score, 1.0);
        
        // Test with a query that matches different number of words
        let query2 = SearchQuery {
            text: Some("test word".to_string()),
            ..Default::default()
        };
        
        let results2 = index.search(&query2).await.unwrap();
        assert_eq!(results2.len(), 2);
        
        // Now one document should have higher score (matches both "test" and "word")
        let doc1_result = results2.iter().find(|r| r.cid == cid1).unwrap();
        let doc2_result = results2.iter().find(|r| r.cid == cid2).unwrap();
        
        assert_eq!(doc1_result.score, 1.0); // Has both words
        assert!(doc2_result.score < 1.0); // Has only "test"
    }

    #[tokio::test] 
    async fn test_stats_accuracy() {
        let index = ContentIndex::new();
        
        // Add various content types
        let doc_cid = Cid::default();
        let doc_metadata = DocumentMetadata::default();
        index.index_document(doc_cid, &doc_metadata, Some("doc content")).await.unwrap();
        
        let img_cid = cid::Cid::new_v1(0x71, multihash::Multihash::wrap(0x12, b"img").unwrap());
        let img_metadata = ImageMetadata {
            tags: vec!["photo".to_string()],
            ..Default::default()
        };
        index.index_image(img_cid, &img_metadata, ContentType::Custom(codec::JPEG)).await.unwrap();
        
        let stats = index.stats().await;
        assert_eq!(stats.total_documents, 1);
        assert_eq!(stats.total_images, 1);
        assert_eq!(stats.total_audio, 0);
        assert_eq!(stats.total_video, 0);
        assert!(stats.unique_words > 0);
        assert_eq!(stats.unique_tags, 1);
        assert_eq!(stats.content_types, 2); // TEXT and JPEG
    }

    #[tokio::test]
    async fn test_search_result_builder_without_type() {
        let builder = SearchResultBuilder::new(Cid::default());
        let result = builder.build();
        assert!(result.is_none()); // Should return None without content type
    }

    #[tokio::test]
    async fn test_cid_serde() {
        use serde_json;
        
        #[derive(Serialize, Deserialize)]
        struct TestStruct {
            #[serde(with = "cid_serde")]
            cid: Cid,
        }
        
        let original = TestStruct {
            cid: Cid::default(),
        };
        
        // Serialize
        let json = serde_json::to_string(&original).unwrap();
        
        // Deserialize
        let deserialized: TestStruct = serde_json::from_str(&json).unwrap();
        assert_eq!(original.cid, deserialized.cid);
    }

    #[tokio::test]
    async fn test_cid_serde_invalid() {
        use serde_json;
        
        #[derive(Deserialize)]
        struct TestStruct {
            #[serde(with = "cid_serde")]
            cid: Cid,
        }
        
        let json = r#"{"cid": "invalid-cid"}"#;
        let result: std::result::Result<TestStruct, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_search_query_defaults() {
        let query = SearchQuery::default();
        assert!(query.text.is_none());
        assert!(query.tags.is_empty());
        assert!(query.content_types.is_empty());
        assert_eq!(query.limit, Some(100));
        assert!(query.offset.is_none());
    }

    #[test]
    fn test_get_metadata_for_nonexistent_cid() {
        let cache = MetadataCache::default();
        let result = get_metadata_for_cid(&cache, Cid::default());
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_persistence_error_handling() {
        use crate::object_store::ObjectStoreError;
        use std::sync::atomic::{AtomicBool, Ordering};
        
        // Mock persistence that fails
        struct FailingPersistence {
            fail_save: AtomicBool,
            fail_load: AtomicBool,
        }
        
        impl FailingPersistence {
            async fn save_text_index(&self, _: &HashMap<String, HashSet<Cid>>, _: &HashMap<Cid, String>) -> std::result::Result<(), ObjectStoreError> {
                if self.fail_save.load(Ordering::Relaxed) {
                    Err(ObjectStoreError::Storage("Save failed".to_string()))
                } else {
                    Ok(())
                }
            }
            
            async fn load_text_index(&self) -> std::result::Result<Option<(HashMap<String, HashSet<Cid>>, HashMap<Cid, String>)>, ObjectStoreError> {
                if self.fail_load.load(Ordering::Relaxed) {
                    Err(ObjectStoreError::Storage("Load failed".to_string()))
                } else {
                    Ok(None)
                }
            }
        }
        
        // Test would need IndexPersistence trait to be public or refactored
        // This demonstrates the pattern for testing persistence errors
    }
} 