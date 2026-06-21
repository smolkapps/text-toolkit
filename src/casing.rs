//! Case transformations over text.

/// The case transformation to apply.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Everything uppercase.
    Upper,
    /// Everything lowercase.
    Lower,
    /// Title Case: first letter of each whitespace-separated word uppercased,
    /// the rest of the word lowercased.
    Title,
    /// Sentence case: first letter of each sentence uppercased, the rest
    /// lowercased. Sentences end at `.`, `!`, or `?`.
    Sentence,
}

/// Apply the case transformation to `text`, preserving all non-letter
/// characters (whitespace, punctuation, newlines) exactly.
pub fn convert(text: &str, mode: Mode) -> String {
    match mode {
        Mode::Upper => text.to_uppercase(),
        Mode::Lower => text.to_lowercase(),
        Mode::Title => title_case(text),
        Mode::Sentence => sentence_case(text),
    }
}

/// Title-case: uppercase the first alphabetic char of each run of
/// alphanumeric characters, lowercase the rest. Word boundaries are any
/// non-alphanumeric character, so "o'brien-smith" -> "O'Brien-Smith".
fn title_case(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut at_word_start = true;
    for ch in text.chars() {
        if ch.is_alphanumeric() {
            if at_word_start {
                out.extend(ch.to_uppercase());
            } else {
                out.extend(ch.to_lowercase());
            }
            at_word_start = false;
        } else {
            out.push(ch);
            at_word_start = true;
        }
    }
    out
}

/// Sentence-case: uppercase the first alphabetic character of each sentence,
/// lowercase everything else. A new sentence begins at the start of input and
/// after a `.`, `!`, or `?` (optionally followed by whitespace/quotes).
fn sentence_case(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    // `start_pending` means "the next alphabetic char begins a sentence".
    let mut start_pending = true;
    for ch in text.chars() {
        if ch.is_alphabetic() {
            if start_pending {
                out.extend(ch.to_uppercase());
                start_pending = false;
            } else {
                out.extend(ch.to_lowercase());
            }
        } else {
            out.push(ch);
            if matches!(ch, '.' | '!' | '?') {
                start_pending = true;
            }
            // Digits do not start a sentence themselves, but they also should
            // not consume a pending start: keep start_pending as-is so the
            // first *letter* after a terminator still gets capitalized.
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upper_lower() {
        assert_eq!(convert("Hello World", Mode::Upper), "HELLO WORLD");
        assert_eq!(convert("Hello World", Mode::Lower), "hello world");
    }

    #[test]
    fn title_basic() {
        assert_eq!(
            convert("the quick brown fox", Mode::Title),
            "The Quick Brown Fox"
        );
    }

    #[test]
    fn title_lowercases_rest_of_word() {
        assert_eq!(convert("hELLO wORLD", Mode::Title), "Hello World");
    }

    #[test]
    fn title_handles_punctuation_boundaries() {
        assert_eq!(convert("o'brien-smith", Mode::Title), "O'Brien-Smith");
    }

    #[test]
    fn title_preserves_whitespace_runs() {
        assert_eq!(convert("a  b\tc", Mode::Title), "A  B\tC");
    }

    #[test]
    fn sentence_basic() {
        assert_eq!(
            convert("hello world. goodbye world.", Mode::Sentence),
            "Hello world. Goodbye world."
        );
    }

    #[test]
    fn sentence_multiple_terminators() {
        assert_eq!(
            convert("really? yes! ok then.", Mode::Sentence),
            "Really? Yes! Ok then."
        );
    }

    #[test]
    fn sentence_lowercases_interior_caps() {
        assert_eq!(
            convert("THE CAT SAT. THE DOG RAN.", Mode::Sentence),
            "The cat sat. The dog ran."
        );
    }

    #[test]
    fn sentence_leading_number_then_letter() {
        // "3 apples..." — first letter to capitalize is 'a'.
        assert_eq!(convert("3 apples fell.", Mode::Sentence), "3 Apples fell.");
    }

    #[test]
    fn sentence_abbrev_edge_known_limitation() {
        // KNOWN LIMITATION (asserted so the behavior is intentional, not a
        // surprise): sentence-case treats every '.' as a sentence terminator,
        // so an abbreviation like "u.s." is mangled. Tracing "the u.s. army":
        //   "the " -> "The " (start-of-input cap), the space does NOT reset
        //   sentence state; 'u' is therefore mid-sentence and lowercases; the
        //   '.' after it arms the next-capital, so 's' -> 'S'; the '.' after
        //   that arms again, so 'a' -> 'A'. Result: "The u.S. Army".
        // Proper abbreviation handling would need a dictionary; out of scope.
        assert_eq!(convert("the u.s. army", Mode::Sentence), "The u.S. Army");
    }
}
