//! Panels with depth, and the one horizon.
//!
//! Depth is elevation: raised things sit nearer the surface, the stage and
//! things put away sit deeper. Where the terminal paints grounds, depth is a
//! ground; where it cannot (16 colors, `NO_COLOR`, an unmeasured ground),
//! depth becomes an edge, because a fill nobody can see separates nothing.
//!
//! Replaces the engine's three modal treatments (`render_modal_surface` with
//! its shadow, `render_underwater_surface` with its two rules, and hand-rolled
//! `Clear` blocks) and lifts `centered_modal_area` (`crates/tui/src/tui/
//! views/mod.rs`, `Hmbown/CodeWhale` `58b1dd3dd`).

use std::borrow::Cow;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

use crate::{KeyHints, Paint, Role, Theme, color, glyphs, text};

/// How far from the surface a panel sits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Depth {
    /// Rails and things put away.
    Deep,
    /// The stage: full-screen sheets. No edge; a two-cell gutter.
    Stage,
    /// A card on the stage.
    Raised,
    /// A decision over everything else: always edged in `BorderStrong`.
    Overlay,
}

impl Depth {
    #[must_use]
    pub const fn ground(self) -> Role {
        match self {
            Depth::Deep => Role::Sidebar,
            Depth::Stage => Role::Background,
            Depth::Raised | Depth::Overlay => Role::Surface,
        }
    }
}

/// A titled panel. [`Panel::draw`] paints it and returns the content area.
#[derive(Clone, Debug)]
pub struct Panel<'a> {
    pub title: Option<Cow<'a, str>>,
    /// Muted text at the right of the title row: a count, "changed 2 min ago".
    pub aside: Option<Cow<'a, str>>,
    pub depth: Depth,
    /// Light the edge in `Primary`: this panel has the keyboard.
    pub focused: bool,
    pub hints: Option<&'a KeyHints>,
}

impl<'a> Panel<'a> {
    #[must_use]
    pub fn new(depth: Depth) -> Self {
        Self {
            title: None,
            aside: None,
            depth,
            focused: false,
            hints: None,
        }
    }

    #[must_use]
    pub fn title(mut self, title: impl Into<Cow<'a, str>>) -> Self {
        self.title = Some(title.into());
        self
    }

    #[must_use]
    pub fn aside(mut self, aside: impl Into<Cow<'a, str>>) -> Self {
        self.aside = Some(aside.into());
        self
    }

    #[must_use]
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    #[must_use]
    pub fn hints(mut self, hints: &'a KeyHints) -> Self {
        self.hints = Some(hints);
        self
    }

    fn edged(&self, theme: &Theme) -> bool {
        match self.depth {
            Depth::Overlay => true,
            Depth::Raised => !theme.paints_grounds(),
            Depth::Deep | Depth::Stage => false,
        }
    }

    /// Paint the panel and return the area left for content.
    pub fn draw(&self, area: Rect, buf: &mut Buffer, theme: &Theme) -> Rect {
        if area.is_empty() {
            return area;
        }
        Clear.render(area, buf);
        let ground = theme.bg(self.depth.ground());
        buf.set_style(area, ground);

        let mut inner = area;
        if self.edged(theme) {
            let edge_role = if self.focused {
                Role::Primary
            } else if self.depth == Depth::Overlay {
                Role::BorderStrong
            } else {
                Role::Border
            };
            let set = if theme.ascii() {
                border::Set {
                    top_left: "+",
                    top_right: "+",
                    bottom_left: "+",
                    bottom_right: "+",
                    vertical_left: "|",
                    vertical_right: "|",
                    horizontal_top: "-",
                    horizontal_bottom: "-",
                }
            } else {
                border::PLAIN
            };
            let block = Block::default()
                .borders(Borders::ALL)
                .border_set(set)
                .border_style(theme.fg(edge_role).patch(ground));
            inner = block.inner(area);
            block.render(area, buf);
        }

        let gutter = match self.depth {
            Depth::Stage if inner.width >= 24 => 2,
            _ if inner.width >= 8 => 1,
            _ => 0,
        };
        let vpad = u16::from(self.edged(theme) && inner.height >= 6);
        inner = Rect {
            x: inner.x + gutter,
            y: inner.y + vpad,
            width: inner.width.saturating_sub(gutter * 2),
            height: inner.height.saturating_sub(vpad * 2),
        };

        if let Some(title) = &self.title
            && inner.height > 0
        {
            let row = Rect { height: 1, ..inner };
            let title = text::display_safe(title);
            let aside = self.aside.as_deref().map(text::display_safe);
            let aside_w = aside.as_deref().map_or(0, |a| text::width(a) + 2);
            let title_w = usize::from(row.width).saturating_sub(aside_w);
            let title = text::truncate(&title, title_w, theme.ascii());
            Line::from(Span::styled(
                title.into_owned(),
                theme.fg(Role::Foreground).add_modifier(Modifier::BOLD),
            ))
            .render(row, buf);
            if let Some(aside) = aside
                && aside_w > 0
                && aside_w <= usize::from(row.width)
            {
                Line::from(Span::styled(aside.into_owned(), theme.fg(Role::Muted)))
                    .right_aligned()
                    .render(row, buf);
            }
            let used = 1 + u16::from(inner.height >= 6);
            inner.y += used.min(inner.height);
            inner.height = inner.height.saturating_sub(used);
        }

        if let Some(hints) = self.hints {
            let lines = hints.lines(inner.width, theme);
            let h = u16::try_from(lines.len())
                .unwrap_or(u16::MAX)
                .min(inner.height);
            if h > 0 {
                let rail = Rect {
                    y: inner.bottom() - h,
                    height: h,
                    ..inner
                };
                Paragraph::new(lines).render(rail, buf);
                let gap = u16::from(inner.height >= h + 4);
                inner.height = inner.height.saturating_sub(h + gap);
            }
        }
        inner
    }
}

impl Paint for Panel<'_> {
    fn paint(&self, area: Rect, buf: &mut Buffer, theme: &Theme) {
        self.draw(area, buf, theme);
    }
}

/// A centered popup rect: starts from the preferred size, never exceeds the
/// frame (keeping a one-cell margin when it can), and never drops below the
/// minimum unless the frame is smaller. From the engine's
/// `centered_modal_area` (#3732).
#[must_use]
pub fn centered(
    area: Rect,
    preferred_width: u16,
    preferred_height: u16,
    min_width: u16,
    min_height: u16,
) -> Rect {
    let avail_width = area.width.saturating_sub(2).max(1);
    let avail_height = area.height.saturating_sub(2).max(1);
    let width = preferred_width.clamp(min_width.min(avail_width), avail_width);
    let height = preferred_height.clamp(min_height.min(avail_height), avail_height);
    Rect {
        x: area.x + area.width.saturating_sub(width) / 2,
        y: area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    }
}

/// The horizon: one full-width rule, drawn once per frame, above the place
/// where the person types. Sugimoto's seascapes, not a table border.
///
/// On a truecolor ground its ends fade into the ground, so it reads as a
/// horizon rather than a box edge. Everywhere else it is a plain `Border`
/// line; ASCII-safe terminals draw `-`.
#[derive(Clone, Debug, Default)]
pub struct HorizonRule<'a> {
    /// Muted words at the left: what the space below is for.
    pub label: Option<Cow<'a, str>>,
    /// Muted words at the right.
    pub aside: Option<Cow<'a, str>>,
}

impl<'a> HorizonRule<'a> {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn label(mut self, label: impl Into<Cow<'a, str>>) -> Self {
        self.label = Some(label.into());
        self
    }

    #[must_use]
    pub fn aside(mut self, aside: impl Into<Cow<'a, str>>) -> Self {
        self.aside = Some(aside.into());
        self
    }
}

impl Paint for HorizonRule<'_> {
    fn paint(&self, area: Rect, buf: &mut Buffer, theme: &Theme) {
        if area.is_empty() {
            return;
        }
        let row = Rect { height: 1, ..area };
        let rule = glyphs::pick("─", theme.ascii());
        let line_style = theme.fg(Role::Border);
        let fade = theme.depth() == color::ColorDepth::TrueColor && theme.paints_grounds();
        let width = row.width;
        let ramp = (width / 8).min(6);
        for i in 0..width {
            let mut style = line_style;
            if fade && ramp > 0 {
                let from_edge = i.min(width - 1 - i);
                if from_edge < ramp {
                    let alpha = f32::from(from_edge + 1) / f32::from(ramp + 1);
                    style = Style::default().fg(color::blend(
                        theme.token(Role::Border),
                        theme.token(Role::Background),
                        alpha,
                    ));
                }
            }
            buf[(row.x + i, row.y)].set_symbol(rule).set_style(style);
        }
        let muted = theme.fg(Role::Muted);
        if let Some(label) = &self.label {
            let label = text::display_safe(label);
            let budget = usize::from(width.saturating_sub(ramp.max(2) * 2 + 4)) / 2;
            let label = format!(" {} ", text::truncate(&label, budget, theme.ascii()));
            let x = row.x + ramp.max(2).min(width);
            buf.set_stringn(x, row.y, &label, usize::from(row.right() - x), muted);
        }
        if let Some(aside) = &self.aside {
            let aside = text::display_safe(aside);
            let budget = usize::from(width.saturating_sub(ramp.max(2) * 2 + 4)) / 2;
            let aside = format!(" {} ", text::truncate(&aside, budget, theme.ascii()));
            let w = u16::try_from(text::width(&aside)).unwrap_or(u16::MAX);
            let x = row.right().saturating_sub(w + ramp.max(2));
            if x > row.x {
                buf.set_stringn(x, row.y, &aside, usize::from(w), muted);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn centered_clamps_to_the_frame() {
        let area = Rect::new(0, 0, 80, 24);
        assert_eq!(centered(area, 68, 10, 44, 8), Rect::new(6, 7, 68, 10));
        let small = Rect::new(0, 0, 30, 6);
        let r = centered(small, 68, 10, 44, 8);
        assert!(r.width <= 28 && r.height <= 4);
    }
}
