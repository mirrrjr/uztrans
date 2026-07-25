//! Reading and writing files/stdio, with the binary-file and invalid-UTF-8
//! detection that lets `uztrans` "gracefully skip unreadable files"
//! instead of panicking or corrupting them.

use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use crate::error::{Result, UztransError};

/// Heuristic binary-file sniff: a NUL byte in the first few KB is a
/// strong signal this isn't text uztrans should touch, and lets us skip
/// obvious binaries before even attempting a UTF-8 validity check.
const SNIFF_LEN: usize = 8192;

pub fn read_to_string(path: &Path) -> Result<String> {
    let bytes = fs::read(path).map_err(|source| UztransError::Read {
        path: path.to_path_buf(),
        source,
    })?;

    let sniff_end = bytes.len().min(SNIFF_LEN);
    if bytes[..sniff_end].contains(&0u8) {
        return Err(UztransError::LooksBinary {
            path: path.to_path_buf(),
        });
    }

    String::from_utf8(bytes).map_err(|_| UztransError::NotUtf8 {
        path: path.to_path_buf(),
    })
}

pub fn read_stdin_to_string() -> Result<String> {
    let mut buf = String::new();
    io::stdin()
        .read_to_string(&mut buf)
        .map_err(|source| UztransError::Read {
            path: PathBuf::from("<stdin>"),
            source,
        })?;
    Ok(buf)
}

pub fn write_string(path: &Path, contents: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|source| UztransError::Write {
                path: path.to_path_buf(),
                source,
            })?;
        }
    }
    fs::write(path, contents).map_err(|source| UztransError::Write {
        path: path.to_path_buf(),
        source,
    })
}

pub fn write_stdout(contents: &str) -> Result<()> {
    let stdout = io::stdout();
    let mut lock = stdout.lock();
    lock.write_all(contents.as_bytes())
        .map_err(|source| UztransError::Write {
            path: PathBuf::from("<stdout>"),
            source,
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn reads_valid_utf8_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("a.txt");
        fs::write(&path, "shahar").unwrap();
        assert_eq!(read_to_string(&path).unwrap(), "shahar");
    }

    #[test]
    fn rejects_binary_looking_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("a.bin");
        fs::write(&path, [0u8, 1, 2, 3, b'h', b'i']).unwrap();
        assert!(matches!(
            read_to_string(&path),
            Err(UztransError::LooksBinary { .. })
        ));
    }

    #[test]
    fn rejects_invalid_utf8() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("a.txt");
        fs::write(&path, [0xFF, 0xFE, b'h', b'i']).unwrap();
        assert!(matches!(
            read_to_string(&path),
            Err(UztransError::NotUtf8 { .. })
        ));
    }

    #[test]
    fn write_creates_parent_dirs() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("nested/deep/out.txt");
        write_string(&path, "salom").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "salom");
    }

    #[test]
    fn missing_file_is_read_error() {
        let path = PathBuf::from("/definitely/does/not/exist.txt");
        assert!(matches!(
            read_to_string(&path),
            Err(UztransError::Read { .. })
        ));
    }
}
