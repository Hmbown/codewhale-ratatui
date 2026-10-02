//! Authored transcript fixtures: prose with semantic spans, a quote, a list,
//! a table, fenced code and linked text. The host owns parsing and copying.

use std::borrow::Cow;

use ratatui::{buffer::Buffer, layout::Rect};

use super::Entry;
use crate::{CodeBlock, Paint, Theme, Transcript, TranscriptBlock, TranscriptSpan};

fn prose(area: Rect, buf: &mut Buffer, theme: &Theme) {
    Transcript::new(vec![
        TranscriptBlock::heading(1, "Release notes"),
        TranscriptBlock::paragraph(vec![
            TranscriptSpan::plain("The "),
            TranscriptSpan::strong("workbar"),
            TranscriptSpan::plain(" is summoned, and the "),
            TranscriptSpan::emphasis("habitat"),
            TranscriptSpan::plain(" stays quiet under "),
            TranscriptSpan::code("MotionMode::Reduced"),
            TranscriptSpan::plain("."),
        ]),
        TranscriptBlock::paragraph(vec![
            TranscriptSpan::muted("Reported by the host. "),
            TranscriptSpan::success("Receipts read."),
        ]),
        TranscriptBlock::quote_by("Ship the quiet version first.", "a review note"),
    ])
    .paint(area, buf, theme);
}

fn list_and_table(area: Rect, buf: &mut Buffer, theme: &Theme) {
    Transcript::new(vec![
        TranscriptBlock::heading(2, "Before landing"),
        TranscriptBlock::ordered(vec![
            TranscriptSpan::plain("Run the narrow layout checks"),
            TranscriptSpan::plain("Read the receipt and the diff"),
            TranscriptSpan::warning("Confirm the 40-column view"),
        ]),
        TranscriptBlock::table(
            vec![Cow::Borrowed("Check"), Cow::Borrowed("State")],
            vec![
                vec![Cow::Borrowed("narrow layout"), Cow::Borrowed("kept")],
                vec![Cow::Borrowed("all profiles"), Cow::Borrowed("kept")],
            ],
        ),
    ])
    .paint(area, buf, theme);
}

fn fenced_code(area: Rect, buf: &mut Buffer, theme: &Theme) {
    let block = CodeBlock::new(
        "rust",
        "let theme = Theme::detect();\nframe.render_widget(view.themed(&theme), area);",
    );
    Transcript::new(vec![
        TranscriptBlock::heading(2, "Paint into the frame"),
        TranscriptBlock::code(block),
    ])
    .paint(area, buf, theme);
}

fn linked(area: Rect, buf: &mut Buffer, theme: &Theme) {
    Transcript::new(vec![
        TranscriptBlock::paragraph(vec![
            TranscriptSpan::plain("Read the "),
            TranscriptSpan::link("component guide", "https://example.invalid/guide"),
            TranscriptSpan::plain(" before adding a component."),
        ]),
        TranscriptBlock::paragraph(vec![TranscriptSpan::muted(
            "Links are metadata; the host dispatches.",
        )]),
    ])
    .paint(area, buf, theme);
}

pub(crate) fn entries() -> Vec<Entry> {
    vec![
        Entry {
            name: "transcript-prose",
            width: 40,
            height: 10,
            draw: prose,
        },
        Entry {
            name: "transcript-list-table",
            width: 40,
            height: 9,
            draw: list_and_table,
        },
        Entry {
            name: "transcript-code",
            width: 40,
            height: 6,
            draw: fenced_code,
        },
        Entry {
            name: "transcript-links",
            width: 40,
            height: 5,
            draw: linked,
        },
    ]
}
