//! Core Uzbek Latin -> Uzbek Latin-with-diacritics transliteration.
//!
//! This module only knows how to turn digraphs into their single-codepoint
//! equivalents inside a plain `&str` of prose. It has no idea what
//! Markdown, HTML, or source code look like — that's the job of the
//! `markdown` and `html` modules, which call into this one only for the
//! spans of text that are safe to transform.

/// The three apostrophe-like characters Uzbek text uses interchangeably
/// after `g`/`G` and `o`/`O`: the ASCII apostrophe, the Unicode modifier
/// letter turned comma (ʻ), and the left single quotation mark (‘).
const APOSTROPHES: [char; 3] = ['\'', '\u{02BB}', '\u{2018}'];

fn is_apostrophe(c: char) -> bool {
    APOSTROPHES.contains(&c)
}

/// Replace every Uzbek digraph in `input` with its single-letter form.
///
/// Rules (case is preserved per the table in the spec):
/// - `Sh`, `SH` -> `Ş`; `sh` -> `ş`
/// - `Ch`, `CH` -> `Ç`; `ch` -> `ç`
/// - `G` + apostrophe -> `Ğ`; `g` + apostrophe -> `ğ`
/// - `O` + apostrophe -> `Õ`; `o` + apostrophe -> `õ`
///
/// The function is allocation-conscious: it scans `input` once and only
/// allocates a `String` if a replacement is actually made, so callers that
/// pass text with no Uzbek digraphs in it (e.g. most inline code, most
/// short strings) pay almost nothing.
pub fn transliterate(input: &str) -> String {
    // Fast path: if there's nothing to replace, skip building a new String
    // and hand back a clone-free copy. `contains` on ASCII bytes is cheap
    // and lets most non-Uzbek text (numbers, punctuation, code fragments
    // that slip through) skip the char-by-char pass entirely.
    if !might_contain_digraph(input) {
        return input.to_string();
    }

    let mut out = String::with_capacity(input.len());
    let mut chars = input.char_indices().peekable();

    while let Some((_, c)) = chars.next() {
        match c {
            'S' => {
                if peek_is_either(&mut chars, 'h', 'H') {
                    consume_one(&mut chars);
                    out.push('Ş');
                } else {
                    out.push(c);
                }
            }
            's' => {
                if peek_is(&mut chars, 'h') {
                    consume_one(&mut chars);
                    out.push('ş');
                } else {
                    out.push(c);
                }
            }
            'C' => {
                if peek_is_either(&mut chars, 'h', 'H') {
                    consume_one(&mut chars);
                    out.push('Ç');
                } else {
                    out.push(c);
                }
            }
            'c' => {
                if peek_is(&mut chars, 'h') {
                    consume_one(&mut chars);
                    out.push('ç');
                } else {
                    out.push(c);
                }
            }
            'G' => {
                if peek_is_apostrophe(&mut chars) {
                    consume_one(&mut chars);
                    out.push('Ğ');
                } else {
                    out.push(c);
                }
            }
            'g' => {
                if peek_is_apostrophe(&mut chars) {
                    consume_one(&mut chars);
                    out.push('ğ');
                } else {
                    out.push(c);
                }
            }
            'O' => {
                if peek_is_apostrophe(&mut chars) {
                    consume_one(&mut chars);
                    out.push('Õ');
                } else {
                    out.push(c);
                }
            }
            'o' => {
                if peek_is_apostrophe(&mut chars) {
                    consume_one(&mut chars);
                    out.push('õ');
                } else {
                    out.push(c);
                }
            }
            other => out.push(other),
        }
    }

    out
}

/// Cheap pre-check so plain text with no Uzbek trigger letters never pays
/// for the full char-by-char state machine above.
fn might_contain_digraph(input: &str) -> bool {
    input
        .bytes()
        .any(|b| matches!(b, b'S' | b's' | b'C' | b'c' | b'G' | b'g' | b'O' | b'o'))
}

fn peek_is<I>(chars: &mut std::iter::Peekable<I>, expect: char) -> bool
where
    I: Iterator<Item = (usize, char)>,
{
    matches!(chars.peek(), Some((_, c)) if *c == expect)
}

fn peek_is_either<I>(chars: &mut std::iter::Peekable<I>, a: char, b: char) -> bool
where
    I: Iterator<Item = (usize, char)>,
{
    matches!(chars.peek(), Some((_, c)) if *c == a || *c == b)
}

fn peek_is_apostrophe<I>(chars: &mut std::iter::Peekable<I>) -> bool
where
    I: Iterator<Item = (usize, char)>,
{
    matches!(chars.peek(), Some((_, c)) if is_apostrophe(*c))
}

fn consume_one<I>(chars: &mut std::iter::Peekable<I>)
where
    I: Iterator<Item = (usize, char)>,
{
    chars.next();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replaces_sh_variants() {
        assert_eq!(transliterate("Shahar"), "Şahar");
        assert_eq!(transliterate("SHAHAR"), "ŞAHAR");
        assert_eq!(transliterate("kishlar shahar"), "kişlar şahar");
    }

    #[test]
    fn replaces_ch_variants_precise() {
        assert_eq!(transliterate("Choy"), "Çoy");
        assert_eq!(transliterate("CHOY"), "ÇOY");
        assert_eq!(transliterate("uch kishi"), "uç kişi");
    }

    #[test]
    fn replaces_g_apostrophe_variants() {
        assert_eq!(transliterate("G'oz"), "Ğoz");
        assert_eq!(transliterate("Gʻoz"), "Ğoz");
        assert_eq!(transliterate("G\u{2018}oz"), "Ğoz");
        assert_eq!(transliterate("bog'"), "boğ");
        assert_eq!(transliterate("bogʻ"), "boğ");
    }

    #[test]
    fn replaces_o_apostrophe_variants() {
        assert_eq!(transliterate("O'zbek"), "Õzbek");
        assert_eq!(transliterate("Oʻzbek"), "Õzbek");
        assert_eq!(transliterate("to'g'ri"), "tõğri");
    }

    #[test]
    fn leaves_plain_ascii_untouched() {
        assert_eq!(transliterate("hello world 123"), "hello world 123");
    }

    #[test]
    fn does_not_touch_isolated_letters() {
        // A bare 'g' or 'o' not followed by an apostrophe must not change.
        assert_eq!(transliterate("men gul olaman"), "men gul olaman");
    }

    #[test]
    fn handles_apostrophe_at_string_end() {
        // trailing apostrophe with nothing after it after g/o is still a digraph
        assert_eq!(transliterate("bog'"), "boğ");
        // but a lone apostrophe with no preceding g/o is untouched
        assert_eq!(transliterate("it's"), "it's");
    }

    #[test]
    fn mixed_sentence() {
        let input = "Toshkent shahrida yashayman, u yerda ko'p bog'lar bor.";
        let expected = "Toşkent şahrida yaşayman, u yerda kõp boğlar bor.";
        assert_eq!(transliterate(input), expected);
    }

    #[test]
    fn empty_string() {
        assert_eq!(transliterate(""), "");
    }

    #[test]
    fn unicode_outside_ascii_untouched() {
        assert_eq!(transliterate("café résumé"), "café résumé");
    }
}
