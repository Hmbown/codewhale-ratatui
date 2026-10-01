//! Gallery: key hints.

use ratatui::{buffer::Buffer, layout::Rect};

use super::{Entry, arrows};
use crate::{KeyHint, KeyHints, Paint, Theme};

fn key_hints(area: Rect, buf: &mut Buffer, theme: &Theme) {
    let hints = KeyHints::new(vec![
        KeyHint::new(arrows(theme), "move"),
        KeyHint::new("Enter", "change"),
        KeyHint::new("r", "reset"),
        KeyHint::new("/", "search"),
        KeyHint::new("Ctrl+S", "save").disabled(),
        KeyHint::new("Esc", "close"),
    ]);
    hints.paint(area, buf, theme);
}

pub(crate) fn entries() -> Vec<Entry> {
    vec![Entry {
        name: "key-hints",
        width: 60,
        height: 2,
        draw: key_hints,
    }]
}
