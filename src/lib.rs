//! Codewhale's terminal component library for ratatui.
//!
//! Reusable components from Codewhale's current terminal UI. Native palettes
//! and layout are available through [`Theme::tui`], [`NativeComposer`],
//! [`Workbar`] and [`TerminalShell`]. Desktop token themes and additional
//! compositions remain available.
//!
//! ```no_run
//! use codewhale_ratatui::{Paint, Theme, KeyHint, KeyHints};
//! # fn draw(frame: &mut ratatui::Frame) {
//! let theme = Theme::detect().tui();
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
pub mod ocean;
pub mod tui_palettes;
pub use tui_palettes::{TuiGround, TuiInk, TuiPalette};
pub mod ombre;
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
pub mod whale_motion;

// Every component module is re-exported whole (see `components/mod.rs`), so
// a package that makes an item `pub` in its own file exports it from here
// without touching this file.
pub use components::*;
pub use ocean::{OceanColumn, OceanPhase, OceanRamp};
pub use ombre::{Ombre, OmbreDirection, WaterPalette};
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
