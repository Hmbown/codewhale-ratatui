//! Text that is safe to measure and paint.
//!
//! Every string a caller passes can come from a model, a tool or a file name.
//! ratatui 0.30 already drops control characters when it writes cells, so an
//! escape sequence cannot drive the terminal. Bidirectional overrides pass,
//! though, and can make `rm -rf ~/x` display as something else, so every
//! component runs caller text through [`display_safe`] first.

use std::borrow::Cow;

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::glyphs;

/// Characters that reorder or hide text without drawing anything.
fn is_hidden_control(c: char) -> bool {
    matches!(c,
        '\u{202A}'..='\u{202E}'   // LRE RLE PDF LRO RLO
        | '\u{2066}'..='\u{2069}' // LRI RLI FSI PDI
        | '\u{200E}' | '\u{200F}' | '\u{061C}' // LRM RLM ALM
    ) || c.is_control()
}

/// `text` without bidi controls or other control characters.
#[must_use]
pub fn display_safe(text: &str) -> Cow<'_, str> {
    if text.chars().any(is_hidden_control) {
        Cow::Owned(text.chars().filter(|c| !is_hidden_control(*c)).collect())
    } else {
        Cow::Borrowed(text)
    }
}

/// Display width in terminal cells.
#[must_use]
pub fn width(text: &str) -> usize {
    UnicodeWidthStr::width(text)
}

/// Fit `text` into `max` cells, ending with `…` (or `.` when ASCII-safe)
/// when anything was cut. Never splits a wide character.
#[must_use]
pub fn truncate(text: &str, max: usize, ascii: bool) -> Cow<'_, str> {
    if width(text) <= max {
        return Cow::Borrowed(text);
    }
    if max == 0 {
        return Cow::Borrowed("");
    }
    let ellipsis = glyphs::pick(glyphs::ELLIPSIS, ascii);
    let budget = max - width(ellipsis);
    let mut out = String::new();
    let mut used = 0;
    for c in text.chars() {
        let w = c.width().unwrap_or(0);
        if used + w > budget {
            break;
        }
        used += w;
        out.push(c);
    }
    let trimmed = out.trim_end();
    let mut out = trimmed.to_string();
    out.push_str(ellipsis);
    Cow::Owned(out)
}

/// Pad `text` with spaces to exactly `cells` wide (truncating first).
#[must_use]
pub fn pad(text: &str, cells: usize, ascii: bool) -> String {
    let fitted = truncate(text, cells, ascii);
    let gap = cells.saturating_sub(width(&fitted));
    format!("{fitted}{}", " ".repeat(gap))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bidi_overrides_are_stripped() {
        let spoof = "rm -rf ~/\u{202E}txt.exe";
        assert_eq!(display_safe(spoof), "rm -rf ~/txt.exe");
        assert!(matches!(display_safe("plain"), Cow::Borrowed(_)));
        assert_eq!(display_safe("a\u{1b}[31mb"), "a[31mb");
    }

    #[test]
    fn truncate_respects_cell_width() {
        assert_eq!(truncate("Shoreline", 20, false), "Shoreline");
        assert_eq!(truncate("Shoreline light", 10, false), "Shoreline…");
        assert_eq!(truncate("Shoreline light", 10, true), "Shoreline.");
        // A wide character is never split.
        assert_eq!(truncate("鲸鱼鲸鱼", 5, false), "鲸鱼…");
        assert_eq!(width(&pad("ab", 4, false)), 4);
    }
}
