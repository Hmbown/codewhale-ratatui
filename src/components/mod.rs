//! The components. Each one paints with a [`crate::Theme`] through
//! [`crate::Paint`], names roles rather than colors, and pairs every state
//! with a mark and a word.
//!
//! Every module is re-exported whole (`pub use module::*`), so a component's
//! author makes items `pub` in the component's own file and never edits this
//! file or `lib.rs`. `spinner` is the exception: its constants and functions
//! live under [`spin`] and are exported by name below.

mod hints;
mod icons;
mod picker;
mod spinner;
mod status;
mod surface;
mod toast;

pub use hints::*;
pub use icons::*;
pub use picker::*;
pub use spinner::{MotionMode, Spinner, duration};
pub use status::*;
pub use surface::*;
pub use toast::*;

/// Spinner frames and cadence.
pub mod spin {
    pub use super::spinner::{
        EARN_DELAY, FRAME_INTERVAL, FRAMES, PENDING_FRAME, STILL_FRAME, frame, next_frame_in,
    };
}

// Modules reserved for the packages that will fill them. Each file is empty
// until its package lands, so its glob re-export has nothing to export yet.

// Package Input.
mod form;
mod text_input;
#[allow(unused_imports)]
pub use form::*;
#[allow(unused_imports)]
pub use text_input::*;

// Package Lists.
mod empty;
mod fuzzy;
mod list;
#[allow(unused_imports)]
pub use empty::*;
#[allow(unused_imports)]
pub use fuzzy::*;
#[allow(unused_imports)]
pub use list::*;

// Package Chrome.
mod heading;
mod keymap;
mod segmented;
mod tabs;
mod toggle;
#[allow(unused_imports)]
pub use heading::*;
#[allow(unused_imports)]
pub use keymap::*;
#[allow(unused_imports)]
pub use segmented::*;
#[allow(unused_imports)]
pub use tabs::*;
#[allow(unused_imports)]
pub use toggle::*;

// Package Display.
mod diff;
mod progress;
mod receipt;
mod tree;
#[allow(unused_imports)]
pub use diff::*;
#[allow(unused_imports)]
pub use progress::*;
#[allow(unused_imports)]
pub use receipt::*;
#[allow(unused_imports)]
pub use tree::*;

// Package Approval.
mod approval;
#[allow(unused_imports)]
pub use approval::*;

// Package Motion.
mod motion;
#[allow(unused_imports)]
pub use motion::*;
