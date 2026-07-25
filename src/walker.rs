//! Turning a CLI path argument (file or directory) into a concrete list
//! of files to process, honoring --recursive, --include, and --exclude.

use std::path::{Path, PathBuf};

use globset::{Glob, GlobSet, GlobSetBuilder};

use crate::config::ExtensionConfig;
use crate::error::{Result, UztransError};

pub struct Filters {
    pub include: Option<GlobSet>,
    pub exclude: Option<GlobSet>,
}

impl Filters {
    pub fn build(include: &[String], exclude: &[String]) -> Result<Self> {
        Ok(Self {
            include: build_globset(include)?,
            exclude: build_globset(exclude)?,
        })
    }

    fn allows(&self, path: &Path) -> bool {
        if let Some(exclude) = &self.exclude {
            if exclude.is_match(path) {
                return false;
            }
        }
        if let Some(include) = &self.include {
            return include.is_match(path);
        }
        true
    }
}

fn build_globset(patterns: &[String]) -> Result<Option<GlobSet>> {
    if patterns.is_empty() {
        return Ok(None);
    }
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        let glob = Glob::new(pattern).map_err(|source| UztransError::InvalidGlob {
            pattern: pattern.clone(),
            source,
        })?;
        builder.add(glob);
    }
    let set = builder
        .build()
        .map_err(|source| UztransError::InvalidGlob {
            pattern: patterns.join(", "),
            source,
        })?;
    Ok(Some(set))
}

/// Expand `roots` (files and/or directories) into the concrete file list
/// to process, applying extension recognition and include/exclude globs.
/// Directories are walked recursively only when `recursive` is true;
/// otherwise only their immediate children are considered.
pub fn collect_files(
    roots: &[PathBuf],
    recursive: bool,
    ext_config: &ExtensionConfig,
    filters: &Filters,
) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();

    for root in roots {
        if !root.exists() {
            return Err(UztransError::NotFound { path: root.clone() });
        }

        if root.is_file() {
            out.push(root.clone());
            continue;
        }

        let max_depth = if recursive { usize::MAX } else { 1 };
        let walker = walkdir::WalkDir::new(root)
            .max_depth(max_depth)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| e.depth() == 0 || !is_hidden_dir(e));

        for entry in walker {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue, // permission errors etc: skip, don't abort the whole walk
            };
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.into_path();
            if is_processable(&path, ext_config, filters) {
                out.push(path);
            }
        }
    }

    Ok(out)
}

fn is_hidden_dir(entry: &walkdir::DirEntry) -> bool {
    entry.file_type().is_dir()
        && entry
            .file_name()
            .to_str()
            .map(|s| s.starts_with('.') && s != "." && s != "..")
            .unwrap_or(false)
}

fn is_processable(path: &Path, ext_config: &ExtensionConfig, filters: &Filters) -> bool {
    let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
        return false;
    };
    if ext_config.doc_kind(ext).is_none() {
        return false;
    }
    filters.allows(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn touch(dir: &Path, rel: &str) {
        let path = dir.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, "shahar").unwrap();
    }

    #[test]
    fn non_recursive_only_top_level() {
        let dir = tempdir().unwrap();
        touch(dir.path(), "a.md");
        touch(dir.path(), "sub/b.md");

        let ext = ExtensionConfig::default();
        let filters = Filters::build(&[], &[]).unwrap();
        let files = collect_files(&[dir.path().to_path_buf()], false, &ext, &filters).unwrap();

        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("a.md"));
    }

    #[test]
    fn recursive_finds_nested_files() {
        let dir = tempdir().unwrap();
        touch(dir.path(), "a.md");
        touch(dir.path(), "sub/b.md");
        touch(dir.path(), "sub/deeper/c.txt");

        let ext = ExtensionConfig::default();
        let filters = Filters::build(&[], &[]).unwrap();
        let files = collect_files(&[dir.path().to_path_buf()], true, &ext, &filters).unwrap();

        assert_eq!(files.len(), 3);
    }

    #[test]
    fn unrecognized_extensions_skipped() {
        let dir = tempdir().unwrap();
        touch(dir.path(), "a.md");
        touch(dir.path(), "b.rs");

        let ext = ExtensionConfig::default();
        let filters = Filters::build(&[], &[]).unwrap();
        let files = collect_files(&[dir.path().to_path_buf()], true, &ext, &filters).unwrap();

        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("a.md"));
    }

    #[test]
    fn exclude_glob_wins_over_include() {
        let dir = tempdir().unwrap();
        touch(dir.path(), "keep.md");
        touch(dir.path(), "skip.md");

        let ext = ExtensionConfig::default();
        let filters = Filters::build(&["*.md".to_string()], &["*skip.md".to_string()]).unwrap();
        let files = collect_files(&[dir.path().to_path_buf()], true, &ext, &filters).unwrap();

        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("keep.md"));
    }

    #[test]
    fn hidden_directories_are_skipped() {
        let dir = tempdir().unwrap();
        touch(dir.path(), "a.md");
        touch(dir.path(), ".git/b.md");

        let ext = ExtensionConfig::default();
        let filters = Filters::build(&[], &[]).unwrap();
        let files = collect_files(&[dir.path().to_path_buf()], true, &ext, &filters).unwrap();

        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("a.md"));
    }

    #[test]
    fn missing_root_is_an_error() {
        let ext = ExtensionConfig::default();
        let filters = Filters::build(&[], &[]).unwrap();
        let result = collect_files(&[PathBuf::from("/no/such/path")], true, &ext, &filters);
        assert!(result.is_err());
    }
}
