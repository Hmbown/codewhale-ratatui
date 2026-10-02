//! Every component with fixture data, for `examples/gallery.rs` and the
//! snapshot tests. The fixtures double as usage examples: the mode and
//! status-line pickers here are the engine's `/mode` and `/statusline`
//! pickers rebuilt from kit parts.
//!
//! One file per area, each exposing `pub(crate) fn entries() -> Vec<Entry>`.
//! [`entries`] concatenates them, so a package edits only its own file:
//!
//! | File | Holds |
//! |---|---|
//! | `hints`, `status`, `surface`, `picker`, `toast`, `icons`, `spinner`, `whale` | the components that exist |
//! | `input` | package Input: text input and form |
//! | `lists` | package Lists: list, fuzzy picker, empty state |
//! | `chrome` | package Chrome: heading, tabs, toggle, segmented, keymap |
//! | `display` | package Display: receipt, diff, tree, progress |
//! | `approval` | package Approval: approval card |
//! | `motion` | package Motion |
//! | `workspace` | messages, composer, tool and agent cards, fleets |
//! | `settings` | values, source, locks, apply timing and defaults |

use crossterm::event::KeyCode;
use ratatui::{buffer::Buffer, layout::Rect};

use crate::{
    Role, Theme,
    keys::{Platform, pair_label},
};

mod approval;
mod artifacts;
mod attention;
mod chrome;
mod display;
mod habitat;
mod hints;
mod icons;
mod input;
mod lists;
mod motion;
mod picker;
mod scenes;
mod settings;
mod spinner;
mod status;
mod surface;
mod toast;
mod verification;
mod whale;
mod workbench;
mod workspace;

/// One gallery entry: a name, the size it is drawn at by default, and a
/// function that paints it. `draw` must paint inside the `Rect` it is given
/// (the gallery draws it at other widths too) and use only the `Theme`.
pub struct Entry {
    pub name: &'static str,
    pub width: u16,
    pub height: u16,
    pub draw: fn(Rect, &mut Buffer, &Theme),
}

/// `↑↓`, or `Up/Down` where marks are ASCII-safe: one spelling for the
/// gallery's key hints.
fn arrows(theme: &Theme) -> String {
    pair_label(KeyCode::Up, KeyCode::Down, Platform::current(theme.ascii()))
}

/// Every entry, in gallery order.
#[must_use]
pub fn entries() -> Vec<Entry> {
    [
        scenes::entries(),
        workbench::entries(),
        attention::entries(),
        artifacts::entries(),
        habitat::entries(),
        hints::entries(),
        status::entries(),
        surface::entries(),
        picker::entries(),
        toast::entries(),
        icons::entries(),
        spinner::entries(),
        verification::entries(),
        whale::entries(),
        input::entries(),
        lists::entries(),
        chrome::entries(),
        display::entries(),
        approval::entries(),
        motion::entries(),
        workspace::entries(),
        settings::entries(),
    ]
    .into_iter()
    .flatten()
    .collect()
}

/// Render one entry for one theme at its own size.
#[must_use]
pub fn render(entry: &Entry, theme: &Theme) -> Buffer {
    render_at(entry, theme, entry.width, entry.height)
}

/// Render one entry for one theme at another size, on the theme's
/// `Background` as the gallery paints it.
#[must_use]
pub fn render_at(entry: &Entry, theme: &Theme, width: u16, height: u16) -> Buffer {
    crate::testing::render(width, height, |area, buf| {
        buf.set_style(area, theme.bg(Role::Background));
        (entry.draw)(area, buf, theme);
    })
}
