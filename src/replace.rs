//! Regex search-and-replace over the whole input.

use regex::Regex;

/// Replace all non-overlapping matches of `pattern` in `text` with
/// `replacement`, returning the new string.
///
/// The replacement string supports the standard `regex` crate substitution
/// syntax: `$1` / `${name}` reference capture groups, and `$$` is a literal
/// `$`. The pattern is applied across the entire input including newlines.
///
/// Returns an error if `pattern` is not a valid regular expression.
pub fn replace(text: &str, pattern: &str, replacement: &str) -> anyhow::Result<String> {
    let re = Regex::new(pattern)?;
    Ok(re.replace_all(text, replacement).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literal_replace() {
        assert_eq!(replace("foo bar foo", "foo", "baz").unwrap(), "baz bar baz");
    }

    #[test]
    fn capture_group_reorder() {
        // Swap "first last" -> "last, first" using numbered captures.
        let real = replace("Alice Smith", r"(\w+) (\w+)", "$2, $1").unwrap();
        assert_eq!(real, "Smith, Alice");
    }

    #[test]
    fn named_capture_group() {
        let real = replace(
            "2026-06-21",
            r"(?P<y>\d{4})-(?P<m>\d{2})-(?P<d>\d{2})",
            "$m/$d/$y",
        )
        .unwrap();
        assert_eq!(real, "06/21/2026");
    }

    #[test]
    fn replace_across_newlines() {
        let out = replace("a\nb\na", "a", "X").unwrap();
        assert_eq!(out, "X\nb\nX");
    }

    #[test]
    fn no_match_returns_unchanged() {
        assert_eq!(replace("hello", "zzz", "q").unwrap(), "hello");
    }

    #[test]
    fn invalid_regex_errors() {
        assert!(replace("x", "(", "y").is_err());
    }

    #[test]
    fn digits_to_hash() {
        let out = replace("abc123def456", r"\d+", "#").unwrap();
        assert_eq!(out, "abc#def#");
    }
}
