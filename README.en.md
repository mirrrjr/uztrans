# uztrans — Installation & Usage (English)

`uztrans` is a command-line tool that transliterates Uzbek Latin
digraphs into their single-letter Unicode forms, while leaving code,
markup, and structured data untouched.

```
Sh, SH -> Ş     sh -> ş
Ch, CH -> Ç     ch -> ç
G'     -> Ğ     g' -> ğ
O'     -> Õ     o' -> õ
```

## 1. Requirements

- Rust and Cargo, version 1.75 or newer (stable toolchain — no nightly
  features are used). If you don't have Rust yet, install it from
  <https://rustup.rs>.

## 2. Installing

Unzip the project, then from inside the `uztrans` folder:

```bash
cd uztrans
cargo install --path .
```

This builds an optimized binary and copies it to `~/.cargo/bin/uztrans`.
Make sure that folder is on your `PATH` (rustup's installer normally
adds this for you automatically). Once installed, `uztrans` is
available from any directory, in any terminal session.

If you'd rather not install it system-wide, you can just build it and
run it from the project folder instead:

```bash
cargo build --release
./target/release/uztrans --help
```

## 3. Basic usage

```bash
# Print a transliterated file to the terminal
uztrans book.md

# Edit a file directly (overwrites it)
uztrans --in-place book.md

# Process a whole folder, including subfolders
uztrans --in-place --recursive docs/

# See what would change, without writing anything
uztrans --dry-run --diff book.md

# Write the result to a different file, leaving the original untouched
uztrans book.md -o book.translit.md

# Read from a pipe, write to a pipe
cat book.md | uztrans > book.translit.md
```

Run `uztrans --help` at any time to see every available flag.

## 4. What it does and doesn't touch

| File type | Behavior |
|---|---|
| `.md`, `.markdown` | Prose is transliterated; code blocks, inline code, and link URLs are left exactly as they were. |
| `.html`, `.htm`, `.xml`, `.xhtml` | Visible text is transliterated; tags, attributes, `<script>`/`<style>` contents, and comments are left exactly as they were. |
| `.txt` | The whole file is treated as prose and transliterated. |
| anything else (`.rs`, `.py`, `.json`, ...) | Never touched, so it's always safe to point `uztrans` at a mixed folder. |

## 5. Common flags

| Flag | What it does |
|---|---|
| `-o, --output <PATH>` | Write to a different file or folder instead of overwriting the input. |
| `-i, --in-place` | Overwrite the input file(s) directly. |
| `-r, --recursive` | When given a folder, also process its subfolders. |
| `--dry-run` | Show what would happen without changing any file. |
| `--diff` | Print a colored, line-by-line preview of the changes. |
| `--include <GLOB>` / `--exclude <GLOB>` | Limit processing to (or skip) files matching a pattern, e.g. `--exclude "*.generated.md"`. |
| `--ext <EXTENSION>` | Also treat a file extension not in the default list as prose. |

If anything is unclear, `uztrans --help` is always the authoritative
reference for the exact flags available in your build.
