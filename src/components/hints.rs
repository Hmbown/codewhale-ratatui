//! Key hints: `↑↓ move · Enter select · Esc cancel`.
//!
//! Lifted from the engine's `ActionHint` and `action_footer_lines`
//! (`crates/tui/src/tui/views/mod.rs`, `Hmbown/CodeWhale` `58b1dd3dd`): hints
//! wrap onto another row rather than run off the edge, and no action is ever
//! dropped (#3732). The key is bold `Foreground` and the verb `Muted`, joined
//! by ` · `. Verbs come first and lower case: "move", not "Navigation".

use std::borrow::Cow;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Modifier,
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::{Paint, Role, Theme, text};

/// One key and what it does.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyHint {
    pub keys: Cow<'static, str>,
    pub verb: Cow<'static, str>,
    /// `false` dims the hint: the key exists but does nothing right now.
    pub enabled: bool,
}

impl KeyHint {
    #[must_use]
    pub fn new(keys: impl Into<Cow<'static, str>>, verb: impl Into<Cow<'static, str>>) -> Self {
        Self {
            keys: keys.into(),
            verb: verb.into(),
            enabled: true,
        }
    }

    /// A hint whose key is spelled from a key event by [`crate::keys`], so
    /// hints and bindings share one spelling: `Ctrl+O`, `⌥V`, `Shift+Tab`.
    #[must_use]
    pub fn chord(
        key: &crossterm::event::KeyEvent,
        platform: crate::keys::Platform,
        verb: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self::new(crate::keys::chord_label(key, platform), verb)
    }

    #[must_use]
    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }

    fn width(&self) -> usize {
        text::width(&self.keys) + 1 + text::width(&self.verb)
    }

    fn spans(&self, theme: &Theme) -> [Span<'static>; 3] {
        let (key_style, verb_style) = if self.enabled {
            (
                theme.fg(Role::Foreground).add_modifier(Modifier::BOLD),
                theme.fg(Role::Muted),
            )
        } else {
            (
                theme.fg(Role::Muted),
                theme.fg(Role::Muted).add_modifier(Modifier::DIM),
            )
        };
        [
            Span::styled(text::display_safe(&self.keys).into_owned(), key_style),
            Span::raw(" "),
            Span::styled(text::display_safe(&self.verb).into_owned(), verb_style),
        ]
    }
}

/// A row of key hints that wraps instead of clipping.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KeyHints {
    pub items: Vec<KeyHint>,
}

impl KeyHints {
    #[must_use]
    pub fn new(items: Vec<KeyHint>) -> Self {
        Self { items }
    }

    /// ` · ` between hints; two spaces when ASCII-safe, because the ASCII
    /// form of `·` is `.` and would read as punctuation.
    fn separator(theme: &Theme) -> Span<'static> {
        if theme.ascii() {
            Span::raw("  ")
        } else {
            Span::styled(" · ", theme.fg(Role::Border))
        }
    }

    /// Lay the hints out in rows no wider than `width`. Packs greedily and
    /// starts a new row rather than truncating; a hint wider than `width`
    /// sits alone on its row.
    #[must_use]
    pub fn lines(&self, width: u16, theme: &Theme) -> Vec<Line<'static>> {
        let width = usize::from(width);
        if self.items.is_empty() || width == 0 {
            return Vec::new();
        }
        let sep = Self::separator(theme);
        let sep_width = text::width(&sep.content);
        let mut lines = Vec::new();
        let mut current: Vec<Span<'static>> = Vec::new();
        let mut used = 0usize;
        for hint in &self.items {
            let w = hint.width();
            if !current.is_empty() && used + sep_width + w > width {
                lines.push(Line::from(std::mem::take(&mut current)));
                used = 0;
            }
            if !current.is_empty() {
                current.push(sep.clone());
                used += sep_width;
            }
            current.extend(hint.spans(theme));
            used += w;
        }
        if !current.is_empty() {
            lines.push(Line::from(current));
        }
        lines
    }
}

impl Paint for KeyHints {
    fn paint(&self, area: Rect, buf: &mut Buffer, theme: &Theme) {
        Paragraph::new(self.lines(area.width, theme)).render(area, buf);
    }

    fn height(&self, width: u16, theme: &Theme) -> u16 {
        u16::try_from(self.lines(width, theme).len()).unwrap_or(u16::MAX)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::Profile;

    fn hints() -> KeyHints {
        KeyHints::new(vec![
            KeyHint::new("↑↓", "move"),
            KeyHint::new("Enter", "select"),
            KeyHint::new("Esc", "cancel"),
        ])
    }

    fn plain(lines: &[Line<'_>]) -> Vec<String> {
        lines.iter().map(|l| l.to_string()).collect()
    }

    #[test]
    fn packs_on_one_row_when_it_fits() {
        let theme = Profile::DarkTrue.theme();
        assert_eq!(
            plain(&hints().lines(80, &theme)),
            ["↑↓ move · Enter select · Esc cancel"]
        );
    }

    #[test]
    fn chord_hints_use_the_key_label_authority() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
        let mac = crate::keys::Platform {
            macos: true,
            ascii: false,
        };
        let alt_v = KeyEvent::new(KeyCode::Char('v'), KeyModifiers::ALT);
        assert_eq!(KeyHint::chord(&alt_v, mac, "details").keys, "⌥V");
        let ctrl_o = KeyEvent::new(KeyCode::Char('o'), KeyModifiers::CONTROL);
        assert_eq!(KeyHint::chord(&ctrl_o, mac, "reasoning").keys, "Ctrl+O");
    }

    #[test]
    fn wraps_and_never_drops_an_action() {
        let theme = Profile::DarkTrue.theme();
        let rows = plain(&hints().lines(22, &theme));
        assert_eq!(rows, ["↑↓ move · Enter select", "Esc cancel"]);
        let rows = plain(&hints().lines(4, &theme));
        assert_eq!(rows.len(), 3, "every hint survives a degenerate width");
    }
}
