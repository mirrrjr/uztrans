//! Which file extensions `uztrans` treats as "text it should process",
//! and which document-type-specific processor handles each one.

use std::collections::HashSet;

/// How a file's contents should be walked before transliteration is
/// applied — this is what lets us skip fenced code blocks, HTML tags,
/// etc., instead of blindly replacing every digraph in the file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocKind {
    Markdown,
    Html,
    PlainText,
}

/// Extensions supported out of the box, mapped to how they should be parsed.
/// `xml`/`xhtml`/`svg`-like formats are close enough to HTML's "tags vs.
/// text nodes" shape that we reuse the HTML processor for them; anything
/// not in this list but explicitly added by the user via `--ext` is
/// treated as plain text (safe default: transliterate everything, since we
/// have no structure to preserve).
pub fn default_doc_kind(ext: &str) -> Option<DocKind> {
    match ext.to_ascii_lowercase().as_str() {
        "md" | "markdown" => Some(DocKind::Markdown),
        "html" | "htm" | "xhtml" | "xml" => Some(DocKind::Html),
        "txt" => Some(DocKind::PlainText),
        _ => None,
    }
}

#[derive(Debug, Clone)]
pub struct ExtensionConfig {
    extra_extensions: HashSet<String>,
}

impl ExtensionConfig {
    pub fn new(extra: impl IntoIterator<Item = String>) -> Self {
        Self {
            extra_extensions: extra
                .into_iter()
                .map(|e| e.trim_start_matches('.').to_ascii_lowercase())
                .collect(),
        }
    }

    /// Returns how to process `ext`, or `None` if this extension isn't
    /// one uztrans has been told to touch.
    pub fn doc_kind(&self, ext: &str) -> Option<DocKind> {
        let lower = ext.to_ascii_lowercase();
        if let Some(kind) = default_doc_kind(&lower) {
            return Some(kind);
        }
        if self.extra_extensions.contains(&lower) {
            return Some(DocKind::PlainText);
        }
        None
    }
}

impl Default for ExtensionConfig {
    fn default() -> Self {
        Self::new(std::iter::empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_extensions_recognized() {
        assert_eq!(default_doc_kind("md"), Some(DocKind::Markdown));
        assert_eq!(default_doc_kind("MD"), Some(DocKind::Markdown));
        assert_eq!(default_doc_kind("html"), Some(DocKind::Html));
        assert_eq!(default_doc_kind("htm"), Some(DocKind::Html));
        assert_eq!(default_doc_kind("xml"), Some(DocKind::Html));
        assert_eq!(default_doc_kind("xhtml"), Some(DocKind::Html));
        assert_eq!(default_doc_kind("txt"), Some(DocKind::PlainText));
        assert_eq!(default_doc_kind("rs"), None);
    }

    #[test]
    fn extra_extensions_are_plain_text() {
        let cfg = ExtensionConfig::new(vec!["rst".to_string(), ".adoc".to_string()]);
        assert_eq!(cfg.doc_kind("rst"), Some(DocKind::PlainText));
        assert_eq!(cfg.doc_kind("adoc"), Some(DocKind::PlainText));
        assert_eq!(cfg.doc_kind("rs"), None);
    }

    #[test]
    fn defaults_still_win_over_missing_extra() {
        let cfg = ExtensionConfig::default();
        assert_eq!(cfg.doc_kind("md"), Some(DocKind::Markdown));
        assert_eq!(cfg.doc_kind("py"), None);
    }
}
