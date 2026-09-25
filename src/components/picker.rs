//! Picker: the list behind the mode picker, the status-line picker, and
//! every other "choose one" or "choose some" list.
//!
//! A selected row carries three cues, because a fill alone measures only
//! about 1.3:1 against its neighbours: the `▸` marker lit in `Primary`, the
//! label in bold, and the `Selected` ground where grounds paint. Checked rows
//! show `●` and unchecked `○` (`[x]` and `[ ]` in ASCII), so on/off is a
//! shape, not a color. Disabled rows say why.
//!
//! Replaces the row rendering in the engine's `views/mode_picker.rs` and
//! `views/status_picker.rs`, and `menu_style::selected_row_style`
//! (`Hmbown/CodeWhale` `58b1dd3dd`).

use std::borrow::Cow;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Modifier,
    symbols::scrollbar,
    text::{Line, Span},
    widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Widget},
};

use crate::{Paint, Role, Theme, glyphs, text};

/// One row.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PickerItem {
    pub label: Cow<'static, str>,
    /// Muted words after the label: what choosing this does.
    pub detail: Option<Cow<'static, str>>,
    /// A shortcut shown before the label (`1`, `2`, `3`).
    pub key: Option<char>,
    /// `Some` makes this a checklist row.
    pub checked: Option<bool>,
    /// `Some(reason)` greys the row and shows the reason as its detail.
    pub disabled: Option<Cow<'static, str>>,
}

impl PickerItem {
    #[must_use]
    pub fn new(label: impl Into<Cow<'static, str>>) -> Self {
        Self {
            label: label.into(),
            ..Self::default()
        }
    }

    #[must_use]
    pub fn detail(mut self, detail: impl Into<Cow<'static, str>>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    #[must_use]
    pub fn key(mut self, key: char) -> Self {
        self.key = Some(key);
        self
    }

    #[must_use]
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = Some(checked);
        self
    }

    #[must_use]
    pub fn disabled(mut self, reason: impl Into<Cow<'static, str>>) -> Self {
        self.disabled = Some(reason.into());
        self
    }
}

/// Which row is selected and how far the list has scrolled.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PickerState {
    pub selected: usize,
    pub offset: usize,
}

impl PickerState {
    #[must_use]
    pub const fn new(selected: usize) -> Self {
        Self {
            selected,
            offset: 0,
        }
    }

    pub fn next(&mut self, len: usize) {
        if len > 0 {
            self.selected = (self.selected + 1) % len;
        }
    }

    pub fn prev(&mut self, len: usize) {
        if len > 0 {
            self.selected = (self.selected + len - 1) % len;
        }
    }

    pub fn home(&mut self) {
        self.selected = 0;
    }

    pub fn end(&mut self, len: usize) {
        self.selected = len.saturating_sub(1);
    }

    /// Move a page, clamped at the ends.
    pub fn page(&mut self, len: usize, rows: u16, down: bool) {
        let step = usize::from(rows.max(1));
        self.selected = if down {
            (self.selected + step).min(len.saturating_sub(1))
        } else {
            self.selected.saturating_sub(step)
        };
    }

    /// The offset that keeps the selection on screen in `rows` rows.
    #[must_use]
    pub fn visible_offset(&self, len: usize, rows: usize) -> usize {
        let rows = rows.max(1);
        let mut offset = self.offset.min(len.saturating_sub(rows));
        if self.selected < offset {
            offset = self.selected;
        } else if self.selected >= offset + rows {
            offset = self.selected + 1 - rows;
        }
        offset
    }

    /// Store the offset [`Picker`] painted with, so scrolling is stable.
    pub fn scroll_into_view(&mut self, len: usize, rows: u16) {
        self.offset = self.visible_offset(len, usize::from(rows));
    }
}

/// Spans plus the cells they cover.
#[derive(Default)]
struct Row {
    spans: Vec<Span<'static>>,
    used: usize,
}

impl Row {
    fn push(&mut self, s: String, style: ratatui::style::Style) {
        self.used += text::width(&s);
        self.spans.push(Span::styled(s, style));
    }
}

/// A list of [`PickerItem`]s.
#[derive(Clone, Copy, Debug)]
pub struct Picker<'a> {
    pub items: &'a [PickerItem],
    pub state: PickerState,
}

impl<'a> Picker<'a> {
    #[must_use]
    pub const fn new(items: &'a [PickerItem], state: PickerState) -> Self {
        Self { items, state }
    }

    /// The item under a click at `(column, row)` when the picker was painted
    /// in `area`, for hosts that take mouse input.
    #[must_use]
    pub fn row_at(&self, area: Rect, column: u16, row: u16) -> Option<usize> {
        let inside =
            column >= area.x && column < area.right() && row >= area.y && row < area.bottom();
        if !inside {
            return None;
        }
        let offset = self
            .state
            .visible_offset(self.items.len(), usize::from(area.height));
        let index = offset + usize::from(row - area.y);
        (index < self.items.len()).then_some(index)
    }

    fn label_column(&self) -> usize {
        self.items
            .iter()
            .map(|i| text::width(&text::display_safe(&i.label)))
            .max()
            .unwrap_or(0)
    }

    fn row(&self, index: usize, width: usize, label_col: usize, theme: &Theme) -> Line<'static> {
        let item = &self.items[index];
        let selected = index == self.state.selected;
        let ascii = theme.ascii();
        let disabled = item.disabled.is_some();

        let mut row = Row::default();

        let marker = glyphs::pick(glyphs::selection_marker(selected), ascii);
        row.push(format!("{marker} "), theme.fg(Role::Primary));
        if let Some(key) = item.key {
            row.push(format!("{key} "), theme.fg(Role::Muted));
        }
        if let Some(checked) = item.checked {
            let (mark, role) = match (checked, ascii) {
                (true, false) => (glyphs::CURRENT, Role::Foreground),
                (false, false) => (glyphs::AVAILABLE, Role::Muted),
                (true, true) => ("[x]", Role::Foreground),
                (false, true) => ("[ ]", Role::Muted),
            };
            row.push(format!("{mark} "), theme.fg(role));
        }

        let label_style = if disabled {
            theme.fg(Role::Muted).add_modifier(Modifier::DIM)
        } else if selected {
            theme.fg(Role::Foreground).add_modifier(Modifier::BOLD)
        } else {
            theme.fg(Role::Foreground)
        };
        let label = text::display_safe(&item.label);
        let label_w = label_col.min(width.saturating_sub(row.used));
        row.push(text::pad(&label, label_w, ascii), label_style);

        let detail = item.disabled.as_ref().or(item.detail.as_ref());
        if let Some(detail) = detail {
            let room = width.saturating_sub(row.used + 2);
            if room >= 4 {
                let detail = text::display_safe(detail);
                row.push("  ".into(), theme.fg(Role::Muted));
                row.push(
                    text::truncate(&detail, room, ascii).into_owned(),
                    theme.fg(Role::Muted),
                );
            }
        }

        let spans = row.spans;
        let mut line = Line::from(spans);
        if selected {
            line = line.style(theme.bg(Role::Selected));
        }
        line
    }
}

impl Paint for Picker<'_> {
    fn paint(&self, area: Rect, buf: &mut Buffer, theme: &Theme) {
        if area.is_empty() || self.items.is_empty() {
            return;
        }
        let rows = usize::from(area.height);
        let overflow = self.items.len() > rows;
        let list = Rect {
            width: area
                .width
                .saturating_sub(u16::from(overflow && area.width > 2)),
            ..area
        };
        let offset = self.state.visible_offset(self.items.len(), rows);
        let label_col = self.label_column();
        for (row, index) in (offset..self.items.len()).take(rows).enumerate() {
            let rect = Rect {
                y: list.y + row as u16,
                height: 1,
                ..list
            };
            if index == self.state.selected {
                buf.set_style(rect, theme.bg(Role::Selected));
            }
            self.row(index, usize::from(list.width), label_col, theme)
                .render(rect, buf);
        }
        if overflow && area.width > 2 {
            let (thumb, track) = if theme.ascii() {
                ("#", "|")
            } else {
                ("█", "│")
            };
            let mut state = ScrollbarState::new(self.items.len().saturating_sub(rows))
                .position(offset)
                .viewport_content_length(rows);
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .symbols(scrollbar::Set {
                    track,
                    thumb,
                    begin: track,
                    end: track,
                })
                .begin_symbol(None)
                .end_symbol(None)
                .thumb_style(theme.fg(Role::Foreground))
                .track_style(theme.fg(Role::Border))
                .render(area, buf, &mut state);
        }
    }

    fn height(&self, _width: u16, _theme: &Theme) -> u16 {
        u16::try_from(self.items.len()).unwrap_or(u16::MAX)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clicks_map_to_the_scrolled_row() {
        let items: Vec<PickerItem> = (0..10)
            .map(|i| PickerItem::new(format!("Item {i}")))
            .collect();
        let picker = Picker::new(&items, PickerState::new(9));
        let area = Rect::new(2, 3, 20, 4);
        assert_eq!(picker.row_at(area, 5, 3), Some(6));
        assert_eq!(picker.row_at(area, 5, 6), Some(9));
        assert_eq!(picker.row_at(area, 1, 3), None);
        assert_eq!(picker.row_at(area, 5, 7), None);
    }

    #[test]
    fn state_wraps_pages_and_keeps_selection_visible() {
        let mut s = PickerState::new(0);
        s.prev(3);
        assert_eq!(s.selected, 2);
        s.next(3);
        assert_eq!(s.selected, 0);
        s.page(20, 5, true);
        assert_eq!(s.selected, 5);
        assert_eq!(s.visible_offset(20, 4), 2);
        s.end(20);
        s.scroll_into_view(20, 4);
        assert_eq!(s.offset, 16);
        s.home();
        assert_eq!(s.visible_offset(20, 4), 0);
    }
}
