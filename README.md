# llm-utils

Rust utilities for LLM text processing. Workspace provides shared libraries for cleaning, splitting, and extracting content from text and HTML for use with large language models.

## Crates

| Crate | Description |
|-------|-------------|
| [llm-text](crates/llm-text) | Text cleaning, HTML-to-text extraction, URL extraction, and text splitting |
| [opml-protocol](crates/opml-protocol) | OPML 2.0 parsing and generation with serde support |

## llm-text

- **Text Cleaning**: Normalize whitespace, remove citations, strip non-ASCII
- **HTML Extraction**: Convert HTML to structured text using readability algorithms
- **URL Extraction**: Extract and deduplicate URLs from text
- **Text Splitting**: Sentence-level and recursive splitting for chunking

## opml-protocol

- **OPML 2.0 Parsing**: Deserialize OPML XML to Rust structs via `quick-xml` + `serde`
- **OPML 2.0 Generation**: Serialize OPML documents back to XML
- **Builder Pattern**: Ergonomic document construction with `derive_builder`
- **Feed Extraction**: Extract RSS feeds grouped by folder hierarchy

## Usage

### llm-text

```rust
use llm_text::text::TextCleaner;
use llm_text::links::extract_urls;
use llm_text::html::clean_html;

// Clean text
let cleaner = TextCleaner::new().reduce_newlines_to_double_newline();
let clean = cleaner.run("Hello\r\n\r\n\r\nWorld!");

// Extract URLs
let urls = extract_urls("Visit https://example.com or https://rust-lang.org");

// HTML to clean text
let text = clean_html("<html><body><h1>Title</h1><p>Content</p></body></html>")?;
```

### opml-protocol

```rust
use opml_protocol::{Opml, OutlineBuilder};
use std::str::FromStr;

// Parse OPML
let opml = Opml::from_str("<opml version=\"2.0\">...</opml>")?;

// Build OPML
let outline = OutlineBuilder::default()
    .text("Feed".to_string())
    .type_(Some("rss".to_string()))
    .xml_url(Some("https://example.com/feed.xml".to_string()))
    .build()?;
```

## Development

```bash
just format
just lint
just test
just publish-check
```

## License

Apache-2.0
