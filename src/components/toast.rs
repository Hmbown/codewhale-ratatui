//! Toasts: one line, a mark and a sentence, stacked at the bottom right.
//!
//! The rendering half of the engine's `StatusToast` (`crates/tui/src/tui/
//! app/status.rs`, `Hmbown/CodeWhale` `58b1dd3dd`). How long a toast lives
//! stays with the host: failures should stay until the person has seen them.

use std::borrow::Cow;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::{Line, Span},
    widgets::Widget,
};

use crate::{Paint, Role, State, StatusMark, Theme, text};

/// One notice: `✓ Theme set to Shoreline`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Toast {
    pub state: State,
    pub text: Cow<'static, str>,
    /// Draws `→`: there is more to open.
    pub opens: bool,
}

impl Toast {
    #[must_use]
    pub fn new(state: State, text: impl Into<Cow<'static, str>>) -> Self {
        Self {
            state,
            text: text.into(),
            opens: false,
        }
    }

    #[must_use]
    pub fn opens(mut self) -> Self {
        self.opens = true;
        self
    }

    fn line(&self, max: usize, theme: &Theme) -> Line<'static> {
        let mark = StatusMark::new(self.state);
        let glyph = mark.glyph(theme);
        let arrow = if self.opens {
            if theme.ascii() { " >" } else { " →" }
        } else {
            ""
        };
        let fixed = 1 + text::width(glyph) + 1 + text::width(arrow) + 1;
        let body = text::display_safe(&self.text);
        let body =
            text::truncate_words(&body, max.saturating_sub(fixed), theme.ascii()).into_owned();
        let mut spans = vec![
            Span::raw(" "),
            Span::styled(glyph, theme.fg(self.state.role())),
            Span::raw(" "),
            Span::styled(body, theme.fg(Role::Foreground)),
        ];
        if !arrow.is_empty() {
            spans.push(Span::styled(arrow, theme.fg(Role::Muted)));
        }
        spans.push(Span::raw(" "));
        Line::from(spans)
    }
}

/// A stack of toasts anchored to the bottom right of an area, newest last
/// (nearest the horizon). Capped to the rows available.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Toasts {
    pub items: Vec<Toast>,
}

impl Toasts {
    #[must_use]
    pub fn new(items: Vec<Toast>) -> Self {
        Self { items }
    }
}

impl Paint for Toasts {
    fn paint(&self, area: Rect, buf: &mut Buffer, theme: &Theme) {
        if area.is_empty() {
            return;
        }
        let max = usize::from(area.width);
        let shown = self.items.len().min(usize::from(area.height));
        let first = self.items.len() - shown;
        for (row, toast) in self.items[first..].iter().enumerate() {
            let line = toast.line(max, theme);
            let w = u16::try_from(line.width())
                .unwrap_or(u16::MAX)
                .min(area.width);
            let rect = Rect {
                x: area.right() - w,
                y: area.bottom() - shown as u16 + row as u16,
                width: w,
                height: 1,
            };
            buf.set_style(rect, theme.bg(Role::Surface));
            line.render(rect, buf);
        }
    }

    fn height(&self, _width: u16, _theme: &Theme) -> u16 {
        u16::try_from(self.items.len()).unwrap_or(u16::MAX)
    }
}
