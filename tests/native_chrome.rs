use std::time::Duration;

use codewhale_ratatui::{
    NativeComposer, NativeComposerDensity, Paint, TuiPalette, WorkflowProgress, WorkflowRun,
    WorkflowRunState,
    testing::{Profile, render, text},
};
use ratatui::{buffer::Buffer, layout::Rect, style::Color};

#[test]
fn running_progress_matches_the_current_native_workbar_row() {
    let theme = Profile::DarkTrue.theme();
    let progress = WorkflowProgress::new(vec![
        WorkflowRun::new("Compare Cline with Codewhale", WorkflowRunState::Running)
            .outcomes(4, 1, 0, 10)
            .elapsed(Duration::from_secs(134))
            .tokens(1_234_567),
    ]);
    let buf = render(110, 1, |area, buf| progress.paint(area, buf, &theme));
    assert_eq!(
        text(&buf),
        " • Compare Cline with Codewhale  ████████××░░░░░░░░░░  4/10 done · 1 failed  2m 14s  ↓1.2M"
    );
    let narrow = render(40, 1, |area, buf| progress.paint(area, buf, &theme));
    let narrow = text(&narrow);
    assert!(
        narrow.contains("4/10 done") && narrow.contains("1 failed"),
        "{narrow}"
    );
    assert!(!narrow.contains(['█', '×', '░', '↓']), "{narrow}");
}

#[test]
fn failed_agents_never_fill_the_success_segment_or_round_fast_work_to_zero() {
    let theme = Profile::DarkTrue.theme();
    let progress = WorkflowProgress::new(vec![
        WorkflowRun::new("Release-readiness audit", WorkflowRunState::Failed)
            .outcomes(0, 2, 0, 2)
            .elapsed(Duration::from_millis(355))
            .reason("[auth] Authorization failed: sign in again. Nothing ran."),
    ]);
    let buf = render(140, 1, |area, buf| progress.paint(area, buf, &theme));
    let shown = text(&buf);
    assert_eq!(shown.matches('×').count(), 20, "{shown}");
    assert!(!shown.contains('█'));
    assert!(shown.contains("0/2 done · 2 failed"));
    assert!(shown.contains("355ms"));
    assert!(shown.contains("Authorization failed: sign in again"));
    assert!(!shown.contains("[auth]") && !shown.contains("Nothing ran"));
    assert!(
        buf.content
            .iter()
            .filter(|c| c.symbol() == "×")
            .all(|c| c.fg == theme.color(codewhale_ratatui::Role::Danger).unwrap())
    );
}

#[test]
fn queued_large_and_folded_run_facts_follow_native_thresholds() {
    let theme = Profile::DarkTrue.theme();
    let large = WorkflowProgress::new(vec![
        WorkflowRun::new("large", WorkflowRunState::Running)
            .outcomes(0, 1, 0, 25)
            .queued(2),
    ]);
    let buf = render(120, 1, |area, buf| large.paint(area, buf, &theme));
    let shown = text(&buf);
    assert!(shown.contains("Large workflow") && shown.contains("2 queued"));
    assert!(
        shown.contains('×'),
        "one failed agent of 25 still owns a visible cell"
    );
    let small = WorkflowProgress::new(vec![
        WorkflowRun::new("small", WorkflowRunState::Running).outcomes(0, 0, 0, 24),
    ]);
    let buf = render(120, 1, |area, buf| small.paint(area, buf, &theme));
    assert!(!text(&buf).contains("Large workflow"));
    assert!(!text(&buf).contains("queued"));
    let runs = (0..12)
        .map(|n| {
            WorkflowRun::new(format!("run {n}"), WorkflowRunState::Running).outcomes(0, 0, 0, 2)
        })
        .collect();
    let many = WorkflowProgress::new(runs);
    assert_eq!(many.desired_rows(), 7);
    let buf = render(100, 7, |area, buf| many.paint(area, buf, &theme));
    assert_eq!(text(&buf).lines().last(), Some(" +6 more · ↓ to manage"));
    let buf = render(100, 1, |area, buf| many.paint(area, buf, &theme));
    assert_eq!(text(&buf), " +12 more · ↓ to manage");
}

#[test]
fn composer_density_enclosure_and_send_geometry_match_the_mounted_policy() {
    for (density, quiet, enclosed, cap) in [
        (NativeComposerDensity::Compact, 2, 3, 7),
        (NativeComposerDensity::Comfortable, 3, 4, 9),
        (NativeComposerDensity::Spacious, 4, 5, 12),
    ] {
        let composer = NativeComposer::new("").density(density);
        assert_eq!(
            composer.clone().enclosed(false).desired_height(80, 20),
            quiet
        );
        assert_eq!(composer.desired_height(80, 20), enclosed);
        assert_eq!(
            NativeComposer::new("x\n".repeat(40))
                .density(density)
                .desired_height(80, 60),
            cap
        );
        assert_eq!(composer.desired_height(80, 1), 1);
    }
    let composer = NativeComposer::new("ship it")
        .focused(true)
        .can_submit(true);
    for width in 1..12 {
        let area = Rect::new(10, 20, width, 4);
        assert!(!composer.has_panel(area));
        assert!(composer.geometry(area).submit.is_none());
    }
    let area = Rect::new(10, 20, 40, 4);
    let geometry = composer.geometry(area);
    assert_eq!(geometry.submit, Some(Rect::new(45, 22, 3, 1)));
    assert_eq!(geometry.inner, Rect::new(11, 21, 33, 2));
    assert_eq!(geometry.text, Rect::new(13, 21, 31, 2));
    assert!(geometry.text.right() < geometry.submit.unwrap().x);
    let mut buf = Buffer::empty(area);
    composer.paint(area, &mut buf, &Profile::DarkTrue.theme());
    assert_eq!(buf[(10, 20)].symbol(), "╭");
    assert_eq!(buf[(49, 23)].symbol(), "╯");
    assert_eq!(buf[(45, 22)].symbol(), "[");
    assert_eq!(buf[(46, 22)].symbol(), "↵");
    assert_eq!(buf[(47, 22)].symbol(), "]");
}

#[test]
fn composer_uses_the_native_background_info_soft_and_agent_target_slots() {
    let source: serde_json::Value =
        serde_json::from_str(include_str!("../assets/tui-palettes.json")).unwrap();
    let source = source.as_array().unwrap();
    let slot = |record: &serde_json::Value, key: &str| {
        let value = &record["slots"][key];
        if let Some(rgb) = value.as_array() {
            Color::Rgb(
                rgb[0].as_u64().unwrap().try_into().unwrap(),
                rgb[1].as_u64().unwrap().try_into().unwrap(),
                rgb[2].as_u64().unwrap().try_into().unwrap(),
            )
        } else {
            match value.as_str().unwrap() {
                "Color::Reset" => Color::Reset,
                "Color::Cyan" => Color::Cyan,
                "Color::Yellow" => Color::Yellow,
                color => panic!("unexpected native slot {key}: {color}"),
            }
        }
    };
    for palette in TuiPalette::ALL {
        let record = source
            .iter()
            .find(|record| record["name"] == palette.name())
            .unwrap();
        let theme = Profile::DarkTrue.theme().tui_palette(palette);
        let composer = NativeComposer::new("")
            .focused(true)
            .placeholder("Write a task")
            .can_submit(true)
            .target("Builder");
        let area = Rect::new(0, 0, 40, 4);
        let geometry = composer.geometry(area);
        let buf = render(area.width, area.height, |area, buf| {
            composer.paint(area, buf, &theme)
        });
        let expected_background = slot(record, "composer_bg");
        assert!(
            buf.content
                .iter()
                .all(|cell| cell.bg == expected_background),
            "{} composer ground must include its complete chrome",
            palette.name()
        );
        let submit = geometry.submit.unwrap();
        assert_eq!(
            buf[(submit.x + 1, submit.y)].fg,
            slot(record, "info"),
            "{} ready submit ink",
            palette.name()
        );
        assert_eq!(
            buf[(geometry.text.x, geometry.text.y)].fg,
            slot(record, "text_soft"),
            "{} idle prompt ink",
            palette.name()
        );
        let target = buf
            .content
            .iter()
            .find(|cell| cell.symbol() == "B")
            .unwrap();
        assert_eq!(
            target.fg,
            slot(record, "accent_action"),
            "{} agent target",
            palette.name()
        );
        if palette == TuiPalette::ShorelineLight {
            assert_ne!(expected_background, slot(record, "panel_bg"));
            assert_ne!(expected_background, slot(record, "elevated_bg"));
        }
    }
}

#[test]
fn caller_unicode_lines_and_caret_survive_while_every_field_loses_controls() {
    let theme = Profile::DarkTrue.theme();
    let composer = NativeComposer::new("cafe\u{301} 鲸鱼\u{202e}\n\nlast\u{1b}")
        .focused(true)
        .can_submit(true)
        .cursor(0)
        .submit_hint("Enter\u{202e} send")
        .target("reviewer\u{1b}");
    let area = Rect::new(0, 0, 60, 7);
    let buf = render(area.width, area.height, |area, buf| {
        composer.paint(area, buf, &theme)
    });
    let shown = text(&buf);
    assert!(shown.contains("cafe\u{301} 鲸鱼"), "{shown}");
    assert!(shown.contains("last"));
    assert!(!shown.contains(['\u{202e}', '\u{1b}']));
    let caret = composer.cursor_position(area).unwrap();
    assert_eq!(caret.x, composer.geometry(area).text.x);
    assert_eq!(
        buf[(composer.geometry(area).prompt_x.unwrap(), caret.y)].symbol(),
        "❯"
    );
    assert!(
        shown
            .lines()
            .any(|row| row.trim_matches(['│', ' ']).is_empty()),
        "logical blank row retained: {shown}"
    );
    let progress = WorkflowProgress::new(vec![
        WorkflowRun::new("A\u{202e}\nB", WorkflowRunState::Degraded)
            .outcomes(1, 1, 0, 2)
            .reason("reason\u{1b}\u{2066} text"),
    ]);
    let buf = render(140, 1, |area, buf| progress.paint(area, buf, &theme));
    assert!(!text(&buf).contains(['\u{202e}', '\u{1b}', '\u{2066}']));
}

#[test]
fn native_chrome_is_bounded_in_tiny_offset_and_maximum_origin_buffers() {
    for profile in Profile::ALL {
        let theme = profile.theme();
        for width in [0, 1, 4, 12, 40] {
            for height in [0, 1, 2, 4] {
                let area = Rect::new(4, 3, width, height);
                let surfaces: Vec<Box<dyn Paint>> = vec![
                    Box::new(
                        NativeComposer::new("鲸鱼 cafe\u{301}\nlast")
                            .focused(true)
                            .can_submit(true)
                            .target("worker")
                            .submit_hint("Enter send"),
                    ),
                    Box::new(WorkflowProgress::new(vec![
                        WorkflowRun::new("鲸鱼 cafe\u{301}", WorkflowRunState::Failed)
                            .outcomes(usize::MAX, usize::MAX, usize::MAX, usize::MAX)
                            .reason("retry later"),
                    ])),
                ];
                for surface in surfaces {
                    let mut buf = Buffer::empty(Rect::new(2, 2, 50, 8));
                    for cell in &mut buf.content {
                        cell.set_symbol("z");
                    }
                    let before = buf.clone();
                    surface.paint(area, &mut buf, &theme);
                    let clipped = area.intersection(buf.area);
                    for y in buf.area.top()..buf.area.bottom() {
                        for x in buf.area.left()..buf.area.right() {
                            if !clipped.contains((x, y).into()) {
                                assert_eq!(buf[(x, y)], before[(x, y)]);
                            }
                        }
                    }
                    let mut edge = Buffer::empty(Rect::new(u16::MAX - 50, u16::MAX - 4, 50, 4));
                    surface.paint(edge.area, &mut edge, &theme);
                }
            }
        }
        let ascii = NativeComposer::new("send this")
            .focused(true)
            .can_submit(true);
        let buf = render(40, 4, |area, buf| ascii.paint(area, buf, &theme));
        if profile == Profile::Ascii {
            assert!(buf.content.iter().all(|cell| cell.symbol().is_ascii()));
        }
    }
}
