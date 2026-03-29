#[derive(Default)]
pub enum Newlines {
    Space,
    Single,
    #[default]
    TwoPlus,
    None,
}

#[derive(Default)]
pub struct TextCleaner {
    pub newlines: Newlines,
    pub remove_non_basic_ascii: bool,
    pub remove_citations: bool,
}

impl TextCleaner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn do_not_reduce_newlines(mut self) -> Self {
        self.newlines = Newlines::None;
        self
    }

    pub fn reduce_newlines_to_single_space(mut self) -> Self {
        self.newlines = Newlines::Space;
        self
    }

    pub fn reduce_newlines_to_single_newline(mut self) -> Self {
        self.newlines = Newlines::Single;
        self
    }

    pub fn reduce_newlines_to_double_newline(mut self) -> Self {
        self.newlines = Newlines::TwoPlus;
        self
    }

    pub fn remove_non_basic_ascii(mut self) -> Self {
        self.remove_non_basic_ascii = true;
        self
    }

    pub fn remove_citations(mut self) -> Self {
        self.remove_citations = true;
        self
    }

    /// Single-pass text cleaning with integrated citation removal, whitespace
    /// collapsing, and newline normalization. Uses a blacklist approach for
    /// character filtering to preserve all visible text including URLs, code,
    /// and multilingual content.
    pub fn run(&self, text: &str) -> String {
        let mut result = String::with_capacity(text.len());
        let mut chars = text.chars().peekable();
        let mut consecutive_newlines: usize = 0;
        let mut last_was_space = false;

        while let Some(c) = chars.next() {
            match c {
                // Handle various newline sequences
                '\r' => {
                    // Check for \r\n sequence
                    if chars.peek() == Some(&'\n') {
                        chars.next();
                    }
                    consecutive_newlines += 1;
                    last_was_space = false;
                }
                '\n' | '\x0B' | '\x0C' | '\u{2028}' => {
                    consecutive_newlines += 1;
                    last_was_space = false;
                }
                // Handle paragraph separator
                '\u{2029}' => {
                    consecutive_newlines += 2;
                    last_was_space = false;
                }
                // Handle various whitespace characters
                ' ' |
                '\t' |
                '\u{00A0}' |
                '\u{1680}' |
                '\u{2000}'..='\u{200A}' |
                '\u{202F}' |
                '\u{205F}' |
                '\u{3000}' => {
                    if consecutive_newlines > 0 {
                        // Emit pending newlines (mode-aware)
                        match self.newlines {
                            Newlines::Space => {
                                result.push(' ');
                                consecutive_newlines = 0;
                                last_was_space = true;
                                continue;
                            }
                            Newlines::Single => {
                                result.push('\n');
                                consecutive_newlines = 0;
                            }
                            Newlines::TwoPlus => {
                                let count = consecutive_newlines.min(2);
                                for _ in 0..count {
                                    result.push('\n');
                                }
                                consecutive_newlines = 0;
                            }
                            Newlines::None => {
                                for _ in 0..consecutive_newlines {
                                    result.push('\n');
                                }
                                consecutive_newlines = 0;
                            }
                        }
                    }
                    if !last_was_space {
                        result.push(' ');
                        last_was_space = true;
                    }
                }
                // Handle escaped whitespace sequences (like \s, \t, \n, \r)
                '\\' => {
                    if let Some(&next) = chars.peek() {
                        match next {
                            's' | 't' => {
                                chars.next(); // consume the character after backslash
                                if !last_was_space && consecutive_newlines == 0 {
                                    result.push(' ');
                                    last_was_space = true;
                                }
                            }
                            'n' | 'r' => {
                                chars.next(); // consume the character after backslash
                                consecutive_newlines += 1;
                                last_was_space = false;
                            }
                            _ => {
                                emit_newlines(
                                    &mut result,
                                    &self.newlines,
                                    &mut consecutive_newlines,
                                );
                                result.push('\\');
                                last_was_space = false;
                            }
                        }
                    }
                }
                // Handle citations [1], [1, 2], [1-3] inline
                '[' if self.remove_citations => {
                    // Try to match citation pattern: [digits with optional , - spaces]
                    let mut buf = Vec::new();
                    buf.push('[');
                    let mut is_citation = false;

                    while let Some(&next) = chars.peek() {
                        if next.is_ascii_digit() || next == ',' || next == '-' || next == ' ' {
                            buf.push(next);
                            chars.next();
                        } else if next == ']' &&
                            buf.len() > 1 &&
                            buf[1..].iter().any(|b| b.is_ascii_digit())
                        {
                            is_citation = true;
                            chars.next(); // consume ']'
                            // Remove trailing space before punctuation
                            if last_was_space &&
                                result.ends_with(' ') &&
                                let Some(&ahead) = chars.peek() &&
                                matches!(ahead, '.' | ',' | '?' | '!' | ':' | ';')
                            {
                                result.pop();
                            }
                            break;
                        } else {
                            break;
                        }
                    }

                    if !is_citation {
                        // Flush pending newlines, then replay buffer
                        emit_newlines(&mut result, &self.newlines, &mut consecutive_newlines);
                        for ch in buf {
                            result.push(ch);
                        }
                        last_was_space = false;
                    }
                }
                // Handle regular characters
                _ => {
                    // Emit accumulated newlines before content
                    emit_newlines(&mut result, &self.newlines, &mut consecutive_newlines);

                    // Filter unwanted characters (blacklist: only remove control chars)
                    if self.remove_non_basic_ascii && !is_valid_text_char(c) {
                        continue;
                    }

                    result.push(c);
                    last_was_space = false;
                }
            }
        }

        // Handle any trailing newlines
        if consecutive_newlines > 0 {
            emit_newlines(&mut result, &self.newlines, &mut consecutive_newlines);
        }

        // Single-pass trailing cleanup: trim leading whitespace and trim
        // trailing spaces.
        trim_trailing_spaces(&result)
    }
}

/// Emit accumulated newlines to the result buffer based on the configured mode.
/// Single/None modes: emit all accumulated newlines.
/// TwoPlus mode: cap at 2.
/// Space mode: emit a single space.
#[inline]
fn emit_newlines(result: &mut String, newlines: &Newlines, consecutive_newlines: &mut usize) {
    if *consecutive_newlines == 0 {
        return;
    }
    match newlines {
        Newlines::Space => {
            result.push(' ');
            *consecutive_newlines = 0;
        }
        Newlines::Single => {
            result.push('\n');
            *consecutive_newlines = 0;
        }
        Newlines::TwoPlus => {
            let count = (*consecutive_newlines).min(2);
            for _ in 0..count {
                result.push('\n');
            }
            *consecutive_newlines = 0;
        }
        Newlines::None => {
            for _ in 0..*consecutive_newlines {
                result.push('\n');
            }
            *consecutive_newlines = 0;
        }
    }
    *consecutive_newlines = 0;
}

/// Check if a character should be kept in cleaned text.
/// Uses a blacklist approach: only removes ASCII control characters (except
/// whitespace), preserving all visible text including Unicode, URLs, code, etc.
fn is_valid_text_char(c: char) -> bool {
    !(c.is_control() && c != '\t' && c != '\n' && c != '\r')
}

/// Trim leading whitespace, trailing spaces, and normalize trailing newlines to max 2.
fn trim_trailing_spaces(text: &str) -> String {
    let trimmed = text.trim_start();
    if trimmed.is_empty() {
        return String::new();
    }
    // Trim trailing spaces and tabs
    let trimmed = trimmed.trim_end_matches([' ', '\t']);
    // Count and normalize trailing newlines
    let newline_count = trimmed.chars().rev().take_while(|&c| c == '\n' || c == '\r').count();
    if newline_count == 0 {
        return trimmed.to_string();
    }
    let body = &trimmed[..trimmed.len() - newline_count];
    let clamped = newline_count.min(2);
    let mut result = String::with_capacity(body.len() + clamped);
    result.push_str(body);
    for _ in 0..clamped {
        result.push('\n');
    }
    result
}

/// Normalize whitespace in text using single-pass processing
pub fn normalize_whitespace(text: &str) -> String {
    TextCleaner::new().do_not_reduce_newlines().run(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_to_single_spaces() {
        let ascii_text =
            "Ascii\tspaces here. Unicode\u{00A0}spaces here.\n And\nof course, newlines.\n\n";
        let ascii_result = "Ascii spaces here. Unicode spaces here. And of course, newlines.";
        assert_eq!(
            TextCleaner::new().reduce_newlines_to_single_space().run(ascii_text),
            ascii_result
        );
    }

    #[test]
    fn test_clean_to_single_newlines() {
        let ascii_text =
            "Ascii\tspaces here. Unicode\u{00A0}spaces here.\nAnd of course, newlines.\n\nCool.";
        let ascii_result =
            "Ascii spaces here. Unicode spaces here.\nAnd of course, newlines.\nCool.";
        assert_eq!(
            TextCleaner::new().reduce_newlines_to_single_newline().run(ascii_text),
            ascii_result
        );
    }

    #[test]
    fn test_clean_to_double_newlines() {
        let ascii_text = "Ascii\tspaces here. Unicode\u{00A0}spaces here.\n\nAscii\n\nparagraphs.\r\n\r\nUnicode\u{2029}paragraphs.\u{2029}\u{2028} Literal\\n\\nparagraphs.\\r\\n\\r\\n";
        let ascii_result = "Ascii spaces here. Unicode spaces here.\n\nAscii\n\nparagraphs.\n\nUnicode\n\nparagraphs.\n\n Literal\n\nparagraphs.\n\n";
        assert_eq!(
            TextCleaner::new().reduce_newlines_to_double_newline().run(ascii_text),
            ascii_result
        );
    }

    #[test]
    fn test_strip_control_chars() {
        // Blacklist approach: only control chars are removed, all visible text
        // including multilingual content is preserved
        let text_with_controls = "Hello\x00World\x01Test\u{00A0}Normal\u{2029}End";
        let expected = "HelloWorldTest Normal\n\nEnd";
        assert_eq!(
            TextCleaner::new()
                .do_not_reduce_newlines()
                .remove_non_basic_ascii()
                .run(text_with_controls),
            expected
        );
    }

    #[test]
    fn test_preserves_urls_and_code() {
        let text = "Visit https://example.com/path_to/file and run x = y + 1";
        let expected = "Visit https://example.com/path_to/file and run x = y + 1";
        assert_eq!(
            TextCleaner::new().do_not_reduce_newlines().remove_non_basic_ascii().run(text),
            expected
        );
    }

    #[test]
    fn test_preserves_multilingual_text() {
        let text = "Hello 世界 Bonne année Привет";
        assert_eq!(
            TextCleaner::new().do_not_reduce_newlines().remove_non_basic_ascii().run(text),
            text
        );
    }

    #[test]
    fn test_normalize_whitespace() {
        let ascii_text = "Ascii\tspaces here. Unicode\u{00A0}spaces here. Literal\\sspaces\\t.";
        let ascii_result = "Ascii spaces here. Unicode spaces here. Literal spaces .";
        assert_eq!(normalize_whitespace(ascii_text), ascii_result);

        let ascii_text =
            "Ascii\nnewlines\n. Unicode\u{2028}newlines.\u{2028}. Literal\\nnewlines.\\n";
        let ascii_result = "Ascii\nnewlines\n. Unicode\nnewlines.\n. Literal\nnewlines.\n";
        assert_eq!(normalize_whitespace(ascii_text), ascii_result);

        let ascii_text = "Ascii\n\nparagraphs\r\n\r\n.Unicode\u{2029}paragraphs.\u{2029} Literal\\n\\nparagraphs.\\r\\n\\r\\n";
        let result = normalize_whitespace(ascii_text);
        let ascii_result =
            "Ascii\n\nparagraphs\n\n.Unicode\n\nparagraphs.\n\n Literal\n\nparagraphs.\n\n";
        assert_eq!(result, ascii_result);
    }

    #[test]
    fn test_remove_compound_citations() {
        let text = "Studies show this [1, 2] and also [3-5] plus [6, 7, 8].";
        let expected = "Studies show this and also plus.";
        assert_eq!(TextCleaner::new().remove_citations().run(text), expected);
    }

    #[test]
    fn test_preserves_non_citation_brackets() {
        let text = "Array [1, 2, 3] and link [click here] are not citations.";
        let expected = "Array and link [click here] are not citations.";
        assert_eq!(TextCleaner::new().remove_citations().run(text), expected);
    }

    #[test]
    fn test_preserves_markdown_links() {
        let text = "See [this link](https://example.com) for details.";
        let expected = "See [this link](https://example.com) for details.";
        assert_eq!(TextCleaner::new().remove_citations().run(text), expected);
    }
}
