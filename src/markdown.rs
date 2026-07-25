//! Markdown-aware transliteration.
//!
//! The key design decision here: we never re-serialize the Markdown AST
//! back to a string. Round-tripping Markdown through a parser and a
//! serializer is lossy (it normalizes whitespace, quote styles, list
//! markers, etc.) and would violate "never modify code, markup, or
//! anything that isn't prose."
//!
//! Instead we use `pulldown-cmark`'s `OffsetIter` purely as a map of
//! "which byte ranges of the original source are visible prose text,"
//! and splice `transliterate()` output into the *original* string only
//! at those ranges. Everything else — code fences, inline code, link
//! destinations, HTML blocks, front matter — passes through byte-for-byte
//! untouched, because we simply never touch it.

use pulldown_cmark::{CowStr, Event, Options, Parser, Tag, TagEnd};

use crate::transliterator::transliterate;

/// Transliterate the prose text of a Markdown document, leaving code
/// blocks, inline code, link/image URLs, and raw HTML untouched.
pub fn transliterate_markdown(source: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);

    let parser = Parser::new_ext(source, options).into_offset_iter();

    // Depth of "do not touch text here" contexts we're currently inside:
    // fenced/indented code blocks and inline code spans. We track a count
    // rather than a bool because CodeBlock start/end events are properly
    // nested with other block content in practice, but being defensive
    // here costs nothing.
    let mut code_depth: u32 = 0;

    // Byte ranges (relative to `source`) that are safe to transliterate,
    // collected in document order and non-overlapping (pulldown-cmark
    // gives us non-overlapping Text event spans).
    let mut safe_ranges: Vec<(usize, usize)> = Vec::new();

    for (event, range) in parser {
        match event {
            Event::Start(Tag::CodeBlock(_)) => code_depth += 1,
            Event::End(TagEnd::CodeBlock) => code_depth = code_depth.saturating_sub(1),
            Event::Code(_) => {
                // Inline code: `Event::Code` carries its own range covering
                // the backticks + content, never touch it.
            }
            Event::Text(_) if code_depth == 0 => {
                safe_ranges.push((range.start, range.end));
            }
            Event::Html(_) | Event::InlineHtml(_) => {
                // Raw HTML embedded in Markdown: never touch. Note we
                // don't recurse into the html.rs processor here — a user
                // who wants HTML inside Markdown transliterated can run
                // uztrans on the extracted HTML separately. Silently
                // altering embedded HTML from the Markdown path risks
                // corrupting attributes we can't see the boundaries of
                // at this layer.
            }
            _ => {}
        }
    }

    splice_transliterated(source, &safe_ranges)
}

/// Apply `transliterate()` to just the given byte ranges of `source`,
/// copying everything outside those ranges through unchanged.
fn splice_transliterated(source: &str, ranges: &[(usize, usize)]) -> String {
    let mut out = String::with_capacity(source.len());
    let mut cursor = 0usize;

    for &(start, end) in ranges {
        if start < cursor || end > source.len() || start > end {
            // Defensive: if pulldown-cmark ever hands back a range that
            // doesn't fit our monotonic cursor assumption, skip it rather
            // than panicking or corrupting output — better to under-
            // transliterate than to garble the document.
            continue;
        }
        out.push_str(&source[cursor..start]);
        out.push_str(&transliterate(&source[start..end]));
        cursor = end;
    }
    out.push_str(&source[cursor..]);
    out
}

/// Extract link/footnote reference definitions is intentionally *not*
/// implemented as a separate pass: `Event::Text` inside a link label is
/// still prose (e.g. `[Toshkentda]`), while the destination itself never
/// produces a `Text` event, so the offset-based approach above already
/// does the right thing without special-casing links.
#[allow(dead_code)]
fn _doc_link_note() {}

// Re-exported so callers can format a value the same way pulldown-cmark
// would for tests/debugging without pulling in the whole crate elsewhere.
#[allow(dead_code)]
pub(crate) fn cow_to_string(c: CowStr<'_>) -> String {
    c.into_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transliterates_plain_paragraph() {
        let input = "Bu Toshkent shahri haqida.";
        let expected = "Bu Toşkent şahri haqida.";
        assert_eq!(transliterate_markdown(input), expected);
    }

    #[test]
    fn leaves_fenced_code_block_untouched() {
        let input =
            "Matn boshida shahar.\n\n```rust\nlet gʻoz = \"shaxsiy\";\n```\n\nOxirida ham shahar.";
        let output = transliterate_markdown(input);
        assert!(output.contains("let gʻoz = \"shaxsiy\";"));
        assert!(!output.contains("Şahar")); // sanity: no accidental capitalization drift
        assert!(output.starts_with("Matn boşida şahar."));
        assert!(output.ends_with("Oxirida ham şahar."));
    }

    #[test]
    fn leaves_inline_code_untouched() {
        let input = "Buyruq `cargo build --release shu yerda` ishlaydi shu yerda.";
        let output = transliterate_markdown(input);
        assert!(output.contains("`cargo build --release shu yerda`"));
        assert!(output.ends_with("işlaydi şu yerda."));
    }

    #[test]
    fn leaves_link_destination_untouched_but_translit_label() {
        let input = "[Bu yerga bosing](https://example.com/gʻoz-sahifa)";
        let output = transliterate_markdown(input);
        assert!(output.contains("(https://example.com/gʻoz-sahifa)"));
        assert!(output.contains("[Bu yerga bosing]"));
    }

    #[test]
    fn indented_code_block_untouched() {
        let input = "Boshlanishi shu.\n\n    let gʻalati = true;\n\nOxiri shu.";
        let output = transliterate_markdown(input);
        assert!(output.contains("    let gʻalati = true;"));
    }

    #[test]
    fn raw_html_block_untouched() {
        let input = "<div class=\"shahar\">matn shu yerda</div>\n\nOddiy shahar matni.";
        let output = transliterate_markdown(input);
        assert!(output.starts_with("<div class=\"shahar\">matn shu yerda</div>"));
        assert!(output.ends_with("Oddiy şahar matni."));
    }

    #[test]
    fn empty_document() {
        assert_eq!(transliterate_markdown(""), "");
    }

    #[test]
    fn heading_and_table_prose_translated() {
        let input = "# Shahar haqida\n\n| Nomi | Tavsif |\n|------|--------|\n| Toshkent | katta shahar |\n";
        let output = transliterate_markdown(input);
        assert!(output.contains("# Şahar haqida"));
        assert!(output.contains("Toşkent"));
        assert!(output.contains("katta şahar"));
    }
}
