//! Codewhale's terminal component library for ratatui.
//!
//! Components, theme roles and marks shared by the Codewhale terminal UI,
//! built from the same design tokens as the Codewhale desktop app and website
//! (`vendor/codewhale-design`; see [`theme::TOKENS_VERSION`]).
//!
//! ```no_run
//! use codewhale_ratatui::{Paint, Theme, KeyHint, KeyHints};
//! # fn draw(frame: &mut ratatui::Frame) {
//! let theme = Theme::detect();
//! let hints = KeyHints::new(vec![
//!     KeyHint::new("↑↓", "move"),
//!     KeyHint::new("Enter", "select"),
//!     KeyHint::new("Esc", "cancel"),
//! ]);
//! frame.render_widget(hints.themed(&theme), frame.area());
//! # }
//! ```

pub mod color;
pub mod detect;
pub mod glyphs;
pub mod keys;
pub mod osc11;
#[rustfmt::skip]
mod roles;
pub mod text;
pub mod theme;
/// The vendored Codewhale design tokens (`vendor/codewhale-design/tokens.rs`):
/// spacing, type and motion constants alongside the colors.
#[path = "../vendor/codewhale-design/tokens.rs"]
pub mod tokens;

mod components;
pub mod gallery;
pub mod testing;
pub mod whale;

pub use components::*;
pub use theme::{Caps, Ground, Role, Theme};
pub use whale::{Whale, WhaleState};

use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

/// A component that paints with a [`Theme`].
///
/// Components hold only what they show. The theme arrives when they paint,
/// so nothing caches a color and a theme change reaches every component on
/// the next frame.
pub trait Paint {
    fn paint(&self, area: Rect, buf: &mut Buffer, theme: &Theme);

    /// Rows this component wants at `width`.
    fn height(&self, _width: u16, _theme: &Theme) -> u16 {
        1
    }

    /// Wrap as a ratatui [`Widget`] for `frame.render_widget`.
    fn themed<'a>(&'a self, theme: &'a Theme) -> Themed<'a, Self>
    where
        Self: Sized,
    {
        Themed {
            component: self,
            theme,
        }
    }
}

/// A component paired with the theme it paints with.
pub struct Themed<'a, P: Paint> {
    component: &'a P,
    theme: &'a Theme,
}

impl<P: Paint> Widget for Themed<'_, P> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.component.paint(area, buf, self.theme);
    }
}
