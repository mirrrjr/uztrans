//! HTML-aware transliteration.
//!
//! ## Why this isn't built on html5ever's tree parser
//!
//! `html5ever` + `markup5ever_rcdom` can build a full DOM, but turning that
//! DOM back into bytes means *serializing* it — and HTML serialization is
//! lossy in exactly the ways we're not allowed to be: it can reorder or
//! re-quote attributes, normalize self-closing syntax, rewrite entity
//! encoding, and collapse whitespace differently than the source did. For
//! a tool whose entire job is "touch prose, never touch markup," round-
//! tripping through a DOM is the wrong shape of solution even though the
//! spec's dependency list suggests it — the same reasoning that keeps
//! `markdown.rs` off `pulldown-cmark`'s serializer applies here.
//!
//! Instead this module is a small, deliberately dumb byte-oriented
//! scanner: it classifies every byte of the input as either "inside a tag
//! / comment / raw-text element" (untouched) or "inside a text node"
//! (safe to transliterate), and splices `transliterate()` output into the
//! *original* bytes at the safe spans only — same splice-based strategy
//! as the Markdown path. Nothing about tag structure, attributes, or
//! entities is ever rewritten because we never parse them into a
//! reconstructable representation in the first place.
//!
//! This intentionally does not handle malformed/tag-soup HTML with the
//! same forgiving heuristics a browser would (html5ever's tokenizer has
//! decades of that logic baked in). For the documents this tool targets
//! — README-adjacent HTML fragments and generated pages, not arbitrary
//! hostile markup — a straightforward scanner is the pragmatic choice;
//! reasonably well-formed input round-trips correctly, which is what the
//! test suite in this module verifies.

use crate::transliterator::transliterate;

/// Elements whose entire content (until the matching end tag) is raw
/// text/code, never prose: never transliterate inside these regardless
/// of what the text looks like.
const RAW_TEXT_ELEMENTS: &[&str] = &["script", "style", "pre", "code", "textarea"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Text,
    Tag,
    Comment,
    /// Inside a raw-text element's content, waiting for its closing tag.
    RawText,
}

pub fn transliterate_html(source: &str) -> String {
    let bytes = source.as_bytes();
    let len = bytes.len();

    let mut safe_ranges: Vec<(usize, usize)> = Vec::new();
    let mut state = State::Text;
    let mut text_start = 0usize;
    let mut raw_end_tag: Option<String> = None;

    let mut i = 0usize;
    while i < len {
        // `bytes[i]` and single-byte-ASCII comparisons below are only
        // meaningful/safe when `i` sits on a char boundary; since we may
        // be scanning multi-byte UTF-8 (e.g. the Uzbek apostrophe
        // characters), every branch that doesn't consume a recognized
        // ASCII token must advance by the current char's full width, not
        // a single byte, or a later `source[i..]` slice will panic.
        let this_char_len = source[i..]
            .chars()
            .next()
            .map(|c| c.len_utf8())
            .unwrap_or(1);

        match state {
            State::Text => {
                if bytes[i] == b'<' {
                    if text_start < i {
                        push_text_range(source, text_start, i, &mut safe_ranges);
                    }
                    if source[i..].to_ascii_lowercase().starts_with("<!--") {
                        state = State::Comment;
                        i += 4;
                        continue;
                    }
                    // Determine if this opens a raw-text element so we
                    // know what closing tag to watch for.
                    if let Some(name) = tag_name_at(source, i) {
                        let lower = name.to_ascii_lowercase();
                        if RAW_TEXT_ELEMENTS.contains(&lower.as_str()) {
                            raw_end_tag = Some(format!("</{}", lower));
                        }
                    }
                    state = State::Tag;
                    i += 1;
                } else {
                    i += this_char_len;
                }
            }
            State::Tag => {
                if bytes[i] == b'>' {
                    i += 1;
                    if raw_end_tag.is_some() {
                        state = State::RawText;
                    } else {
                        state = State::Text;
                        text_start = i;
                    }
                } else {
                    i += this_char_len;
                }
            }
            State::Comment => {
                if source[i..].starts_with("-->") {
                    i += 3;
                    state = State::Text;
                    text_start = i;
                } else {
                    i += this_char_len;
                }
            }
            State::RawText => {
                let end_tag = raw_end_tag.as_deref().unwrap_or("</");
                if i + end_tag.len() <= len
                    && source.is_char_boundary(i + end_tag.len())
                    && source[i..i + end_tag.len()].eq_ignore_ascii_case(end_tag)
                {
                    // Hand off to Tag state to consume the closing tag
                    // itself (e.g. `</script>`), which also isn't prose.
                    raw_end_tag = None;
                    state = State::Tag;
                    i += 1;
                } else {
                    i += this_char_len;
                }
            }
        }
    }

    if state == State::Text && text_start < len {
        push_text_range(source, text_start, len, &mut safe_ranges);
    }

    splice_transliterated(source, &safe_ranges)
}

/// If `source[pos..]` starts with `<` followed by an element name (an
/// opening tag, not a closing tag or `<!...`), return that name.
fn tag_name_at(source: &str, pos: usize) -> Option<String> {
    let rest = &source[pos + 1..];
    let rest = rest
        .strip_prefix(|c: char| c == '/')
        .map(|_| None)
        .unwrap_or(Some(rest));
    let rest = rest?;
    let mut name = String::new();
    for c in rest.chars() {
        if c.is_ascii_alphanumeric() || c == '-' {
            name.push(c);
        } else {
            break;
        }
    }
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

/// Push `source[start..end]` as a safe (transliteratable) range, but carve
/// out HTML character-reference spans (`&amp;`, `&#39;`, `&#x27;`, ...)
/// within it first, since those are markup, not prose, even though they
/// appear inside a text node.
fn push_text_range(source: &str, start: usize, end: usize, out: &mut Vec<(usize, usize)>) {
    let text = &source[start..end];
    let mut cursor = 0usize;
    let bytes = text.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'&' {
            if let Some(semi_rel) = text[i..].find(';') {
                // Cap entity length so a stray `&` in prose (e.g. "R&D")
                // doesn't swallow the rest of the paragraph looking for a
                // `;` that belongs to something else entirely.
                if semi_rel <= 32 {
                    let entity_end = i + semi_rel + 1;
                    if cursor < i {
                        out.push((start + cursor, start + i));
                    }
                    // entity itself [i, entity_end) is skipped, not pushed
                    cursor = entity_end;
                    i = entity_end;
                    continue;
                }
            }
        }
        i += 1;
    }
    if cursor < text.len() {
        out.push((start + cursor, start + text.len()));
    }
}

fn splice_transliterated(source: &str, ranges: &[(usize, usize)]) -> String {
    let mut out = String::with_capacity(source.len());
    let mut cursor = 0usize;
    for &(start, end) in ranges {
        if start < cursor || end > source.len() || start > end {
            continue;
        }
        out.push_str(&source[cursor..start]);
        out.push_str(&transliterate(&source[start..end]));
        cursor = end;
    }
    out.push_str(&source[cursor..]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transliterates_simple_text_node() {
        let input = "<p>Bu Toshkent shahri.</p>";
        assert_eq!(transliterate_html(input), "<p>Bu Toşkent şahri.</p>");
    }

    #[test]
    fn leaves_tag_names_and_attributes_untouched() {
        let input = "<div class=\"shahar-kartasi\" data-shahar=\"Toshkent\">Shahar</div>";
        let output = transliterate_html(input);
        assert!(output.starts_with("<div class=\"shahar-kartasi\" data-shahar=\"Toshkent\">"));
        assert!(output.ends_with("Şahar</div>"));
    }

    #[test]
    fn leaves_script_content_untouched() {
        let input = "<p>shahar</p><script>var shahar = \"gʻalati\";</script><p>shahar</p>";
        let output = transliterate_html(input);
        assert!(output.contains("var shahar = \"gʻalati\";"));
        assert!(output.starts_with("<p>şahar</p>"));
        assert!(output.ends_with("<p>şahar</p>"));
    }

    #[test]
    fn leaves_style_content_untouched() {
        let input = "<style>.shahar { color: red; }</style><p>shahar</p>";
        let output = transliterate_html(input);
        assert!(output.contains(".shahar { color: red; }"));
        assert!(output.ends_with("<p>şahar</p>"));
    }

    #[test]
    fn leaves_pre_code_untouched() {
        let input = "<pre><code>let gʻoz = 1;</code></pre><p>gʻoz</p>";
        let output = transliterate_html(input);
        assert!(output.contains("<pre><code>let gʻoz = 1;</code></pre>"));
        assert!(output.ends_with("<p>ğoz</p>"));
    }

    #[test]
    fn leaves_comments_untouched() {
        let input = "<!-- shahar haqida izoh --><p>shahar</p>";
        let output = transliterate_html(input);
        assert!(output.starts_with("<!-- shahar haqida izoh -->"));
        assert!(output.ends_with("<p>şahar</p>"));
    }

    #[test]
    fn leaves_entities_untouched() {
        let input = "<p>R&amp;D bo'yicha shahar &mdash; katta.</p>";
        let output = transliterate_html(input);
        assert!(output.contains("R&amp;D"));
        assert!(output.contains("&mdash;"));
        assert!(output.contains("bõyiça şahar"));
    }

    #[test]
    fn nested_tags_prose_still_translated() {
        let input = "<div><p>Bu <strong>shahar</strong> haqida <em>gʻoz</em> gapiradi.</p></div>";
        let output = transliterate_html(input);
        assert_eq!(
            output,
            "<div><p>Bu <strong>şahar</strong> haqida <em>ğoz</em> gapiradi.</p></div>"
        );
    }

    #[test]
    fn empty_document() {
        assert_eq!(transliterate_html(""), "");
    }

    #[test]
    fn self_closing_tags_untouched() {
        let input = "<p>shahar<br/>gʻoz</p>";
        let output = transliterate_html(input);
        assert_eq!(output, "<p>şahar<br/>ğoz</p>");
    }
}
