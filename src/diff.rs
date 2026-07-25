//! Colored unified-diff preview for `--diff`.

use owo_colors::OwoColorize;
use similar::{ChangeTag, TextDiff};

pub fn print_diff(label: &str, before: &str, after: &str) {
    println!("{}", format!("--- {label}").bold());
    let diff = TextDiff::from_lines(before, after);
    for change in diff.iter_all_changes() {
        let line = change.to_string_lossy();
        match change.tag() {
            ChangeTag::Delete => print!("{}", format!("-{line}").red()),
            ChangeTag::Insert => print!("{}", format!("+{line}").green()),
            ChangeTag::Equal => print!(" {line}"),
        }
    }
    println!();
}
