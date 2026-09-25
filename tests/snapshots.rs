//! Every gallery component in every terminal profile, snapshotted with
//! insta, plus the rules every frame must keep.
//!
//! Snapshots record glyphs and the role each run was painted with, not hex
//! values, so a token change does not rewrite them but painting the wrong
//! role does. Review changes with `cargo insta review`.

use codewhale_ratatui::{
    Paint, State, Toast, Toasts, gallery,
    testing::{self, Profile},
};
use ratatui::style::Color;

fn dump(entry: &gallery::Entry) -> String {
    let mut out = String::new();
    for profile in Profile::ALL {
        let theme = profile.theme();
        let buf = gallery::render(entry, &theme);
        out.push_str(&format!("== {}\n", profile.name()));
        out.push_str(&testing::styled(&buf, &theme));
        out.push('\n');
    }
    out
}

#[test]
fn gallery_snapshots() {
    for entry in gallery::entries() {
        insta::assert_snapshot!(entry.name, dump(&entry));
    }
}

/// What each profile may put on the screen.
fn color_allowed(profile: Profile, color: Color) -> bool {
    match (profile, color) {
        (_, Color::Reset) => true,
        (Profile::NoColor | Profile::Ascii, _) => false,
        (Profile::Ansi16 | Profile::UnknownGround, Color::Rgb(..) | Color::Indexed(_)) => false,
        (Profile::Dark256 | Profile::Light256, Color::Rgb(..)) => false,
        (Profile::Dark256 | Profile::Light256, Color::Indexed(i)) => i >= 16,
        _ => true,
    }
}

#[test]
fn every_frame_keeps_the_rules() {
    let mut broken = Vec::new();
    for profile in Profile::ALL {
        let theme = profile.theme();
        for entry in gallery::entries() {
            let buf = gallery::render(&entry, &theme);
            let text = testing::text(&buf);
            let at = format!("{} · {}", entry.name, profile.name());
            if text.contains("...") {
                broken.push(format!("{at}: `...` where the one ellipsis is `…`"));
            }
            if profile == Profile::Ascii && !text.is_ascii() {
                broken.push(format!("{at}: non-ASCII glyph in ASCII-safe output"));
            }
            for banned in ['═', '≈', '∿'] {
                if text.contains(banned) {
                    broken.push(format!("{at}: `{banned}` rule"));
                }
            }
            let rules = text
                .lines()
                // A horizon runs from the left edge; a panel's edge starts
                // with a corner.
                .filter(|l| {
                    l.starts_with('─')
                        && l.chars().filter(|c| *c == '─').count() * 2 > usize::from(entry.width)
                })
                .count();
            if rules > 1 {
                broken.push(format!("{at}: {rules} horizons; one per frame"));
            }
            for cell in buf.content() {
                for color in [cell.fg, cell.bg] {
                    if !color_allowed(profile, color) {
                        broken.push(format!("{at}: {color:?} is not allowed here"));
                        break;
                    }
                }
            }
            if !theme.paints_grounds() && buf.content().iter().any(|c| c.bg != Color::Reset) {
                broken.push(format!("{at}: painted a ground the terminal cannot show"));
            }
        }
    }
    broken.dedup();
    assert!(broken.is_empty(), "{}", broken.join("\n"));
}

#[test]
fn caller_text_cannot_reorder_itself() {
    let theme = Profile::DarkTrue.theme();
    let toasts = Toasts::new(vec![Toast::new(
        State::NeedsYou,
        "Approve rm -rf ~/\u{202E}txt.exe",
    )]);
    let buf = testing::render(40, 1, |area, buf| toasts.paint(area, buf, &theme));
    let text = testing::text(&buf);
    assert!(text.contains("rm -rf ~/txt.exe"), "{text}");
    assert!(!text.contains('\u{202E}'));
}
