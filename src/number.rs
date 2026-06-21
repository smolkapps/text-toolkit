//! Line numbering and head/tail selection.

/// Prefix each line with a 1-based, right-aligned line number followed by a
/// tab. Numbering width is sized to the largest line number so columns align.
///
/// Operates on the provided slice of lines and returns the rendered block with
/// a trailing newline (empty input -> empty output).
pub fn number(lines: &[&str]) -> String {
    if lines.is_empty() {
        return String::new();
    }
    let width = lines.len().to_string().len();
    let mut out = String::new();
    for (i, line) in lines.iter().enumerate() {
        let n = i + 1;
        out.push_str(&format!("{n:>width$}\t{line}\n"));
    }
    out
}

/// Return the first `n` lines (clamped to the available count).
pub fn head<'a>(lines: &[&'a str], n: usize) -> Vec<&'a str> {
    lines.iter().take(n).copied().collect()
}

/// Return the last `n` lines (clamped to the available count), preserving order.
pub fn tail<'a>(lines: &[&'a str], n: usize) -> Vec<&'a str> {
    let len = lines.len();
    let start = len.saturating_sub(n);
    lines[start..].to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(s: &str) -> Vec<&str> {
        crate::split_lines(s)
    }

    #[test]
    fn numbers_lines_with_tab() {
        let input = lines("a\nb\nc\n");
        assert_eq!(number(&input), "1\ta\n2\tb\n3\tc\n");
    }

    #[test]
    fn numbering_is_right_aligned_to_width() {
        // 10 lines -> width 2, so line 1 becomes " 1".
        let v: Vec<&str> = (0..10).map(|_| "x").collect();
        let out = number(&v);
        assert!(out.starts_with(" 1\tx\n"), "got: {out:?}");
        assert!(out.contains("10\tx\n"));
    }

    #[test]
    fn head_takes_first_n() {
        let input = lines("1\n2\n3\n4\n5\n");
        assert_eq!(head(&input, 3), vec!["1", "2", "3"]);
    }

    #[test]
    fn head_clamps_to_len() {
        let input = lines("1\n2\n");
        assert_eq!(head(&input, 10), vec!["1", "2"]);
    }

    #[test]
    fn tail_takes_last_n_in_order() {
        let input = lines("1\n2\n3\n4\n5\n");
        assert_eq!(tail(&input, 2), vec!["4", "5"]);
    }

    #[test]
    fn tail_clamps_to_len() {
        let input = lines("1\n2\n");
        assert_eq!(tail(&input, 10), vec!["1", "2"]);
    }

    #[test]
    fn empty_inputs() {
        let empty: Vec<&str> = Vec::new();
        assert_eq!(number(&empty), "");
        assert!(head(&empty, 5).is_empty());
        assert!(tail(&empty, 5).is_empty());
    }
}
