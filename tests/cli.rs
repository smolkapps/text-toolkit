//! End-to-end integration tests for the `text-toolkit` binary.
//!
//! These exercise the real compiled CLI: argument parsing, stdin piping,
//! file/`-o` I/O, and a couple of full subcommand round-trips. The fine-grained
//! algorithmic correctness lives in the per-module unit tests; here we just
//! confirm the wiring works through an actual process.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

fn bin() -> Command {
    Command::cargo_bin("text-toolkit").expect("binary builds")
}

#[test]
fn wc_via_stdin_default_counts() {
    // "a b\nc d e\n" -> 2 lines, 5 words, 10 bytes ("a b\n"=4 + "c d e\n"=6).
    bin()
        .arg("wc")
        .write_stdin("a b\nc d e\n")
        .assert()
        .success()
        .stdout("2 5 10\n");
}

#[test]
fn wc_select_words_only() {
    bin()
        .args(["wc", "--words"])
        .write_stdin("one two three\n")
        .assert()
        .success()
        .stdout("3\n");
}

#[test]
fn wc_pattern_count() {
    bin()
        .args(["wc", "--pattern", "ba"])
        .write_stdin("bar baz bat\n")
        .assert()
        .success()
        .stdout("3\n");
}

#[test]
fn dedup_global_via_stdin() {
    bin()
        .arg("dedup")
        .write_stdin("a\nb\na\nc\nb\n")
        .assert()
        .success()
        .stdout("a\nb\nc\n");
}

#[test]
fn dedup_adjacent_keeps_nonadjacent() {
    bin()
        .args(["dedup", "--adjacent"])
        .write_stdin("a\nb\na\nc\nb\n")
        .assert()
        .success()
        .stdout("a\nb\na\nc\nb\n");
}

#[test]
fn dedup_ignore_case() {
    bin()
        .args(["dedup", "--ignore-case"])
        .write_stdin("Apple\napple\nBanana\n")
        .assert()
        .success()
        .stdout("Apple\nBanana\n");
}

#[test]
fn sort_numeric_via_stdin() {
    bin()
        .args(["sort", "--numeric"])
        .write_stdin("10\n2\n100\n9\n")
        .assert()
        .success()
        .stdout("2\n9\n10\n100\n");
}

#[test]
fn sort_lexical_default() {
    bin()
        .arg("sort")
        .write_stdin("banana\napple\ncherry\n")
        .assert()
        .success()
        .stdout("apple\nbanana\ncherry\n");
}

#[test]
fn sort_reverse_unique() {
    bin()
        .args(["sort", "--reverse", "--unique"])
        .write_stdin("b\na\nb\nc\na\n")
        .assert()
        .success()
        .stdout("c\nb\na\n");
}

#[test]
fn sort_by_field_with_delim() {
    bin()
        .args(["sort", "--numeric", "--by-field", "2", "--delim", ","])
        .write_stdin("alice,30\nbob,5\ncarol,12\n")
        .assert()
        .success()
        .stdout("bob,5\ncarol,12\nalice,30\n");
}

#[test]
fn case_upper_and_title() {
    bin()
        .args(["case", "--upper"])
        .write_stdin("hello world\n")
        .assert()
        .success()
        .stdout("HELLO WORLD\n");

    bin()
        .args(["case", "--title"])
        .write_stdin("the quick brown fox\n")
        .assert()
        .success()
        .stdout("The Quick Brown Fox\n");
}

#[test]
fn freq_top_n_ordering() {
    // a:3, b:2, c:1 -> top 2 = a then b.
    bin()
        .args(["freq", "--top", "2"])
        .write_stdin("a a a b b c\n")
        .assert()
        .success()
        .stdout("3\ta\n2\tb\n");
}

#[test]
fn freq_ignore_case() {
    bin()
        .args(["freq", "--ignore-case", "--top", "1"])
        .write_stdin("The the THE cat\n")
        .assert()
        .success()
        .stdout("3\tthe\n");
}

#[test]
fn wrap_respects_width() {
    let out = bin()
        .args(["wrap", "--width", "12"])
        .write_stdin("the quick brown fox jumps\n")
        .assert()
        .success();
    // No output line should exceed 12 chars.
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    for line in stdout.lines() {
        assert!(line.chars().count() <= 12, "line too long: {line:?}");
    }
    // And every word survives intact.
    for w in ["the", "quick", "brown", "fox", "jumps"] {
        assert!(stdout.split_whitespace().any(|t| t == w));
    }
}

#[test]
fn replace_with_capture_group() {
    bin()
        .args(["replace", "--pattern", r"(\w+) (\w+)", "--with", "$2, $1"])
        .write_stdin("Alice Smith\n")
        .assert()
        .success()
        .stdout("Smith, Alice\n");
}

#[test]
fn trim_normalizes_blank_lines_and_trailing_ws() {
    bin()
        .arg("trim")
        .write_stdin("\n\nhello   \n\n\nworld\t\n\n")
        .assert()
        .success()
        .stdout("hello\n\nworld\n");
}

#[test]
fn number_prefixes_lines() {
    bin()
        .arg("number")
        .write_stdin("a\nb\nc\n")
        .assert()
        .success()
        .stdout("1\ta\n2\tb\n3\tc\n");
}

#[test]
fn head_and_tail() {
    bin()
        .args(["head", "-n", "2"])
        .write_stdin("1\n2\n3\n4\n5\n")
        .assert()
        .success()
        .stdout("1\n2\n");

    bin()
        .args(["tail", "-n", "2"])
        .write_stdin("1\n2\n3\n4\n5\n")
        .assert()
        .success()
        .stdout("4\n5\n");
}

#[test]
fn reads_from_file_and_writes_to_output() {
    let dir = tempdir();
    let input = dir.join("in.txt");
    let output = dir.join("out.txt");
    fs::write(&input, "banana\napple\ncherry\n").unwrap();

    bin()
        .arg("sort")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .assert()
        .success()
        .stdout(predicate::str::is_empty());

    let written = fs::read_to_string(&output).unwrap();
    assert_eq!(written, "apple\nbanana\ncherry\n");
}

#[test]
fn missing_file_errors_nonzero() {
    bin()
        .arg("wc")
        .arg("/nonexistent/path/should/not/exist.txt")
        .assert()
        .failure()
        .stderr(predicate::str::contains("reading"));
}

#[test]
fn invalid_regex_errors_nonzero() {
    bin()
        .args(["replace", "--pattern", "(", "--with", "x"])
        .write_stdin("hello\n")
        .assert()
        .failure();
}

/// Create a unique temporary directory without pulling in an extra crate.
fn tempdir() -> std::path::PathBuf {
    let mut p = std::env::temp_dir();
    let unique = format!(
        "text-toolkit-test-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    p.push(unique);
    fs::create_dir_all(&p).unwrap();
    p
}
