# llm-text

A Rust library for processing text for LLM consumption. Provides utilities for cleaning, splitting, and extracting content from HTML and other sources.

## Features

- **Text Cleaning**: Clean and normalize text with configurable newline handling
- **HTML Processing**: Extract clean text from HTML content
- **Link Extraction**: Extract and analyze links from text
- **Text Splitting**: Split large texts into manageable chunks

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
llm-text = "0.1.0"
```

## Usage

```rust
use llm_text::text::TextCleaner;

let dirty_text = "Hello\r\n\r\n\r\nWorld!";
let cleaner = TextCleaner::new().reduce_newlines_to_single_newline();
let clean = cleaner.clean(dirty_text);
assert_eq!(clean, "Hello\r\nWorld!");
```
