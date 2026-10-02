use std::time::Duration;

use codewhale_ratatui::{
    Caps, MotionMode, Paint, Role, Theme,
    color::{contrast_ratio, relative_luminance},
    gallery,
    ocean::{OceanColumn, OceanPhase, OceanRamp},
    testing::{Profile, assert_frames_keep_the_rules, frames_for},
};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
};

const PHASES: [OceanPhase; 8] = [
    OceanPhase::Idle,
    OceanPhase::Typing,
    OceanPhase::Working,
    OceanPhase::Verifying,
    OceanPhase::Waiting,
    OceanPhase::Approval,
    OceanPhase::Done,
    OceanPhase::Failed,
];

fn ordinary(area: Rect, theme: &Theme) -> Buffer {
    let mut buf = Buffer::empty(area);
    buf.set_style(area, theme.bg(Role::Background));
    buf
}

#[test]
fn authored_stops_and_depth_match_the_native_terminal() {
    let ramp = OceanRamp::for_theme(&Profile::DarkTrue.theme()).unwrap();
    assert_eq!(OceanRamp::SURFACE, Color::Rgb(0x10, 0x2a, 0x45));
    assert_eq!(OceanRamp::MIDDLE, Color::Rgb(0x0a, 0x1e, 0x33));
    assert_eq!(OceanRamp::DEEP, Color::Rgb(0x06, 0x13, 0x20));
    assert_eq!(ramp.color_at_context(0, 5, 0), OceanRamp::SURFACE);
    // The native middle stop is a control point, not a flat middle band.
    assert_eq!(ramp.color_at_context(2, 5, 0), Color::Rgb(11, 31, 51));
    assert_eq!(ramp.color_at_context(4, 5, 0), OceanRamp::DEEP);
    assert_eq!(ramp.color_at_context(u16::MAX, 5, 0), OceanRamp::DEEP);
    assert_eq!(ramp.color_at_context(0, 5, 50), Color::Rgb(11, 31, 51));
    assert_eq!(ramp.color_at_context(0, 5, 100), OceanRamp::DEEP);
    assert_eq!(ramp.color_at_context(0, 5, 255), OceanRamp::DEEP);
    assert_eq!(ramp.color_at_context(0, 0, 0), OceanRamp::SURFACE);
    assert_eq!(ramp.color_at_context(0, 1, 50), Color::Rgb(11, 31, 51));
    let mut previous = ramp.color_at_context(0, 80, 0);
    for row in 1..80 {
        let next = ramp.color_at_context(row, 80, 0);
        assert!(relative_luminance(next).unwrap() <= relative_luminance(previous).unwrap());
        previous = next;
    }
}

#[test]
fn the_native_breath_is_deterministic_and_has_a_ninety_second_period() {
    let ramp = OceanRamp::for_theme(&Profile::DarkTrue.theme()).unwrap();
    let at = |row, phase, millis| {
        ramp.color_at_phase_context(row, 5, Duration::from_millis(millis), phase, 0)
    };
    assert_eq!(at(0, OceanPhase::Working, 22_500), Color::Rgb(16, 42, 70));
    assert_eq!(at(4, OceanPhase::Working, 22_500), Color::Rgb(7, 21, 35));
    assert_eq!(at(0, OceanPhase::Verifying, 22_500), Color::Rgb(17, 44, 71));
    assert_eq!(at(0, OceanPhase::Working, 67_500), OceanRamp::SURFACE);
    assert_eq!(at(4, OceanPhase::Working, 67_500), OceanRamp::DEEP);
    for phase in PHASES {
        assert_eq!(at(2, phase, 22_500), at(2, phase, 112_500));
    }
}

#[test]
fn attention_and_failure_are_steady_even_with_a_stale_completion_clock() {
    let theme = Profile::DarkTrue.theme();
    let viewport = Rect::new(3, 5, 40, 12);
    for phase in [
        OceanPhase::Waiting,
        OceanPhase::Approval,
        OceanPhase::Failed,
    ] {
        let expected = OceanColumn::new(Duration::ZERO, MotionMode::Still)
            .phase(phase)
            .color_at_y(5, viewport, &theme);
        assert_ne!(expected, Some(OceanRamp::SURFACE));
        for motion in [MotionMode::Full, MotionMode::Reduced, MotionMode::Still] {
            for elapsed in [
                Duration::ZERO,
                Duration::from_secs(22),
                Duration::from_secs(67),
            ] {
                assert_eq!(
                    OceanColumn::new(elapsed, motion)
                        .phase(phase)
                        .presence(0)
                        .completion_elapsed(Duration::from_millis(320))
                        .color_at_y(5, viewport, &theme),
                    expected,
                );
            }
        }
    }
}

#[test]
fn completion_has_the_native_boundary_and_obeys_motion_policy() {
    let theme = Profile::DarkTrue.theme();
    let viewport = Rect::new(0, 0, 40, 5);
    let color = |millis, motion| {
        OceanColumn::new(Duration::from_secs(22), motion)
            .phase(OceanPhase::Done)
            .presence(0)
            .completion_elapsed(Duration::from_millis(millis))
            .color_at_y(0, viewport, &theme)
            .unwrap()
    };
    assert_eq!(OceanRamp::COMPLETION_BREATH, Duration::from_millis(800));
    assert_eq!(color(0, MotionMode::Full), Color::Rgb(14, 37, 61));
    assert_eq!(color(320, MotionMode::Full), Color::Rgb(18, 47, 77));
    assert_eq!(color(800, MotionMode::Full), OceanRamp::SURFACE);
    assert_eq!(color(8_000, MotionMode::Full), OceanRamp::SURFACE);
    for motion in [MotionMode::Reduced, MotionMode::Still] {
        for millis in [0, 320, 799, 800, 8_000] {
            assert_eq!(color(millis, motion), OceanRamp::SURFACE);
        }
    }
}

#[test]
fn reduced_and_still_ignore_elapsed_for_every_phase() {
    let theme = Profile::DarkTrue.theme();
    let viewport = Rect::new(0, 0, 80, 24);
    for motion in [MotionMode::Reduced, MotionMode::Still] {
        for phase in PHASES {
            for row in 0..24 {
                let at = |elapsed| {
                    OceanColumn::new(elapsed, motion)
                        .phase(phase)
                        .completion_elapsed(elapsed)
                        .color_at_y(row, viewport, &theme)
                };
                assert_eq!(at(Duration::ZERO), at(Duration::from_secs(22)));
                assert_eq!(at(Duration::ZERO), at(Duration::from_secs(67)));
            }
        }
    }
}

#[test]
fn semantic_surfaces_and_all_ink_and_symbols_remain_owned_by_the_host() {
    let theme = Profile::DarkTrue.theme();
    let mut buf = ordinary(Rect::new(2, 4, 10, 3), &theme);
    for (index, role) in [
        Role::Sidebar,
        Role::Background,
        Role::Surface,
        Role::Hover,
        Role::Selected,
        Role::DiffAddedTint,
        Role::DiffRemovedTint,
        Role::Primary,
    ]
    .into_iter()
    .enumerate()
    {
        buf[(2 + index as u16, 4)]
            .set_symbol("海")
            .set_style(theme.fg(Role::Foreground).patch(theme.bg(role)))
            .set_style(Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED));
    }
    buf[(10, 4)]
        .set_bg(Color::Rgb(8, 9, 10))
        .set_symbol("e\u{301}");
    buf[(11, 4)].set_bg(Color::Reset).set_symbol("host");
    let before = buf.clone();
    OceanColumn::new(Duration::ZERO, MotionMode::Still).apply(buf.area, &mut buf, &theme);
    assert_ne!(
        buf[(2, 4)].bg,
        before[(2, 4)].bg,
        "ordinary Sidebar changes"
    );
    assert_ne!(
        buf[(3, 4)].bg,
        before[(3, 4)].bg,
        "ordinary Background changes"
    );
    for index in 2..10 {
        assert_eq!(
            buf[(2 + index, 4)],
            before[(2 + index, 4)],
            "protected fill at {index}"
        );
    }
    for (old, new) in before.content.iter().zip(&buf.content) {
        let mut restored = new.clone();
        restored.set_bg(old.bg);
        assert_eq!(&restored, old, "only the background may change");
    }
}

#[test]
fn unknown_unreadable_or_reversed_ink_keeps_its_original_ground() {
    let theme = Profile::DarkTrue.theme();
    let mut buf = ordinary(Rect::new(0, 0, 6, 1), &theme);
    buf[(0, 0)].set_symbol("?").set_fg(Color::Reset);
    buf[(1, 0)].set_symbol("?").set_fg(Color::Yellow);
    buf[(2, 0)].set_symbol("?").set_fg(Color::Rgb(16, 42, 69));
    buf[(3, 0)]
        .set_symbol("?")
        .set_fg(theme.color(Role::Foreground).unwrap())
        .set_style(Style::default().add_modifier(Modifier::REVERSED));
    buf[(4, 0)]
        .set_symbol("?")
        .set_fg(Color::Rgb(255, 255, 255));
    let before = buf.clone();
    OceanColumn::new(Duration::ZERO, MotionMode::Still).apply(buf.area, &mut buf, &theme);
    for x in 0..4 {
        assert_eq!(buf[(x, 0)], before[(x, 0)]);
    }
    assert_eq!(
        buf[(4, 0)].bg,
        OceanRamp::SURFACE,
        "known high-contrast custom ink is safe"
    );
    assert_eq!(
        buf[(5, 0)].bg,
        OceanRamp::SURFACE,
        "blank ordinary water needs no ink proof"
    );
}

#[test]
fn all_authored_treatments_keep_audited_ink_contrast_at_every_depth() {
    let theme = Profile::DarkTrue.theme();
    let ramp = OceanRamp::for_theme(&theme).unwrap();
    let inks = [
        (Role::Foreground, 4.5),
        (Role::Muted, 4.5),
        (Role::Primary, 4.5),
        (Role::Live, 4.5),
        (Role::Attention, 4.5),
        (Role::Danger, 4.5),
        (Role::Hint, 4.5),
        (Role::Dim, 3.0),
        (Role::BorderStrong, 3.0),
    ];
    for context in [0, 25, 50, 75, 100] {
        for row in 0..80 {
            for phase in PHASES {
                for millis in [0, 22_500, 45_000, 67_500] {
                    let bg = ramp.color_at_phase_context(
                        row,
                        80,
                        Duration::from_millis(millis),
                        phase,
                        context,
                    );
                    for (role, floor) in inks {
                        assert!(
                            contrast_ratio(theme.color(role).unwrap(), bg).unwrap() >= floor,
                            "{role:?} on {phase:?}, row {row}, context {context}, time {millis}: {bg:?}"
                        );
                    }
                }
            }
            for millis in [0, 160, 320, 480, 799, 800] {
                let bg = ramp.color_at_completion_context(
                    row,
                    80,
                    Duration::from_millis(millis),
                    context,
                );
                for (role, floor) in inks {
                    assert!(
                        contrast_ratio(theme.color(role).unwrap(), bg).unwrap() >= floor,
                        "{role:?} on completion, row {row}, context {context}, time {millis}: {bg:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn shared_viewport_is_continuous_across_bands_and_buffer_clipping() {
    let theme = Profile::DarkTrue.theme();
    let viewport = Rect::new(5, 8, 40, 12);
    let column = OceanColumn::new(Duration::ZERO, MotionMode::Still).viewport(viewport);
    let mut full = ordinary(viewport, &theme);
    column.apply(viewport, &mut full, &theme);
    let mut bands = ordinary(viewport, &theme);
    column.apply(Rect::new(5, 8, 40, 4), &mut bands, &theme);
    column.apply(Rect::new(5, 12, 40, 8), &mut bands, &theme);
    assert_eq!(bands, full);
    let visible = Rect::new(9, 11, 8, 4);
    let mut clipped = ordinary(visible, &theme);
    column.apply(viewport, &mut clipped, &theme);
    for y in visible.top()..visible.bottom() {
        for x in visible.left()..visible.right() {
            assert_eq!(clipped[(x, y)], full[(x, y)]);
        }
    }
}

#[test]
fn tiny_empty_off_buffer_and_maximum_coordinate_areas_are_safe() {
    let theme = Profile::DarkTrue.theme();
    let column = OceanColumn::new(Duration::ZERO, MotionMode::Still);
    for area in [
        Rect::new(0, 0, 0, 0),
        Rect::new(7, 9, 1, 1),
        Rect::new(u16::MAX - 4, u16::MAX - 3, 4, 3),
    ] {
        let mut buf = ordinary(area, &theme);
        column.paint(area, &mut buf, &theme);
        if !area.is_empty() {
            assert_eq!(buf[(area.x, area.y)].bg, OceanRamp::SURFACE);
        }
    }
    let area = Rect::new(10, 12, 4, 3);
    let mut buf = ordinary(area, &theme);
    let before = buf.clone();
    column.apply(Rect::new(0, 0, 1, 1), &mut buf, &theme);
    assert_eq!(buf, before);
    column.apply(Rect::new(11, 13, 2, 1), &mut buf, &theme);
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            if y != 13 || !(11..13).contains(&x) {
                assert_eq!(
                    buf[(x, y)],
                    before[(x, y)],
                    "paint stayed inside its request"
                );
            }
        }
    }
}

#[test]
fn every_fallback_profile_and_terminal_owned_ground_is_unchanged() {
    for profile in Profile::ALL {
        let theme = profile.theme();
        let area = Rect::new(0, 0, 40, 12);
        let mut buf = ordinary(area, &theme);
        let before = buf.clone();
        OceanColumn::new(Duration::from_millis(22_500), MotionMode::Full)
            .phase(OceanPhase::Approval)
            .apply(area, &mut buf, &theme);
        if profile == Profile::DarkTrue {
            assert_ne!(buf, before);
        } else {
            assert_eq!(buf, before, "{} is an exact fallback", profile.name());
            assert!(OceanRamp::for_theme(&theme).is_none());
        }
    }
    let theme = Profile::DarkTrue.theme().without_base_ground();
    let mut buf = ordinary(Rect::new(0, 0, 40, 12), &theme);
    buf.set_style(buf.area, theme.bg(Role::Sidebar));
    let before = buf.clone();
    OceanColumn::new(Duration::ZERO, MotionMode::Full).apply(buf.area, &mut buf, &theme);
    assert_eq!(buf, before, "the host kept its own base ground");
}

#[test]
fn ascii_is_chrome_policy_and_cannot_rewrite_host_unicode() {
    let normal = Profile::DarkTrue.theme();
    let ascii = Theme::new(Caps {
        ascii: true,
        ..normal.caps()
    });
    let mut buf = ordinary(Rect::new(0, 0, 10, 3), &normal);
    buf.set_string(0, 0, "海 e\u{301}", normal.fg(Role::Foreground));
    let before = buf.clone();
    let column = OceanColumn::new(Duration::ZERO, MotionMode::Still);
    let mut expected = buf.clone();
    column.apply(buf.area, &mut expected, &normal);
    column.apply(buf.area, &mut buf, &ascii);
    assert_eq!(buf, expected);
    for (old, new) in before.content.iter().zip(&buf.content) {
        assert_eq!(old.symbol(), new.symbol());
        assert_eq!(old.fg, new.fg);
    }
}

#[test]
fn native_gallery_scenes_keep_the_nine_profile_rules_at_narrow_widths() {
    let entries: Vec<_> = gallery::entries()
        .into_iter()
        .filter(|entry| entry.name.starts_with("ocean-"))
        .collect();
    assert_eq!(entries.len(), 4);
    for entry in entries {
        assert_frames_keep_the_rules(&frames_for(
            entry.name,
            &Profile::ALL,
            &[40, 80],
            entry.height,
            entry.draw,
        ));
    }
}
