//! Word- and character-frequency counting.

use std::collections::HashMap;

/// Options for [`frequencies`].
#[derive(Debug, Clone, Copy, Default)]
pub struct Options {
    /// Count individual characters instead of words.
    pub chars: bool,
    /// Fold case before counting (so "The" and "the" combine).
    pub ignore_case: bool,
    /// Keep only the top N most frequent items. `None` = keep all.
    pub top: Option<usize>,
}

/// A single frequency entry: the item and its count.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub item: String,
    pub count: usize,
}

/// Tokenize for word-frequency: split on whitespace, then trim surrounding
/// ASCII punctuation so "world." and "world" count together. Empty tokens are
/// dropped.
fn words(text: &str) -> impl Iterator<Item = &str> {
    text.split_whitespace()
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()))
        .filter(|w| !w.is_empty())
}

/// Compute frequencies over `text`.
///
/// Ordering is deterministic: primarily by descending count, and ties are
/// broken by the item's ascending lexical order. This makes top-N output
/// stable and reproducible across runs (HashMap iteration order is not).
pub fn frequencies(text: &str, opts: Options) -> Vec<Entry> {
    let mut counts: HashMap<String, usize> = HashMap::new();

    let fold = |s: &str| -> String {
        if opts.ignore_case {
            s.to_lowercase()
        } else {
            s.to_string()
        }
    };

    if opts.chars {
        for ch in text.chars() {
            // Skip whitespace so char-frequency focuses on visible content.
            if ch.is_whitespace() {
                continue;
            }
            *counts.entry(fold(&ch.to_string())).or_insert(0) += 1;
        }
    } else {
        for w in words(text) {
            *counts.entry(fold(w)).or_insert(0) += 1;
        }
    }

    let mut entries: Vec<Entry> = counts
        .into_iter()
        .map(|(item, count)| Entry { item, count })
        .collect();

    entries.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.item.cmp(&b.item)));

    if let Some(n) = opts.top {
        entries.truncate(n);
    }

    entries
}

/// Render entries as `count\titem` lines (tab-separated), one per line.
pub fn format(entries: &[Entry]) -> String {
    let mut out = String::new();
    for e in entries {
        out.push_str(&e.count.to_string());
        out.push('\t');
        out.push_str(&e.item);
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn word_counts_basic() {
        let e = frequencies("a b a c a b", Options::default());
        assert_eq!(
            e[0],
            Entry {
                item: "a".into(),
                count: 3
            }
        );
        assert_eq!(
            e[1],
            Entry {
                item: "b".into(),
                count: 2
            }
        );
        assert_eq!(
            e[2],
            Entry {
                item: "c".into(),
                count: 1
            }
        );
    }

    #[test]
    fn tie_break_is_lexical() {
        // 'b' and 'c' both appear twice -> lexical order puts b before c.
        let e = frequencies("c b c b a", Options::default());
        assert_eq!(e[0].item, "b");
        assert_eq!(e[1].item, "c");
        assert_eq!(e[2].item, "a");
    }

    #[test]
    fn top_n_truncates() {
        let e = frequencies(
            "a a a b b c",
            Options {
                top: Some(2),
                ..Default::default()
            },
        );
        assert_eq!(e.len(), 2);
        assert_eq!(e[0].item, "a");
        assert_eq!(e[1].item, "b");
    }

    #[test]
    fn ignore_case_combines() {
        let e = frequencies(
            "The the THE cat",
            Options {
                ignore_case: true,
                ..Default::default()
            },
        );
        assert_eq!(
            e[0],
            Entry {
                item: "the".into(),
                count: 3
            }
        );
    }

    #[test]
    fn case_sensitive_separates() {
        let e = frequencies("The the", Options::default());
        // Two distinct items, each count 1, lexical order: "The" < "the".
        assert_eq!(e.len(), 2);
        assert_eq!(e[0].item, "The");
        assert_eq!(e[1].item, "the");
    }

    #[test]
    fn strips_punctuation() {
        let e = frequencies("world. world, world!", Options::default());
        assert_eq!(e.len(), 1);
        assert_eq!(
            e[0],
            Entry {
                item: "world".into(),
                count: 3
            }
        );
    }

    #[test]
    fn char_frequency_skips_whitespace() {
        let e = frequencies(
            "aab b",
            Options {
                chars: true,
                ..Default::default()
            },
        );
        assert_eq!(
            e[0],
            Entry {
                item: "a".into(),
                count: 2
            }
        );
        assert_eq!(
            e[1],
            Entry {
                item: "b".into(),
                count: 2
            }
        );
        // No whitespace entry.
        assert!(e.iter().all(|x| x.item != " "));
    }

    #[test]
    fn format_is_count_tab_item() {
        let e = vec![Entry {
            item: "x".into(),
            count: 5,
        }];
        assert_eq!(format(&e), "5\tx\n");
    }
}
