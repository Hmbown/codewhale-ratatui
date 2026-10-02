use codewhale_ratatui::{
    ContextPreviewItem, ContextPreviewState, Paint, PendingInputAction, PendingInputItem,
    PendingInputPreview, PendingInputStatus,
    testing::{self, Profile, render},
};
use ratatui::layout::Rect;

fn fixture() -> PendingInputPreview<'static> {
    PendingInputPreview::new(
        vec![
            PendingInputItem::new("q-1", "First queued message", PendingInputStatus::Queued),
            PendingInputItem::new("s-1", "A steering note", PendingInputStatus::Steering),
            PendingInputItem::new("p-1", "A paused follow-up", PendingInputStatus::Paused),
            PendingInputItem::new("e-1", "An editing follow-up", PendingInputStatus::Editing),
            PendingInputItem::new("f-1", "Already on its way", PendingInputStatus::InFlight),
        ],
        vec![
            ContextPreviewItem::new("c-1", "notes.md", ContextPreviewState::Included),
            ContextPreviewItem::new("c-2", "draft.md", ContextPreviewState::Unconfirmed),
            ContextPreviewItem::new("c-3", "old.log", ContextPreviewState::Removable),
        ],
    )
    .selected("q-1")
}

#[test]
fn pending_input_keeps_the_rules_in_every_profile_and_width() {
    testing::assert_rules(6, |area, buf, theme| {
        fixture().paint(area, buf, theme);
    });
}

#[test]
fn an_empty_preview_asks_for_zero_rows_and_paints_nothing() {
    let empty = PendingInputPreview::new(Vec::new(), Vec::new());
    for profile in Profile::ALL {
        assert_eq!(Paint::height(&empty, 40, &profile.theme()), 0);
    }
    let theme = Profile::DarkTrue.theme();
    let buf = render(40, 3, |area, buf| empty.paint(area, buf, &theme));
    assert!(testing::text(&buf).trim().is_empty());
}

#[test]
fn the_one_row_narrow_view_keeps_an_action_discoverable() {
    let theme = Profile::DarkTrue.theme();
    let buf = render(40, 1, |area, buf| fixture().paint(area, buf, &theme));
    let shown = testing::text(&buf);
    assert!(shown.contains("Send now"), "{shown}");
    assert!(codewhale_ratatui::text::width(&shown) <= 40);
}

#[test]
fn status_words_and_context_states_are_explicit() {
    let theme = Profile::DarkTrue.theme();
    let buf = render(80, 8, |area, buf| fixture().paint(area, buf, &theme));
    let shown = testing::text(&buf);
    for status in PendingInputStatus::ALL {
        assert!(
            shown.contains(status.word()),
            "missing {}: {shown}",
            status.word()
        );
    }
    for state in ContextPreviewState::ALL {
        assert!(
            shown.contains(state.word()),
            "missing {}: {shown}",
            state.word()
        );
    }
    // No false delivered or verified claim anywhere.
    for banned in ["sent", "delivered", "verified", "failed"] {
        assert!(!shown.contains(banned), "{banned} in {shown}");
    }
}

#[test]
fn actions_are_metadata_and_in_flight_items_offer_none() {
    let preview = fixture();
    assert_eq!(
        preview.actions_for("q-1"),
        Some(PendingInputAction::ALL.as_slice())
    );
    assert_eq!(preview.selected_index(), Some(0));
    assert!(
        preview
            .actions_for("f-1")
            .is_some_and(|actions| actions.is_empty())
    );
    assert!(preview.actions_for("absent").is_none());
    assert_eq!(preview.actions(), PendingInputAction::ALL.to_vec());
    assert_eq!(preview.pending_count(), 5);
}

#[test]
fn caller_text_is_sanitized_before_it_reaches_the_cells() {
    let item = PendingInputItem::new(
        "x",
        "rm -rf ~/\u{202E}txt.exe\u{1b}[31m\u{9b}31m",
        PendingInputStatus::Queued,
    );
    let preview = PendingInputPreview::new(vec![item], Vec::new());
    let theme = Profile::DarkTrue.theme();
    let buf = render(60, 3, |area, buf| preview.paint(area, buf, &theme));
    let shown = testing::text(&buf);
    assert!(shown.contains("rm -rf ~/txt.exe"), "{shown}");
    assert!(!shown.contains('\u{202e}'));
    assert!(!shown.contains('\u{1b}'));
    assert!(!shown.contains('\u{9b}'));
}

#[test]
fn cjk_and_long_lines_stay_inside_the_requested_width() {
    let preview = PendingInputPreview::new(
        vec![PendingInputItem::new(
            "cjk",
            "鲸鱼".repeat(30),
            PendingInputStatus::Queued,
        )],
        Vec::new(),
    );
    let theme = Profile::DarkTrue.theme();
    for width in [12u16, 24, 40] {
        let buf = render(width, 4, |area, buf| preview.paint(area, buf, &theme));
        for line in testing::text(&buf).lines() {
            assert!(
                codewhale_ratatui::text::width(line) <= usize::from(width),
                "{width}: {line}"
            );
        }
    }
}

#[test]
fn painting_outside_the_buffer_area_writes_nothing() {
    let preview = fixture();
    let theme = Profile::DarkTrue.theme();
    let buf = render(20, 4, |_, buf| {
        preview.paint(Rect::new(28, 9, 40, 6), buf, &theme);
    });
    assert!(testing::text(&buf).trim().is_empty());
}

#[test]
fn requested_height_bounds_the_painted_rows_and_the_rest_is_counted() {
    let preview = fixture();
    let theme = Profile::DarkTrue.theme();
    let buf = render(40, 5, |area, buf| preview.paint(area, buf, &theme));
    let shown = testing::text(&buf);
    assert!(shown.lines().count() <= 5);
    assert!(shown.contains("+5 more"), "{shown}");
    assert!(shown.contains("Send now"));
}

#[test]
fn row_limit_and_multiline_summary_keep_the_callers_intent() {
    let theme = Profile::DarkTrue.theme();
    let item = PendingInputItem::new("id", "first\nsecond", PendingInputStatus::Queued);
    let preview = PendingInputPreview::new(vec![item], vec![]).max_rows(0);
    let buf = render(60, 8, |area, buf| preview.paint(area, buf, &theme));
    assert!(testing::text(&buf).trim().is_empty());
    let preview = preview.max_rows(3);
    let buf = render(60, 8, |area, buf| preview.paint(area, buf, &theme));
    assert!(testing::text(&buf).contains("first second"));
    assert!(
        testing::text(&buf)
            .lines()
            .skip(3)
            .all(|line| line.trim().is_empty())
    );
}
