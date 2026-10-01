//! Gallery: toasts.

use ratatui::{buffer::Buffer, layout::Rect};

use super::Entry;
use crate::{Paint, State, Theme, Toast, Toasts};

fn toasts(area: Rect, buf: &mut Buffer, theme: &Theme) {
    Toasts::new(vec![
        Toast::new(State::Done, "Theme set to Shoreline"),
        Toast::new(State::NeedsYou, "A command is waiting for your approval").opens(),
        Toast::new(
            State::Failed,
            "Could not save: the settings file is read-only",
        )
        .opens(),
    ])
    .paint(area, buf, theme);
}

pub(crate) fn entries() -> Vec<Entry> {
    vec![Entry {
        name: "toasts",
        width: 60,
        height: 3,
        draw: toasts,
    }]
}
