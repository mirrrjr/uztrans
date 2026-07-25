//! Application error types.
//!
//! We use `thiserror` because `uztrans`'s core logic (walking, parsing,
//! transliterating) is used both as the CLI's engine and, in principle,
//! as a library — callers benefit from a structured error enum rather
//! than an opaque `anyhow::Error`. `main.rs` wraps everything in
//! `anyhow` at the top level for convenient `?` propagation and exit-code
//! mapping.

use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum UztransError {
    #[error("failed to read {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to write {path}: {source}")]
    Write {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("{path} is not valid UTF-8; skipping")]
    NotUtf8 { path: PathBuf },

    #[error("{path} looks like a binary file; skipping")]
    LooksBinary { path: PathBuf },

    #[error("invalid glob pattern {pattern:?}: {source}")]
    InvalidGlob {
        pattern: String,
        #[source]
        source: globset::Error,
    },

    #[error("no such file or directory: {path}")]
    NotFound { path: PathBuf },

    #[error("--in-place cannot be combined with -o/--output")]
    ConflictingOutputMode,

    #[error("--in-place cannot be used when reading from stdin")]
    InPlaceWithStdin,
}

pub type Result<T> = std::result::Result<T, UztransError>;
