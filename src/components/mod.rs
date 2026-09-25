//! The components. Each one paints with a [`crate::Theme`] through
//! [`crate::Paint`], names roles rather than colors, and pairs every state
//! with a mark and a word.

mod hints;
mod icons;
mod picker;
mod spinner;
mod status;
mod surface;
mod toast;

pub use hints::{KeyHint, KeyHints};
pub use icons::Icon;
pub use picker::{Picker, PickerItem, PickerState};
pub use spinner::{MotionMode, Spinner, duration};
pub use status::{State, StatusMark};
pub use surface::{Depth, HorizonRule, Panel, centered};
pub use toast::{Toast, Toasts};

/// Spinner frames and cadence.
pub mod spin {
    pub use super::spinner::{
        EARN_DELAY, FRAME_INTERVAL, FRAMES, PENDING_FRAME, STILL_FRAME, frame, next_frame_in,
    };
}
