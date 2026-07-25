//! Command-line interface definition.

use std::path::PathBuf;

use clap::Parser;

/// Transliterate Uzbek Latin digraphs (sh, ch, g', o') into their
/// single-letter Unicode forms (ş, ç, ğ, õ) in prose text, while leaving
/// code, markup, and structured data untouched.
#[derive(Debug, Parser)]
#[command(name = "uztrans", version, about, long_about = None)]
pub struct Cli {
    /// File(s) or directory to process. Omit to read a single document
    /// from stdin (written to stdout unless --output/-o is given).
    pub paths: Vec<PathBuf>,

    /// Write output next to a directory tree instead of overwriting it;
    /// for a single input file, this is the output file path. Cannot be
    /// combined with --in-place.
    #[arg(short, long, value_name = "PATH")]
    pub output: Option<PathBuf>,

    /// Edit matched files in place instead of writing to stdout. Cannot
    /// be combined with --output or with reading from stdin.
    #[arg(short = 'i', long = "in-place")]
    pub in_place: bool,

    /// Recurse into subdirectories when a directory is given. Without
    /// this flag, only files directly inside the given directory are
    /// processed.
    #[arg(short = 'r', long)]
    pub recursive: bool,

    /// Show what would change without writing anything.
    #[arg(long)]
    pub dry_run: bool,

    /// Print a line for every file processed (and every file skipped, and why).
    #[arg(short, long, conflicts_with = "quiet")]
    pub verbose: bool,

    /// Suppress all non-error output.
    #[arg(short, long, conflicts_with = "verbose")]
    pub quiet: bool,

    /// Only process files whose path matches this glob (may be repeated).
    #[arg(long = "include", value_name = "GLOB")]
    pub include: Vec<String>,

    /// Skip files whose path matches this glob (may be repeated); takes
    /// precedence over --include when both match the same file.
    #[arg(long = "exclude", value_name = "GLOB")]
    pub exclude: Vec<String>,

    /// Treat files with this extension as plain prose text in addition
    /// to the built-in defaults (md, txt, html, htm, xml, xhtml). May be
    /// repeated, e.g. `--ext rst --ext adoc`.
    #[arg(long = "ext", value_name = "EXTENSION")]
    pub extra_extensions: Vec<String>,

    /// Print a colored unified diff of the changes instead of (or in
    /// addition to, with --verbose) writing output.
    #[arg(long)]
    pub diff: bool,

    /// Process files in a directory tree in parallel. Safe to use because
    /// each file is read, transformed, and written independently.
    #[arg(long)]
    pub parallel: bool,
}

impl Cli {
    /// Validate flag combinations that clap's own `conflicts_with` can't
    /// express because they depend on *values* (is `paths` empty?), not
    /// just on other flags being present.
    pub fn validate(&self) -> crate::error::Result<()> {
        if self.in_place && self.output.is_some() {
            return Err(crate::error::UztransError::ConflictingOutputMode);
        }
        if self.in_place && self.paths.is_empty() {
            return Err(crate::error::UztransError::InPlaceWithStdin);
        }
        Ok(())
    }
}
