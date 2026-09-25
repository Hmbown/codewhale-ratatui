//! Codewhale's terminal component library for ratatui.
//!
//! Theme roles, glyphs and key labels shared by the Codewhale terminal UI,
//! built from the same design tokens as the Codewhale desktop app and website
//! (`vendor/codewhale-design`; see [`theme::TOKENS_VERSION`]).

pub mod color;
pub mod detect;
pub mod glyphs;
pub mod keys;
pub mod osc11;
mod roles;
pub mod text;
pub mod theme;
/// The vendored Codewhale design tokens (`vendor/codewhale-design/tokens.rs`):
/// spacing, type and motion constants alongside the colors.
#[path = "../vendor/codewhale-design/tokens.rs"]
pub mod tokens;

pub mod testing;

pub use theme::{Caps, Role, Theme};
