//! Trim trailing whitespace and normalize blank lines.

/// Trim each line's trailing whitespace and collapse runs of blank lines.
///
/// - Trailing whitespace (spaces, tabs, `\r`) is stripped from every line.
/// - Runs of two or more consecutive blank lines collapse to a single blank
///   line.
/// - Leading and trailing blank lines are removed entirely.
/// - The result always ends with exactly one trailing newline (unless empty).
pub fn trim(text: &str) -> String {
    let mut lines: Vec<&str> = text
        .split('\n')
        .map(|l| l.trim_end_matches([' ', '\t', '\r']))
        .collect();

    // `split('\n')` on a trailing-newline input yields a trailing empty element;
    // drop trailing blanks (and leading blanks) below regardless.

    // Collapse interior runs of blank lines to one.
    let mut collapsed: Vec<&str> = Vec::with_capacity(lines.len());
    let mut prev_blank = false;
    for line in lines.drain(..) {
        let is_blank = line.is_empty();
        if is_blank && prev_blank {
            continue;
        }
        collapsed.push(line);
        prev_blank = is_blank;
    }

    // Strip leading blank lines.
    while collapsed.first().is_some_and(|l| l.is_empty()) {
        collapsed.remove(0);
    }
    // Strip trailing blank lines.
    while collapsed.last().is_some_and(|l| l.is_empty()) {
        collapsed.pop();
    }

    if collapsed.is_empty() {
        return String::new();
    }

    let mut out = collapsed.join("\n");
    out.push('\n');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_trailing_whitespace() {
        assert_eq!(trim("hello   \nworld\t\n"), "hello\nworld\n");
    }

    #[test]
    fn collapses_blank_runs() {
        assert_eq!(trim("a\n\n\n\nb\n"), "a\n\nb\n");
    }

    #[test]
    fn removes_leading_and_trailing_blanks() {
        assert_eq!(trim("\n\nhello\n\n\n"), "hello\n");
    }

    #[test]
    fn strips_cr_from_crlf() {
        assert_eq!(trim("a\r\nb\r\n"), "a\nb\n");
    }

    #[test]
    fn whitespace_only_line_becomes_blank_then_collapses() {
        // "a", "   " (->blank), "b" : a single blank stays between.
        assert_eq!(trim("a\n   \nb\n"), "a\n\nb\n");
    }

    #[test]
    fn all_blank_input_yields_empty() {
        assert_eq!(trim("\n\n   \n\t\n"), "");
    }

    #[test]
    fn single_line_no_newline_gets_one() {
        assert_eq!(trim("hello   "), "hello\n");
    }
}
