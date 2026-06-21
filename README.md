# text-toolkit

A fast, dependency-light command-line toolkit for common text operations,
written in Rust. One binary, many subcommands — each reads a file (or stdin)
and writes to stdout (or a `-o` file). No network, no telemetry.

## Install / build

```sh
cargo build --release
# binary at target/release/text-toolkit
```

## Usage

Every subcommand takes an optional input `FILE` (defaults to stdin) and an
optional `-o/--output FILE` (defaults to stdout):

```sh
text-toolkit <SUBCOMMAND> [FILE] [-o OUTPUT] [flags...]
cat data.txt | text-toolkit sort --numeric
```

### Subcommands

| Subcommand | What it does |
|------------|--------------|
| `wc`       | Count lines / words / chars / bytes; or count regex matches with `--pattern`. |
| `dedup`    | Remove duplicate lines (global by default; `--adjacent` for `uniq`-style; `--ignore-case`). |
| `sort`     | Sort lines: `--numeric`, `--reverse`, `--unique`, `--by-field N --delim C`. |
| `case`     | Change case: `--upper`, `--lower`, `--title`, `--sentence`. |
| `freq`     | Word (or `--chars`) frequency; `--top N`, `--ignore-case`. |
| `wrap`     | Word-wrap to `--width 80` (never splits a word). |
| `trim`     | Strip trailing whitespace and normalize blank lines. |
| `number`   | Prefix each line with a right-aligned line number. |
| `head` / `tail` | First / last `-n N` lines. |
| `replace`  | Regex search/replace: `--pattern RE --with STR` (supports `$1`, `${name}`). |

### Examples

```sh
# Count: lines words bytes (default)
printf 'a b\nc d e\n' | text-toolkit wc
# -> 2 5 8

# Only word count
text-toolkit wc --words README.md

# Count occurrences of a pattern
text-toolkit wc --pattern '\bfox\b' samples/prose.txt

# Numeric sort
text-toolkit sort --numeric samples/numbers.txt

# Sort CSV-ish data by the 2nd field, numerically
printf 'alice,30\nbob,5\n' | text-toolkit sort --numeric --by-field 2 --delim ,

# Dedup, case-insensitive, global
text-toolkit dedup --ignore-case names.txt

# Top 5 most common words, case-insensitive
text-toolkit freq --ignore-case --top 5 samples/prose.txt

# Word-wrap to 60 columns
text-toolkit wrap --width 60 article.txt

# Title-case a heading
echo 'the lord of the rings' | text-toolkit case --title

# Swap "First Last" -> "Last, First"
echo 'Alice Smith' | text-toolkit replace --pattern '(\w+) (\w+)' --with '$2, $1'
```

## Design

- `src/lib.rs` exposes the core: each operation is a pure function
  (`&str`/`&[&str]` in, owned `String`/`Vec` out) in its own module
  (`wc`, `dedup`, `sort`, `casing`, `freq`, `wrap`, `trim`, `number`, `replace`).
- `src/main.rs` is a thin [clap](https://docs.rs/clap) wrapper that only handles
  argument parsing and I/O, delegating all logic to the library.
- Tests: extensive per-module unit tests plus end-to-end CLI integration tests
  (`tests/cli.rs`) that pipe through the real compiled binary via
  [assert_cmd](https://docs.rs/assert_cmd).

### Line-handling conventions

- Input lines split on `\n`, with a single trailing `\r` stripped per line
  (CRLF-safe). A trailing newline does not create a phantom empty final line.
- `wc -l` counts newline terminators (matching GNU `wc`), so text without a
  trailing newline counts one fewer line than the number of visible rows.
- `freq` and `sort` produce deterministic ordering (ties broken lexically), so
  output is reproducible run to run.

## License

MIT — see [LICENSE](LICENSE).
