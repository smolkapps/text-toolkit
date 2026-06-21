//! Line sorting: lexical or numeric, ascending or descending, optional unique,
//! optional key extraction by delimited field.

use std::cmp::Ordering;

/// Options for [`sort`].
#[derive(Debug, Clone, Default)]
pub struct Options {
    /// Compare lines as numbers (leading numeric value) rather than lexically.
    pub numeric: bool,
    /// Reverse the final ordering.
    pub reverse: bool,
    /// Drop duplicate lines after sorting (based on the full line text).
    pub unique: bool,
    /// 1-based field index to use as the sort key. `None` uses the whole line.
    pub by_field: Option<usize>,
    /// Field delimiter when `by_field` is set. Defaults to a single space.
    pub delim: char,
}

/// Extract the sort key for a line, honoring `by_field`/`delim`.
///
/// Fields are 1-based. If the requested field does not exist, the key is the
/// empty string (so short lines sort before longer ones / numerically as 0).
fn key_for<'a>(line: &'a str, opts: &Options) -> &'a str {
    match opts.by_field {
        Some(n) if n >= 1 => line.split(opts.delim).nth(n - 1).unwrap_or(""),
        _ => line,
    }
}

/// Parse the leading numeric portion of `s` for numeric comparison.
///
/// Skips leading whitespace, then parses an optional sign and a run that
/// `f64::from_str` accepts. Non-numeric or empty keys parse as 0.0 so they sort
/// consistently at the bottom of the numeric range. This is a pragmatic
/// approximation of GNU `sort -n` (which treats non-numeric leading text as 0).
fn parse_num(s: &str) -> f64 {
    let t = s.trim_start();
    // Find the longest prefix that parses as f64 by trying progressively.
    // Cheap approach: take chars while they could belong to a number.
    let mut end = 0;
    let bytes = t.as_bytes();
    let mut seen_dot = false;
    let mut seen_e = false;
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i] as char;
        let ok = match c {
            '0'..='9' => true,
            '+' | '-' => i == 0 || (seen_e && matches!(bytes[i - 1] as char, 'e' | 'E')),
            '.' if !seen_dot && !seen_e => {
                seen_dot = true;
                true
            }
            'e' | 'E' if !seen_e && i > 0 => {
                seen_e = true;
                true
            }
            _ => false,
        };
        if !ok {
            break;
        }
        end = i + 1;
        i += 1;
    }
    t[..end].parse::<f64>().unwrap_or(0.0)
}

/// Sort `lines` according to `opts`, returning a new owned vector.
///
/// Sorting is stable, so equal keys keep their original relative order before
/// any `reverse` is applied. `unique` removes adjacent equal *lines* after the
/// sort (matching `sort -u`, where uniqueness is on the whole line).
pub fn sort<'a>(lines: &[&'a str], opts: &Options) -> Vec<&'a str> {
    let mut v: Vec<&'a str> = lines.to_vec();

    v.sort_by(|a, b| {
        let ka = key_for(a, opts);
        let kb = key_for(b, opts);
        let ord = if opts.numeric {
            parse_num(ka)
                .partial_cmp(&parse_num(kb))
                .unwrap_or(Ordering::Equal)
        } else {
            ka.cmp(kb)
        };
        if opts.reverse {
            ord.reverse()
        } else {
            ord
        }
    });

    if opts.unique {
        v.dedup();
    }

    v
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(s: &str) -> Vec<&str> {
        crate::split_lines(s)
    }

    fn opts() -> Options {
        Options {
            delim: ' ',
            ..Default::default()
        }
    }

    #[test]
    fn lexical_ascending() {
        let input = lines("banana\napple\ncherry\n");
        let out = sort(&input, &opts());
        assert_eq!(out, vec!["apple", "banana", "cherry"]);
    }

    #[test]
    fn lexical_vs_numeric_differ() {
        // Lexically "10" < "9"; numerically 9 < 10.
        let input = lines("10\n9\n100\n2\n");
        let lex = sort(&input, &opts());
        assert_eq!(lex, vec!["10", "100", "2", "9"]);

        let num = sort(
            &input,
            &Options {
                numeric: true,
                ..opts()
            },
        );
        assert_eq!(num, vec!["2", "9", "10", "100"]);
    }

    #[test]
    fn reverse_numeric() {
        let input = lines("3\n1\n2\n");
        let out = sort(
            &input,
            &Options {
                numeric: true,
                reverse: true,
                ..opts()
            },
        );
        assert_eq!(out, vec!["3", "2", "1"]);
    }

    #[test]
    fn unique_removes_duplicates() {
        let input = lines("b\na\nb\na\nc\n");
        let out = sort(
            &input,
            &Options {
                unique: true,
                ..opts()
            },
        );
        assert_eq!(out, vec!["a", "b", "c"]);
    }

    #[test]
    fn by_field_numeric() {
        // Sort by the 2nd whitespace-delimited field, numerically.
        let input = lines("alice 30\nbob 5\ncarol 12\n");
        let out = sort(
            &input,
            &Options {
                numeric: true,
                by_field: Some(2),
                delim: ' ',
                ..Default::default()
            },
        );
        assert_eq!(out, vec!["bob 5", "carol 12", "alice 30"]);
    }

    #[test]
    fn by_field_custom_delim_lexical() {
        // CSV-ish: sort by 2nd comma field lexically.
        let input = lines("x,charlie\ny,alpha\nz,bravo\n");
        let out = sort(
            &input,
            &Options {
                by_field: Some(2),
                delim: ',',
                ..Default::default()
            },
        );
        assert_eq!(out, vec!["y,alpha", "z,bravo", "x,charlie"]);
    }

    #[test]
    fn missing_field_sorts_as_empty() {
        let input = lines("a b\nc\nd e\n");
        let out = sort(
            &input,
            &Options {
                by_field: Some(2),
                delim: ' ',
                ..Default::default()
            },
        );
        // "c" has no 2nd field -> empty key sorts first.
        assert_eq!(out[0], "c");
    }

    #[test]
    fn numeric_handles_negative_and_float() {
        let input = lines("-3\n2.5\n-1\n0\n");
        let out = sort(
            &input,
            &Options {
                numeric: true,
                ..opts()
            },
        );
        assert_eq!(out, vec!["-3", "-1", "0", "2.5"]);
    }

    #[test]
    fn parse_num_extracts_leading_number() {
        assert_eq!(parse_num("42abc"), 42.0);
        assert_eq!(parse_num("  -7.5 tail"), -7.5);
        assert_eq!(parse_num("notanumber"), 0.0);
        assert_eq!(parse_num("1e3x"), 1000.0);
    }
}
