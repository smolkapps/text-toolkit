//! Word/line/char/byte counting, plus pattern occurrence counting.

use regex::Regex;

/// Counts produced by [`count`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Counts {
    pub lines: usize,
    pub words: usize,
    pub chars: usize,
    pub bytes: usize,
}

/// Which fields to display. If none are selected the caller should default to
/// showing lines, words, and bytes (the classic `wc` default).
#[derive(Debug, Clone, Copy, Default)]
pub struct Select {
    pub lines: bool,
    pub words: bool,
    pub chars: bool,
    pub bytes: bool,
}

impl Select {
    /// True if no field flag was explicitly requested.
    pub fn is_empty(&self) -> bool {
        !(self.lines || self.words || self.chars || self.bytes)
    }
}

/// Compute all counts over `text`.
///
/// - `lines`: number of newline (`\n`) characters. This matches GNU `wc -l`,
///   which counts line terminators, so trailing-newline-terminated text counts
///   the final line and text without a trailing newline does not count the
///   last partial line as a terminator.
/// - `words`: maximal runs of non-whitespace separated by Unicode whitespace.
/// - `chars`: number of Unicode scalar values (`char`s).
/// - `bytes`: number of UTF-8 bytes.
pub fn count(text: &str) -> Counts {
    Counts {
        lines: text.bytes().filter(|&b| b == b'\n').count(),
        words: text.split_whitespace().count(),
        chars: text.chars().count(),
        bytes: text.len(),
    }
}

/// Format `counts` honoring `select`. When `select.is_empty()`, defaults to
/// lines/words/bytes. Fields are emitted in the fixed order lines, words,
/// chars, bytes — each on the same line, separated by a single space, matching
/// roughly the column ordering of GNU `wc`.
pub fn format(counts: &Counts, select: &Select) -> String {
    let sel = if select.is_empty() {
        Select {
            lines: true,
            words: true,
            chars: false,
            bytes: true,
        }
    } else {
        *select
    };

    let mut parts: Vec<String> = Vec::new();
    if sel.lines {
        parts.push(counts.lines.to_string());
    }
    if sel.words {
        parts.push(counts.words.to_string());
    }
    if sel.chars {
        parts.push(counts.chars.to_string());
    }
    if sel.bytes {
        parts.push(counts.bytes.to_string());
    }
    parts.join(" ")
}

/// Count occurrences of `pattern` (a regex) in `text`.
///
/// Counts non-overlapping matches across the whole input (multi-line). Returns
/// an error if the pattern is not a valid regex.
pub fn count_pattern(text: &str, pattern: &str) -> anyhow::Result<usize> {
    let re = Regex::new(pattern)?;
    Ok(re.find_iter(text).count())
}

#[cfg(test)]
mod tests {
    use super::*;

    // A known sample with a deliberate mix: 3 lines (all newline-terminated),
    // 9 words, a tab and double spaces, plus a 2-byte UTF-8 char ("é").
    const SAMPLE: &str = "hello world foo\nbar  baz\tqux\nthe café end\n";

    #[test]
    fn counts_lines_words_chars_bytes() {
        let c = count(SAMPLE);
        assert_eq!(c.lines, 3, "three newline terminators");
        assert_eq!(c.words, 9, "split_whitespace collapses runs/tabs");
        // chars: count scalar values. "café" contributes 4 chars.
        assert_eq!(c.chars, SAMPLE.chars().count());
        // bytes > chars because é is 2 bytes in UTF-8.
        assert_eq!(c.bytes, SAMPLE.len());
        assert!(c.bytes > c.chars, "é makes bytes exceed chars");
        assert_eq!(c.bytes, c.chars + 1, "exactly one extra byte from é");
    }

    #[test]
    fn no_trailing_newline_counts_one_fewer_line() {
        // GNU wc -l counts terminators, so no trailing \n -> last line not counted.
        assert_eq!(count("a\nb").lines, 1);
        assert_eq!(count("a\nb\n").lines, 2);
    }

    #[test]
    fn empty_input_all_zero() {
        assert_eq!(count(""), Counts::default());
    }

    #[test]
    fn format_default_is_lines_words_bytes() {
        let c = count(SAMPLE);
        let s = format(&c, &Select::default());
        assert_eq!(s, format!("{} {} {}", c.lines, c.words, c.bytes));
    }

    #[test]
    fn format_selected_only_words() {
        let c = count(SAMPLE);
        let sel = Select {
            words: true,
            ..Default::default()
        };
        assert_eq!(format(&c, &sel), c.words.to_string());
    }

    #[test]
    fn format_selected_chars_and_lines_order() {
        let c = count(SAMPLE);
        let sel = Select {
            lines: true,
            chars: true,
            ..Default::default()
        };
        // Order is always lines, words, chars, bytes.
        assert_eq!(format(&c, &sel), format!("{} {}", c.lines, c.chars));
    }

    #[test]
    fn pattern_count_simple() {
        let n = count_pattern(SAMPLE, "ba").unwrap();
        assert_eq!(n, 2, "matches 'ba' in bar and baz");
    }

    #[test]
    fn pattern_count_word_boundary() {
        let n = count_pattern("foo foofoo foo", r"\bfoo\b").unwrap();
        assert_eq!(n, 2);
    }

    #[test]
    fn pattern_count_invalid_regex_errors() {
        assert!(count_pattern("x", "(").is_err());
    }
}
