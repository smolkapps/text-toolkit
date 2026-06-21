//! text-toolkit: a thin clap front-end over the `text_toolkit` library.
//!
//! Every subcommand reads from a positional `[FILE]` argument or, when that is
//! omitted, from standard input. Output goes to the path given with `-o/--output`
//! or to standard output. All real logic lives in the library so it is unit
//! tested independently of argument parsing and I/O.

use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};

use text_toolkit::{
    casing, dedup, freq, join_lines, number as numbering, replace, sort, split_lines, trim, wc,
    wrap,
};

#[derive(Parser)]
#[command(
    name = "text-toolkit",
    version,
    about = "Common text operations: wc, dedup, sort, case, freq, wrap, trim, number, head/tail, replace.",
    propagate_version = true
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

/// Shared input/output options every subcommand accepts.
#[derive(Args, Clone)]
struct Io {
    /// Input file to read. If omitted, reads from standard input.
    #[arg(value_name = "FILE")]
    file: Option<PathBuf>,

    /// Write output to this file instead of standard output.
    #[arg(short = 'o', long = "output", value_name = "FILE")]
    output: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Command {
    /// Count lines, words, characters, and bytes (or count regex matches).
    Wc(WcArgs),
    /// Remove duplicate lines.
    Dedup(DedupArgs),
    /// Sort lines lexically or numerically.
    Sort(SortArgs),
    /// Change the letter case of the text.
    Case(CaseArgs),
    /// Show word or character frequencies.
    Freq(FreqArgs),
    /// Word-wrap text to a maximum width.
    Wrap(WrapArgs),
    /// Strip trailing whitespace and normalize blank lines.
    Trim(TrimArgs),
    /// Prefix each line with its line number.
    Number(NumberArgs),
    /// Print the first N lines.
    Head(HeadArgs),
    /// Print the last N lines.
    Tail(TailArgs),
    /// Regex search and replace.
    Replace(ReplaceArgs),
}

#[derive(Args)]
struct WcArgs {
    #[command(flatten)]
    io: Io,
    /// Count lines.
    #[arg(short = 'l', long)]
    lines: bool,
    /// Count words.
    #[arg(short = 'w', long)]
    words: bool,
    /// Count characters (Unicode scalar values).
    #[arg(short = 'c', long = "chars")]
    chars: bool,
    /// Count bytes.
    #[arg(short = 'b', long)]
    bytes: bool,
    /// Instead of the usual counts, count occurrences of this regex pattern.
    #[arg(long = "pattern", value_name = "REGEX")]
    pattern: Option<String>,
}

#[derive(Args)]
struct DedupArgs {
    #[command(flatten)]
    io: Io,
    /// Only collapse identical *adjacent* lines (like `uniq`).
    #[arg(long)]
    adjacent: bool,
    /// Compare lines case-insensitively.
    #[arg(short = 'i', long = "ignore-case")]
    ignore_case: bool,
}

#[derive(Args)]
struct SortArgs {
    #[command(flatten)]
    io: Io,
    /// Compare according to leading numeric value.
    #[arg(short = 'n', long)]
    numeric: bool,
    /// Reverse the result order.
    #[arg(short = 'r', long)]
    reverse: bool,
    /// Output only unique lines (after sorting).
    #[arg(short = 'u', long)]
    unique: bool,
    /// Sort by the Nth field (1-based) instead of the whole line.
    #[arg(long = "by-field", value_name = "N")]
    by_field: Option<usize>,
    /// Field delimiter used with --by-field (default: space).
    #[arg(long, value_name = "CHAR", default_value = " ")]
    delim: String,
}

#[derive(Args)]
struct CaseArgs {
    #[command(flatten)]
    io: Io,
    /// UPPERCASE everything.
    #[arg(long, group = "mode")]
    upper: bool,
    /// lowercase everything.
    #[arg(long, group = "mode")]
    lower: bool,
    /// Title Case Each Word.
    #[arg(long, group = "mode")]
    title: bool,
    /// Sentence case (capitalize the start of each sentence).
    #[arg(long, group = "mode")]
    sentence: bool,
}

#[derive(Args)]
struct FreqArgs {
    #[command(flatten)]
    io: Io,
    /// Count characters instead of words.
    #[arg(long = "chars")]
    chars: bool,
    /// Fold case before counting.
    #[arg(short = 'i', long = "ignore-case")]
    ignore_case: bool,
    /// Show only the top N most frequent items.
    #[arg(long = "top", value_name = "N")]
    top: Option<usize>,
}

#[derive(Args)]
struct WrapArgs {
    #[command(flatten)]
    io: Io,
    /// Maximum line width in characters.
    #[arg(long, default_value_t = 80)]
    width: usize,
}

#[derive(Args)]
struct TrimArgs {
    #[command(flatten)]
    io: Io,
}

#[derive(Args)]
struct NumberArgs {
    #[command(flatten)]
    io: Io,
}

#[derive(Args)]
struct HeadArgs {
    #[command(flatten)]
    io: Io,
    /// Number of lines to print.
    #[arg(short = 'n', long, default_value_t = 10)]
    lines: usize,
}

#[derive(Args)]
struct TailArgs {
    #[command(flatten)]
    io: Io,
    /// Number of lines to print.
    #[arg(short = 'n', long, default_value_t = 10)]
    lines: usize,
}

#[derive(Args)]
struct ReplaceArgs {
    #[command(flatten)]
    io: Io,
    /// Regular expression to search for.
    #[arg(long = "pattern", value_name = "REGEX")]
    pattern: String,
    /// Replacement string (supports $1, ${name} capture references).
    #[arg(long = "with", value_name = "STR")]
    with: String,
}

/// Read the full input for a subcommand: from the file if given, else stdin.
fn read_input(io: &Io) -> Result<String> {
    match &io.file {
        Some(path) => {
            fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))
        }
        None => {
            let mut buf = String::new();
            io::stdin()
                .read_to_string(&mut buf)
                .context("reading standard input")?;
            Ok(buf)
        }
    }
}

/// Write `content` to the output file if given, else to stdout.
fn write_output(io: &Io, content: &str) -> Result<()> {
    match &io.output {
        Some(path) => {
            fs::write(path, content).with_context(|| format!("writing {}", path.display()))
        }
        None => {
            let stdout = io::stdout();
            let mut handle = stdout.lock();
            handle
                .write_all(content.as_bytes())
                .context("writing standard output")?;
            Ok(())
        }
    }
}

/// Take a single delimiter character from a `--delim` string, erroring if the
/// user passes something other than exactly one character.
fn single_char(s: &str, flag: &str) -> Result<char> {
    let mut chars = s.chars();
    match (chars.next(), chars.next()) {
        (Some(c), None) => Ok(c),
        _ => anyhow::bail!("{flag} must be exactly one character (got {s:?})"),
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Wc(a) => {
            let text = read_input(&a.io)?;
            let out = if let Some(pat) = a.pattern {
                let n = wc::count_pattern(&text, &pat)?;
                format!("{n}\n")
            } else {
                let counts = wc::count(&text);
                let select = wc::Select {
                    lines: a.lines,
                    words: a.words,
                    chars: a.chars,
                    bytes: a.bytes,
                };
                format!("{}\n", wc::format(&counts, &select))
            };
            write_output(&a.io, &out)
        }

        Command::Dedup(a) => {
            let text = read_input(&a.io)?;
            let lines = split_lines(&text);
            let out = dedup::dedup(
                &lines,
                dedup::Options {
                    adjacent: a.adjacent,
                    ignore_case: a.ignore_case,
                },
            );
            write_output(&a.io, &join_lines(out))
        }

        Command::Sort(a) => {
            let text = read_input(&a.io)?;
            let lines = split_lines(&text);
            let delim = single_char(&a.delim, "--delim")?;
            let out = sort::sort(
                &lines,
                &sort::Options {
                    numeric: a.numeric,
                    reverse: a.reverse,
                    unique: a.unique,
                    by_field: a.by_field,
                    delim,
                },
            );
            write_output(&a.io, &join_lines(out))
        }

        Command::Case(a) => {
            let text = read_input(&a.io)?;
            let mode = if a.upper {
                casing::Mode::Upper
            } else if a.lower {
                casing::Mode::Lower
            } else if a.title {
                casing::Mode::Title
            } else if a.sentence {
                casing::Mode::Sentence
            } else {
                anyhow::bail!("choose one of --upper, --lower, --title, --sentence");
            };
            write_output(&a.io, &casing::convert(&text, mode))
        }

        Command::Freq(a) => {
            let text = read_input(&a.io)?;
            let entries = freq::frequencies(
                &text,
                freq::Options {
                    chars: a.chars,
                    ignore_case: a.ignore_case,
                    top: a.top,
                },
            );
            write_output(&a.io, &freq::format(&entries))
        }

        Command::Wrap(a) => {
            let text = read_input(&a.io)?;
            write_output(&a.io, &wrap::wrap(&text, a.width))
        }

        Command::Trim(a) => {
            let text = read_input(&a.io)?;
            write_output(&a.io, &trim::trim(&text))
        }

        Command::Number(a) => {
            let text = read_input(&a.io)?;
            let lines = split_lines(&text);
            write_output(&a.io, &numbering::number(&lines))
        }

        Command::Head(a) => {
            let text = read_input(&a.io)?;
            let lines = split_lines(&text);
            let out = numbering::head(&lines, a.lines);
            write_output(&a.io, &join_lines(out))
        }

        Command::Tail(a) => {
            let text = read_input(&a.io)?;
            let lines = split_lines(&text);
            let out = numbering::tail(&lines, a.lines);
            write_output(&a.io, &join_lines(out))
        }

        Command::Replace(a) => {
            let text = read_input(&a.io)?;
            let out = replace::replace(&text, &a.pattern, &a.with)?;
            write_output(&a.io, &out)
        }
    }
}

fn main() {
    if let Err(err) = run() {
        eprintln!("text-toolkit: {err:#}");
        std::process::exit(1);
    }
}
