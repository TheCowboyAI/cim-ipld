//! Content Transformation Demonstration
//!
//! This example showcases content transformation capabilities in CIM-IPLD

use cim_ipld::{
    MarkdownDocument, DocumentMetadata,
    CodecOperations,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Content Transformation Demo ===\n");
    
    // Markdown transformations
    demo_markdown_transformations()?;
    
    // Metadata enrichment
    demo_metadata_enrichment()?;
    
    // Format conversions
    demo_format_conversions()?;
    
    Ok(())
}

fn demo_markdown_transformations() -> Result<(), Box<dyn std::error::Error>> {
    println!("1. Markdown Transformations:");
    
    let markdown = MarkdownDocument {
        content: r#"# Technical Documentation

## Introduction
This is a **technical** document with various elements:

### Code Example
```rust
fn main() {
    println!("Hello, IPLD!");
}
```

### Features
- Content addressing
- Cryptographic integrity
- Type safety

### Links
- [CIM Architecture](https://github.com/thecowboyai/alchemist)
- [IPLD Spec](https://ipld.io/)
"#.to_string(),
        metadata: DocumentMetadata {
            title: Some("Technical Documentation".to_string()),
            author: Some("Tech Writer".to_string()),
            tags: vec!["documentation".to_string(), "technical".to_string()],
            ..Default::default()
        },
    };
    
    println!("  Original markdown:");
    println!("    Title: {:?}", markdown.metadata.title);
    println!("    Content length: {} bytes", markdown.content.len());
    println!("    Preview: {}", &markdown.content[..100.min(markdown.content.len())]);
    
    // Simulate transformation to extract headers
    let headers = extract_markdown_headers(&markdown.content);
    println!("\n  Extracted headers:");
    for (level, text) in headers {
        println!("    {}{} {}", " ".repeat(level - 1), "#".repeat(level), text);
    }
    
    // Simulate transformation to extract links
    let links = extract_markdown_links(&markdown.content);
    println!("\n  Extracted links:");
    for (text, url) in links {
        println!("    [{}] -> {}", text, url);
    }
    
    println!();
    Ok(())
}

fn demo_metadata_enrichment() -> Result<(), Box<dyn std::error::Error>> {
    println!("2. Metadata Enrichment:");
    
    let mut doc = MarkdownDocument {
        content: "# Simple Document\n\nThis is a simple document.".to_string(),
        metadata: DocumentMetadata {
            title: Some("Simple Document".to_string()),
            ..Default::default()
        },
    };
    
    println!("  Original metadata:");
    println!("    {:?}", doc.metadata);
    
    // Enrich metadata
    let word_count = count_words(&doc.content);
    doc.metadata.language = Some("en".to_string());
    doc.metadata.tags = vec!["auto-tagged".to_string(), "simple".to_string()];
    doc.metadata.created_at = Some(chrono::Utc::now().timestamp() as u64);
    
    println!("\n  Enriched metadata:");
    println!("    Word count: {}", word_count);
    println!("    Language: {:?}", doc.metadata.language);
    println!("    Tags: {:?}", doc.metadata.tags);
    println!("    Created: {:?}", doc.metadata.created_at);
    
    println!();
    Ok(())
}

fn demo_format_conversions() -> Result<(), Box<dyn std::error::Error>> {
    println!("3. Format Conversions:");
    
    // Create a document
    let doc = MarkdownDocument {
        content: "# Title\n\nContent goes here.".to_string(),
        metadata: DocumentMetadata {
            title: Some("Conversion Example".to_string()),
            author: Some("Demo Author".to_string()),
            ..Default::default()
        },
    };
    
    // Convert to different encodings
    let json = doc.to_dag_json()?;
    let cbor = doc.to_dag_cbor()?;
    let pretty = doc.to_dag_json_pretty()?;
    
    println!("  Original document: {} bytes", doc.content.len());
    println!("  DAG-JSON encoded: {} bytes", json.len());
    println!("  DAG-CBOR encoded: {} bytes", cbor.len());
    println!("  Compression ratio: {:.1}%", 
        (cbor.len() as f64 / json.len() as f64) * 100.0
    );
    
    println!("\n  Pretty JSON (first 200 chars):");
    println!("{}", &pretty[..200.min(pretty.len())]);
    
    // Simulate conversion to plain text
    let plain_text = markdown_to_plain(&doc.content);
    println!("\n  Plain text conversion:");
    println!("    {}", plain_text);
    
    Ok(())
}

// Helper functions

fn extract_markdown_headers(content: &str) -> Vec<(usize, String)> {
    content.lines()
        .filter_map(|line| {
            if line.starts_with('#') {
                let level = line.chars().take_while(|&c| c == '#').count();
                let text = line[level..].trim().to_string();
                Some((level, text))
            } else {
                None
            }
        })
        .collect()
}

fn extract_markdown_links(content: &str) -> Vec<(String, String)> {
    let re = regex::Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").unwrap();
    re.captures_iter(content)
        .map(|cap| (cap[1].to_string(), cap[2].to_string()))
        .collect()
}

fn count_words(content: &str) -> u32 {
    content.split_whitespace().count() as u32
}

fn markdown_to_plain(content: &str) -> String {
    // Simple conversion: remove markdown syntax
    content
        .lines()
        .map(|line| {
            line.trim_start_matches('#')
                .trim()
                .replace("**", "")
                .replace("*", "")
                .replace("`", "")
        })
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}