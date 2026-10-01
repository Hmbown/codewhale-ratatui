//! Package Approval: the approval card.
//!
//! This file belongs to the package. Add each component's tests here:
//! snapshots with `insta::assert_snapshot!(name, testing::snapshot(height,
//! paint))` and the frame rules with `testing::assert_rules(height, paint)`,
//! both at 40, 80 and 120 columns in every profile. Snapshots land in
//! `tests/snapshots/<name>.snap`; review them with `cargo insta review`.

use codewhale_ratatui::testing;

/// The rule check runs over every profile at every width. An empty frame
/// keeps every rule; replace it with the real component as it lands.
#[test]
fn the_frame_rules_hold_in_every_profile_at_every_width() {
    testing::assert_rules(3, |_area, _buf, _theme| {});
}
