#[derive(Default)]
pub enum Newlines {
    Space,
    Single,
    #[default]
    TwoPlus,
    None,
}

/// Represents a single step in the text cleaning pipeline.
/// Each step processes a character and updates the cleaning state.
enum CleanStep {
    /// Character should be emitted as-is
    Emit(char),
    /// A whitespace character was encountered
    Whitespace,
    /// A newline sequence was encountered
    Newline(usize),
    /// An escaped whitespace/newline sequence was processed
    EscapedWhitespace,
    /// An escaped newline sequence was processed
    EscapedNewline,
    /// A citation was removed. If true, the next character is punctuation
    /// and we should remove the trailing space before it.
    CitationRemoved(bool),
    /// A non-citation bracket and its contents should be replayed
    ReplayNonCitation(Vec<char>),
}

/// Internal state for the single-pass text cleaner.
struct CleanState {
    result: String,
    consecutive_newlines: usize,
    last_was_space: bool,
}

impl CleanState {
    fn with_capacity(capacity: usize) -> Self {
        Self {
            result: String::with_capacity(capacity),
            consecutive_newlines: 0,
            last_was_space: false,
        }
    }
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
        let mut state = CleanState::with_capacity(text.len());
        let mut chars = text.chars().peekable();

        while let Some(c) = chars.next() {
            let step = self.classify_char(c, &mut chars);

            match step {
                CleanStep::Newline(count) => {
                    self.handle_newline(&mut state, count);
                }
                CleanStep::Whitespace => {
                    self.handle_whitespace(&mut state);
                }
                CleanStep::EscapedWhitespace => {
                    self.handle_escaped_whitespace(&mut state);
                }
                CleanStep::EscapedNewline => {
                    state.consecutive_newlines += 1;
                    state.last_was_space = false;
                }
                CleanStep::CitationRemoved(remove_trailing_space) => {
                    // Citation was already consumed in classify_char
                    // If the next character is punctuation, remove the trailing space
                    if remove_trailing_space && state.last_was_space && state.result.ends_with(' ')
                    {
                        state.result.pop();
                        state.last_was_space = false;
                    }
                }
                CleanStep::ReplayNonCitation(buf) => {
                    self.emit_newlines(&mut state);
                    for ch in buf {
                        state.result.push(ch);
                    }
                    state.last_was_space = false;
                }
                CleanStep::Emit(ch) => {
                    self.emit_newlines(&mut state);
                    if !self.remove_non_basic_ascii || is_valid_text_char(ch) {
                        state.result.push(ch);
                    }
                    state.last_was_space = false;
                }
            }
        }

        // Handle any trailing newlines
        if state.consecutive_newlines > 0 {
            self.emit_newlines(&mut state);
        }

        trim_trailing_spaces(&state.result)
    }

    /// Classify a character and return the appropriate cleaning step.
    fn classify_char(
        &self,
        c: char,
        chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    ) -> CleanStep {
        match c {
            // Handle various newline sequences
            '\r' => {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                CleanStep::Newline(1)
            }
            '\n' | '\x0B' | '\x0C' | '\u{2028}' => CleanStep::Newline(1),
            '\u{2029}' => CleanStep::Newline(2),

            // Handle various whitespace characters
            ' ' |
            '\t' |
            '\u{00A0}' |
            '\u{1680}' |
            '\u{2000}'..='\u{200A}' |
            '\u{202F}' |
            '\u{205F}' |
            '\u{3000}' => CleanStep::Whitespace,

            // Handle escaped whitespace/newline sequences
            '\\' => self.classify_escape(chars),

            // Handle citations
            '[' if self.remove_citations => self.classify_citation(chars),

            // Regular character
            _ => CleanStep::Emit(c),
        }
    }

    /// Classify an escape sequence after backslash.
    fn classify_escape(&self, chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> CleanStep {
        if let Some(&next) = chars.peek() {
            match next {
                's' | 't' => {
                    chars.next();
                    CleanStep::EscapedWhitespace
                }
                'n' | 'r' => {
                    chars.next();
                    CleanStep::EscapedNewline
                }
                _ => CleanStep::Emit('\\'),
            }
        } else {
            CleanStep::Emit('\\')
        }
    }

    /// Classify a potential citation starting with '['.
    fn classify_citation(&self, chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> CleanStep {
        let mut buf = vec!['['];
        let mut is_citation = false;

        while let Some(&next) = chars.peek() {
            if next.is_ascii_digit() || next == ',' || next == '-' || next == ' ' {
                buf.push(next);
                chars.next();
            } else if next == ']' && buf.len() > 1 && buf[1..].iter().any(|b| b.is_ascii_digit()) {
                is_citation = true;
                chars.next();
                break;
            } else {
                break;
            }
        }

        if is_citation {
            // Check if the next character is punctuation - if so, we should
            // remove the trailing space before it
            let next_is_punctuation =
                chars.peek().is_some_and(|&c| matches!(c, '.' | ',' | '?' | '!' | ':' | ';'));
            CleanStep::CitationRemoved(next_is_punctuation)
        } else {
            CleanStep::ReplayNonCitation(buf)
        }
    }

    /// Handle a newline character by updating the consecutive newline count.
    fn handle_newline(&self, state: &mut CleanState, count: usize) {
        state.consecutive_newlines += count;
        state.last_was_space = false;
    }

    /// Handle a whitespace character, emitting pending newlines if needed.
    fn handle_whitespace(&self, state: &mut CleanState) {
        if state.consecutive_newlines > 0 {
            match self.newlines {
                Newlines::Space => {
                    state.result.push(' ');
                    state.consecutive_newlines = 0;
                    state.last_was_space = true;
                    return;
                }
                Newlines::Single => {
                    state.result.push('\n');
                    state.consecutive_newlines = 0;
                }
                Newlines::TwoPlus => {
                    let count = state.consecutive_newlines.min(2);
                    for _ in 0..count {
                        state.result.push('\n');
                    }
                    state.consecutive_newlines = 0;
                }
                Newlines::None => {
                    for _ in 0..state.consecutive_newlines {
                        state.result.push('\n');
                    }
                    state.consecutive_newlines = 0;
                }
            }
        }
        if !state.last_was_space {
            state.result.push(' ');
            state.last_was_space = true;
        }
    }

    /// Handle an escaped whitespace sequence (e.g., \s, \t).
    fn handle_escaped_whitespace(&self, state: &mut CleanState) {
        if !state.last_was_space && state.consecutive_newlines == 0 {
            state.result.push(' ');
            state.last_was_space = true;
        }
    }

    /// Emit accumulated newlines to the result buffer.
    fn emit_newlines(&self, state: &mut CleanState) {
        if state.consecutive_newlines == 0 {
            return;
        }
        match self.newlines {
            Newlines::Space => {
                state.result.push(' ');
            }
            Newlines::Single => {
                state.result.push('\n');
            }
            Newlines::TwoPlus => {
                let count = state.consecutive_newlines.min(2);
                for _ in 0..count {
                    state.result.push('\n');
                }
            }
            Newlines::None => {
                for _ in 0..state.consecutive_newlines {
                    state.result.push('\n');
                }
            }
        }
        state.consecutive_newlines = 0;
    }
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
