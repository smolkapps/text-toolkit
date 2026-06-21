//! Word-wrapping text to a maximum column width.

/// Word-wrap `text` so that no output line exceeds `width` columns, never
/// splitting a word across lines. Width is measured in Unicode scalar values
/// (chars), which is a reasonable approximation for typical prose.
///
/// Behavior:
/// - Each input line (separated by `\n`) is wrapped independently, so existing
///   hard line breaks are preserved as paragraph boundaries.
/// - Words within a line are separated by runs of whitespace; on output, words
///   are joined with a single space.
/// - A word longer than `width` is placed on its own line and left intact
///   (it overflows rather than being broken).
/// - Blank input lines are preserved as blank output lines.
/// - A `width` of 0 is treated as "no wrapping" for safety (every word still
///   goes on its own line would be useless), so we clamp to at least 1.
pub fn wrap(text: &str, width: usize) -> String {
    let width = width.max(1);
    let mut out_lines: Vec<String> = Vec::new();

    for raw_line in text.split('\n') {
        let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
        let words: Vec<&str> = line.split_whitespace().collect();

        if words.is_empty() {
            // Preserve blank/whitespace-only lines as a single blank line.
            out_lines.push(String::new());
            continue;
        }

        let mut current = String::new();
        for word in words {
            if current.is_empty() {
                current.push_str(word);
            } else if current.chars().count() + 1 + word.chars().count() <= width {
                current.push(' ');
                current.push_str(word);
            } else {
                out_lines.push(std::mem::take(&mut current));
                current.push_str(word);
            }
        }
        if !current.is_empty() {
            out_lines.push(current);
        }
    }

    let mut s = out_lines.join("\n");
    if !s.is_empty() {
        s.push('\n');
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn max_width(s: &str) -> usize {
        s.lines().map(|l| l.chars().count()).max().unwrap_or(0)
    }

    #[test]
    fn wraps_at_width() {
        let input = "the quick brown fox jumps over the lazy dog";
        let out = wrap(input, 15);
        for line in out.lines() {
            assert!(line.chars().count() <= 15, "line too long: {line:?}");
        }
    }

    #[test]
    fn never_splits_words() {
        let input = "alpha bravo charlie delta";
        let out = wrap(input, 10);
        // Every original word must appear intact as a whitespace-bounded token.
        for w in ["alpha", "bravo", "charlie", "delta"] {
            assert!(
                out.split_whitespace().any(|t| t == w),
                "word {w} was split or lost"
            );
        }
    }

    #[test]
    fn long_word_overflows_on_own_line() {
        let input = "hi supercalifragilistic bye";
        let out = wrap(input, 5);
        let lines: Vec<&str> = out.lines().collect();
        assert!(lines.contains(&"supercalifragilistic"));
        assert!(lines.contains(&"hi"));
        assert!(lines.contains(&"bye"));
    }

    #[test]
    fn exact_fit_packs_words() {
        // "ab cd" is exactly 5 chars, should stay on one line at width 5.
        assert_eq!(wrap("ab cd", 5), "ab cd\n");
        // width 4 forces a break.
        assert_eq!(wrap("ab cd", 4), "ab\ncd\n");
    }

    #[test]
    fn preserves_paragraph_breaks() {
        let input = "one two\n\nthree four";
        let out = wrap(input, 80);
        assert_eq!(out, "one two\n\nthree four\n");
    }

    #[test]
    fn collapses_internal_whitespace() {
        let out = wrap("a    b\tc", 80);
        assert_eq!(out, "a b c\n");
    }

    #[test]
    fn width_respected_unless_single_word_too_long() {
        // The only way a line may exceed `width` is when it consists of a single
        // word that is itself longer than `width` (long words overflow rather
        // than being split). Any over-width line that contains a space would be
        // a packing bug. Note "consectetur" is 11 chars, so width 10 *must*
        // overflow on that token alone — which is allowed.
        let prose = "Lorem ipsum dolor sit amet consectetur adipiscing elit sed do";
        for w in [10, 20, 40] {
            let out = wrap(prose, w);
            for line in out.lines() {
                if line.chars().count() > w {
                    assert!(
                        !line.contains(' '),
                        "width {w}: over-width line packed multiple words: {line:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn width_respected_when_all_words_fit() {
        // When every word fits within the width, the width is never exceeded.
        let prose = "the cat sat on a mat and ran far away today now";
        for w in [6, 12, 20] {
            let out = wrap(prose, w);
            assert!(max_width(&out) <= w, "width {w} exceeded with short words");
        }
    }

    #[test]
    fn empty_input_empty_output() {
        assert_eq!(wrap("", 80), "");
    }
}
