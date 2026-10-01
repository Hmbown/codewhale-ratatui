//! Gallery entries for package Display: the receipt, diff, tree and progress.
//!
//! Add one `Entry` per component state worth seeing (normal, empty, error,
//! narrow, disabled) to `entries()`. This is the only gallery file the
//! package edits; `gallery/mod.rs` already calls it.

use super::Entry;

pub(crate) fn entries() -> Vec<Entry> {
    Vec::new()
}
