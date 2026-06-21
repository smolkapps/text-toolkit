//! text-toolkit core library.
//!
//! Each operation is implemented as a pure function that takes input text
//! (`&str`) plus an options struct and returns an owned `String` (or a count).
//! The `main.rs` binary is a thin clap wrapper over these functions, so all
//! behavior is unit-testable without spawning a process.

pub mod casing;
pub mod dedup;
pub mod freq;
pub mod number;
pub mod replace;
pub mod sort;
pub mod trim;
pub mod wc;
pub mod wrap;

/// Split text into logical lines for line-oriented operations.
///
/// We split on `\n` and strip a single trailing `\r` from each line so that
/// CRLF input is handled. A trailing newline at the end of the input does NOT
/// produce a spurious empty final line (i.e. `"a\nb\n"` -> `["a", "b"]`), which
/// matches the intuitive line count and avoids emitting a blank line on output.
/// Input without a trailing newline (`"a\nb"`) also yields `["a", "b"]`.
pub fn split_lines(text: &str) -> Vec<&str> {
    if text.is_empty() {
        return Vec::new();
    }
    let trimmed = text.strip_suffix('\n').unwrap_or(text);
    // After stripping the final '\n', an input that was exactly "\n" becomes ""
    // which should be a single empty line, so guard the empty case explicitly.
    if trimmed.is_empty() {
        return vec![""];
    }
    trimmed
        .split('\n')
        .map(|l| l.strip_suffix('\r').unwrap_or(l))
        .collect()
}

/// Join lines back into a single string with `\n` separators and a trailing
/// newline. Empty input produces empty output (no stray newline).
pub fn join_lines<I, S>(lines: I) -> String
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut out = String::new();
    let mut any = false;
    for line in lines {
        out.push_str(line.as_ref());
        out.push('\n');
        any = true;
    }
    if !any {
        return String::new();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_lines_basic() {
        assert_eq!(split_lines("a\nb\nc"), vec!["a", "b", "c"]);
    }

    #[test]
    fn split_lines_trailing_newline_no_empty() {
        assert_eq!(split_lines("a\nb\n"), vec!["a", "b"]);
    }

    #[test]
    fn split_lines_crlf() {
        assert_eq!(split_lines("a\r\nb\r\n"), vec!["a", "b"]);
    }

    #[test]
    fn split_lines_empty_input() {
        assert!(split_lines("").is_empty());
    }

    #[test]
    fn split_lines_single_newline_is_one_blank() {
        assert_eq!(split_lines("\n"), vec![""]);
    }

    #[test]
    fn split_lines_internal_blanks_preserved() {
        assert_eq!(split_lines("a\n\nb\n"), vec!["a", "", "b"]);
    }

    #[test]
    fn join_round_trip() {
        let lines = split_lines("a\nb\nc\n");
        assert_eq!(join_lines(lines), "a\nb\nc\n");
    }

    #[test]
    fn join_empty_is_empty() {
        let empty: Vec<&str> = Vec::new();
        assert_eq!(join_lines(empty), "");
    }
}
