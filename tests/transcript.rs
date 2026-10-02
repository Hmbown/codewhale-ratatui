use std::borrow::Cow;

use codewhale_ratatui::{
    CodeBlock, Paint, Transcript, TranscriptAction, TranscriptBlock, TranscriptSpan,
    TranscriptSpanRole,
    testing::{self, Profile, render},
};
use ratatui::layout::Rect;

fn fixture() -> Transcript<'static> {
    Transcript::new(vec![
        TranscriptBlock::heading(1, "Session closeout"),
        TranscriptBlock::paragraph(vec![
            TranscriptSpan::plain("The "),
            TranscriptSpan::strong("workbar"),
            TranscriptSpan::plain(" folds TODO, context and price; "),
            TranscriptSpan::emphasis("nothing"),
            TranscriptSpan::plain(" is opened from here."),
        ]),
        TranscriptBlock::quote("Ship the quiet version first."),
        TranscriptBlock::list(vec![
            TranscriptSpan::plain("Run the narrow checks"),
            TranscriptSpan::plain("Read the receipt"),
        ]),
        TranscriptBlock::table(
            vec![Cow::Borrowed("Check"), Cow::Borrowed("State")],
            vec![
                vec![Cow::Borrowed("narrow layout"), Cow::Borrowed("kept")],
                vec![Cow::Borrowed("all profiles"), Cow::Borrowed("kept")],
            ],
        ),
        TranscriptBlock::code(CodeBlock::new(
            "rust",
            "let theme = Theme::detect();\nframe.render_widget(view.themed(&theme), area);",
        )),
    ])
}

#[test]
fn transcript_keeps_the_rules_in_every_profile_and_width() {
    testing::assert_rules(16, |area, buf, theme| {
        fixture().paint(area, buf, theme);
    });
}

#[test]
fn code_copy_text_keeps_the_exact_original_newlines() {
    let source = "fn main() {\r\n\tprintln!(\"hi\");\r\n}\n";
    let block = CodeBlock::new("rust", source);
    assert_eq!(block.source(), source);
    assert_eq!(block.copy_text(), source);
    assert!(block.copy_text().contains("\r\n"));
    assert!(block.copy_text().ends_with('\n'));
    assert_eq!(block.copy_action(), Some(TranscriptAction::Copy));
    assert_eq!(block.actions(), [TranscriptAction::Copy]);

    let theme = Profile::DarkTrue.theme();
    let buf = render(40, 8, |area, buf| block.paint(area, buf, &theme));
    let shown = testing::text(&buf);
    assert!(shown.contains("rust"));
    assert!(shown.contains("Copy"));
    assert!(shown.contains("fn main() {"));
    assert!(!shown.contains('\r'));
}

#[test]
fn a_display_copy_never_changes_what_the_host_would_copy() {
    let block = CodeBlock::new("text", "exact\nsource\n").display("shown text");
    assert_eq!(block.copy_text(), "exact\nsource\n");
    let theme = Profile::DarkTrue.theme();
    let buf = render(40, 4, |area, buf| block.paint(area, buf, &theme));
    let shown = testing::text(&buf);
    assert!(shown.contains("shown text"));
    assert!(!shown.contains("source"));

    let bare = CodeBlock::new("text", "x")
        .copyable(false)
        .copy_label("Copy");
    assert_eq!(bare.copy_action(), None);
    assert!(bare.actions().is_empty());
    let buf = render(40, 2, |area, buf| bare.paint(area, buf, &theme));
    assert!(!testing::text(&buf).contains("Copy"));
}

#[test]
fn unsafe_display_is_sanitized_and_link_targets_stay_out_of_band() {
    let transcript = Transcript::new(vec![
        TranscriptBlock::paragraph(vec![TranscriptSpan::plain(
            "run rm -rf ~/\u{202E}txt.exe now",
        )]),
        TranscriptBlock::paragraph(vec![
            TranscriptSpan::plain("Read "),
            TranscriptSpan::link("the component guide", "https://example.invalid/guide"),
            TranscriptSpan::plain(" first."),
        ]),
    ]);
    let theme = Profile::DarkTrue.theme();
    let buf = render(60, 6, |area, buf| transcript.paint(area, buf, &theme));
    let shown = testing::text(&buf);
    assert!(shown.contains("rm -rf ~/txt.exe now"), "{shown}");
    assert!(!shown.contains('\u{202e}'));
    assert!(shown.contains("the component guide"));
    assert!(!shown.contains("example.invalid"));
    assert!(!shown.contains("https"));
    assert!(!shown.contains('\u{1b}'));
    let styled = testing::styled(&buf, &theme);
    assert!(styled.contains("underline"), "{styled}");

    let links = transcript.links();
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].text, "the component guide");
    assert_eq!(links[0].target, "https://example.invalid/guide");
}

#[test]
fn caller_semantic_roles_are_painted_as_given() {
    let theme = Profile::DarkTrue.theme();
    let spans: Vec<TranscriptSpan<'static>> = TranscriptSpanRole::ALL
        .iter()
        .map(|role| TranscriptSpan::new("x", *role))
        .collect();
    let block = TranscriptBlock::paragraph(spans);
    let buf = render(40, 2, |area, buf| block.paint(area, buf, &theme));
    let styled = testing::styled(&buf, &theme);
    assert!(styled.contains("underline"), "{styled}");
    assert!(styled.contains("bold"), "{styled}");
    assert!(styled.contains("italic"), "{styled}");
}

#[test]
fn a_requested_offset_and_height_clip_the_painted_rows() {
    let transcript = Transcript::new(vec![
        TranscriptBlock::heading(1, "First"),
        TranscriptBlock::heading(2, "Second"),
    ])
    .offset(1);
    let theme = Profile::DarkTrue.theme();
    let buf = render(40, 1, |area, buf| transcript.paint(area, buf, &theme));
    let shown = testing::text(&buf);
    assert!(shown.contains("Second"), "{shown}");
    assert!(!shown.contains("First"));
    assert_eq!(
        transcript.lines(40, &theme).len(),
        usize::from(transcript.height(40, &theme))
    );
}

#[test]
fn narrow_unicode_code_and_tables_stay_inside_the_requested_width() {
    let transcript = Transcript::new(vec![
        TranscriptBlock::code(CodeBlock::new("rust", "鲸".repeat(12))),
        TranscriptBlock::table(
            vec![Cow::Borrowed("文件"), Cow::Borrowed("state")],
            vec![vec![Cow::Borrowed("提交记录"), Cow::Borrowed("queued")]],
        ),
        TranscriptBlock::list(vec![TranscriptSpan::plain("鲸鱼".repeat(20))]),
    ]);
    let theme = Profile::DarkTrue.theme();
    for width in [12u16, 20, 40] {
        let buf = render(width, 24, |area, buf| transcript.paint(area, buf, &theme));
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
    let transcript = fixture();
    let theme = Profile::DarkTrue.theme();
    let buf = render(20, 4, |_, buf| {
        transcript.paint(Rect::new(30, 10, 24, 6), buf, &theme);
    });
    assert!(testing::text(&buf).trim().is_empty());
}

#[test]
fn structured_blocks_render_their_own_marks() {
    let theme = Profile::DarkTrue.theme();
    let buf = render(60, 16, |area, buf| fixture().paint(area, buf, &theme));
    let shown = testing::text(&buf);
    assert!(shown.contains("Session closeout"));
    assert!(shown.contains("Ship the quiet version first."));
    assert!(shown.contains("• Run the narrow checks"));
    assert!(shown.contains("Check"));
    assert!(shown.contains("narrow layout"));
    assert!(shown.contains("rust"));
    assert!(shown.contains("Copy"));

    let ordered = TranscriptBlock::ordered(vec![TranscriptSpan::plain("one")]);
    let buf = render(40, 1, |area, buf| ordered.paint(area, buf, &theme));
    assert!(testing::text(&buf).contains("1. one"));
}

#[test]
fn wrapping_retains_all_words_and_code_keeps_logical_lines() {
    let theme = Profile::DarkTrue.theme();
    let block = TranscriptBlock::prose("alpha beta gamma delta epsilon zeta");
    let lines = block.lines(12, &theme);
    assert!(lines.len() >= 3);
    let visible: String = lines
        .iter()
        .map(|line| line.to_string())
        .collect::<Vec<_>>()
        .join(" ");
    for word in ["alpha", "beta", "gamma", "delta", "epsilon", "zeta"] {
        assert!(visible.contains(word), "lost {word}: {visible}");
    }
    let code = CodeBlock::new("rust", "first();\n\tsecond();\nthird();");
    let shown = code.lines(40, &theme);
    assert_eq!(shown.len(), 4, "header plus three numbered source lines");
    assert!(shown[1].to_string().contains("1 │"));
    assert!(shown[2].to_string().contains("2 │"));
    assert!(shown[2].to_string().contains("    second();"));
    assert!(shown[3].to_string().contains("3 │"));
    let wide = TranscriptBlock::prose("鲸鱼".repeat(10));
    let lines = wide.lines(8, &theme);
    assert_eq!(
        lines.iter().map(|l| l.to_string()).collect::<String>(),
        "鲸鱼".repeat(10)
    );
    assert!(lines.iter().all(|line| line.width() <= 8));
}

#[test]
fn narrow_code_elides_chrome_before_hiding_source_and_tables_keep_separation() {
    let theme = Profile::DarkTrue.theme();
    for width in [2, 4, 6, 8] {
        let lines = CodeBlock::new("", "x鲸")
            .copyable(false)
            .lines(width, &theme);
        let shown = lines
            .iter()
            .map(|line| line.to_string())
            .collect::<String>();
        assert!(
            shown.contains('x'),
            "source hidden by gutter at {width}: {shown}"
        );
        assert!(lines.iter().all(|line| line.width() <= usize::from(width)));
    }
    let table = TranscriptBlock::table(vec!["value".into()], vec![vec!["first\nsecond".into()]]);
    assert!(
        table
            .lines(40, &theme)
            .iter()
            .any(|line| line.to_string().contains("first second"))
    );
}
