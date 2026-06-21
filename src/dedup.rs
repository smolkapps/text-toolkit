//! Remove duplicate lines, globally or only adjacent duplicates.

use std::collections::HashSet;

/// Options for [`dedup`].
#[derive(Debug, Clone, Copy, Default)]
pub struct Options {
    /// Only collapse runs of identical *adjacent* lines (like `uniq`).
    /// When false (default), removes all later duplicates anywhere in the input.
    pub adjacent: bool,
    /// Compare lines case-insensitively (ASCII + Unicode via `to_lowercase`).
    pub ignore_case: bool,
}

/// Compute the comparison key for a line given the options.
fn key(line: &str, ignore_case: bool) -> String {
    if ignore_case {
        line.to_lowercase()
    } else {
        line.to_string()
    }
}

/// Deduplicate `lines`, preserving the first occurrence and original order.
///
/// - Global (default): the first time a line is seen it is kept; every later
///   line whose key matches an already-seen key is dropped, no matter how far
///   apart they are.
/// - Adjacent (`opts.adjacent`): only consecutive duplicate lines collapse to
///   one, so non-adjacent repeats survive.
pub fn dedup<'a>(lines: &[&'a str], opts: Options) -> Vec<&'a str> {
    let mut out: Vec<&'a str> = Vec::with_capacity(lines.len());

    if opts.adjacent {
        let mut prev: Option<String> = None;
        for &line in lines {
            let k = key(line, opts.ignore_case);
            if prev.as_deref() != Some(k.as_str()) {
                out.push(line);
                prev = Some(k);
            }
        }
    } else {
        let mut seen: HashSet<String> = HashSet::new();
        for &line in lines {
            let k = key(line, opts.ignore_case);
            if seen.insert(k) {
                out.push(line);
            }
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(s: &str) -> Vec<&str> {
        crate::split_lines(s)
    }

    #[test]
    fn global_removes_nonadjacent_duplicates() {
        let input = lines("a\nb\na\nc\nb\n");
        let out = dedup(&input, Options::default());
        assert_eq!(out, vec!["a", "b", "c"]);
    }

    #[test]
    fn adjacent_keeps_nonadjacent_duplicates() {
        let input = lines("a\nb\na\nc\nb\n");
        let out = dedup(
            &input,
            Options {
                adjacent: true,
                ..Default::default()
            },
        );
        // 'a' and 'b' reappear non-adjacently, so they are kept.
        assert_eq!(out, vec!["a", "b", "a", "c", "b"]);
    }

    #[test]
    fn adjacent_collapses_runs() {
        let input = lines("a\na\na\nb\nb\nc\n");
        let out = dedup(
            &input,
            Options {
                adjacent: true,
                ..Default::default()
            },
        );
        assert_eq!(out, vec!["a", "b", "c"]);
    }

    #[test]
    fn ignore_case_global() {
        let input = lines("Apple\napple\nBANANA\nbanana\n");
        let out = dedup(
            &input,
            Options {
                ignore_case: true,
                ..Default::default()
            },
        );
        // First-seen casing preserved.
        assert_eq!(out, vec!["Apple", "BANANA"]);
    }

    #[test]
    fn ignore_case_adjacent() {
        let input = lines("Foo\nfoo\nFOO\nbar\n");
        let out = dedup(
            &input,
            Options {
                adjacent: true,
                ignore_case: true,
            },
        );
        assert_eq!(out, vec!["Foo", "bar"]);
    }

    #[test]
    fn case_sensitive_keeps_differing_case() {
        let input = lines("a\nA\na\n");
        let out = dedup(&input, Options::default());
        assert_eq!(out, vec!["a", "A"]);
    }

    #[test]
    fn empty_input() {
        let input: Vec<&str> = Vec::new();
        assert!(dedup(&input, Options::default()).is_empty());
    }
}
