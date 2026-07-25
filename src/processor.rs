//! Dispatches a document to the right structure-aware transliterator
//! based on its `DocKind`, and reports whether anything actually changed.

use crate::config::DocKind;
use crate::html::transliterate_html;
use crate::markdown::transliterate_markdown;
use crate::transliterator::transliterate;

pub struct ProcessResult {
    pub output: String,
    pub changed: bool,
}

pub fn process(source: &str, kind: DocKind) -> ProcessResult {
    let output = match kind {
        DocKind::Markdown => transliterate_markdown(source),
        DocKind::Html => transliterate_html(source),
        DocKind::PlainText => transliterate(source),
    };
    let changed = output != source;
    ProcessResult { output, changed }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_text_dispatch() {
        let result = process("shahar", DocKind::PlainText);
        assert_eq!(result.output, "şahar");
        assert!(result.changed);
    }

    #[test]
    fn markdown_dispatch_preserves_code() {
        let result = process("shahar `sh` shahar", DocKind::Markdown);
        assert!(result.output.contains("`sh`"));
        assert!(result.changed);
    }

    #[test]
    fn html_dispatch_preserves_tags() {
        let result = process("<p>shahar</p>", DocKind::Html);
        assert_eq!(result.output, "<p>şahar</p>");
        assert!(result.changed);
    }

    #[test]
    fn no_op_reports_unchanged() {
        let result = process("hello world", DocKind::PlainText);
        assert!(!result.changed);
        assert_eq!(result.output, "hello world");
    }
}
