use std::io::Cursor;

use dom_smoothie::{Article, Config, Readability};
use eyre::Result;
use html2text::from_read;

/// Clean HTML content and convert to Markdown-like structured text.
/// This preserves structural elements like headings, lists, and tables
/// which are valuable for LLM understanding.
pub fn clean_html(html: &str) -> Result<String> {
    let cfg = Config { max_elements_to_parse: 9000, ..Default::default() };
    let mut readable = Readability::new(html, Some("http://example.com"), Some(cfg))?;
    let article: Article = readable.parse()?;

    // Convert HTML to structured text preserving headings, lists, links, etc.
    let content = Cursor::new(article.content.as_bytes());
    let text = from_read(content, 10000)?;

    // Clean up excess whitespace while preserving structure
    Ok(super::text::TextCleaner::new().reduce_newlines_to_double_newline().run(&text))
}
