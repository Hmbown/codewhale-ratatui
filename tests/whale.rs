//! The Braille whale matches the v2 kit's own stills, dot for dot, for all
//! 17 actions at both sizes the widget draws.
//!
//! `tests/whale-stills/*.txt` are copied unchanged from codewhale-app
//! `vendor/whale-character-v2/braille/` (branch `beta/batch-20260925`,
//! `d486398`), which `braille-demo.cjs` renders from the same Director poses
//! that `tools/export-whale.cjs` exports into `assets/whale-v2.scenes`.

use codewhale_ratatui::whale::{self, WhaleState};

fn still(name: &str) -> String {
    let path = format!(
        "{}/tests/whale-stills/{name}.txt",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// The still's art rows; the kit writes its label on the row after.
fn art(still: &str, rows: usize) -> Vec<String> {
    still
        .lines()
        .take(rows)
        .map(|l| l.trim_end().to_string())
        .collect()
}

fn rendered(state: WhaleState, cols: u16, rows: u16) -> Vec<String> {
    whale::frame(state, cols, rows)
        .expect("viewport fits")
        .text()
        .lines()
        .map(|l| l.trim_end().to_string())
        .collect()
}

/// Every action in the kit, by its still's name.
fn states() -> impl Iterator<Item = (&'static str, WhaleState)> {
    WhaleState::ALL.into_iter().map(|s| (s.key(), s))
}

#[test]
fn the_kit_has_seventeen_actions_and_we_draw_them_all() {
    assert_eq!(WhaleState::ALL.len(), 17);
    let dir = format!("{}/tests/whale-stills", env!("CARGO_MANIFEST_DIR"));
    let mut keys: Vec<String> = std::fs::read_dir(&dir)
        .expect("stills")
        .filter_map(|e| {
            let name = e.ok()?.file_name().into_string().ok()?;
            Some(name.strip_suffix("-32x16.txt")?.to_string())
        })
        .collect();
    keys.sort();
    let mut ours: Vec<&str> = states().map(|(k, _)| k).collect();
    ours.sort_unstable();
    assert_eq!(keys, ours, "one WhaleState per kit still");
    for (key, state) in states() {
        assert_eq!(WhaleState::from_key(key), Some(state));
        assert!(!state.words().is_empty());
    }
}

#[test]
fn every_state_matches_the_kit_at_32x16() {
    for (name, state) in states() {
        let expected = art(&still(&format!("{name}-32x16")), 16);
        assert_eq!(rendered(state, 32, 16), expected, "{name} 32x16");
    }
}

#[test]
fn every_state_matches_the_kit_at_20x10() {
    for (name, state) in states() {
        let expected = art(&still(&format!("{name}-20x10")), 10);
        assert_eq!(rendered(state, 20, 10), expected, "{name} 20x10");
    }
}

#[test]
fn the_pod_draws_exactly_the_calves_it_is_given() {
    let dots = |calves| {
        whale::frame(WhaleState::Pod { calves }, 32, 16)
            .expect("fits")
            .cells
            .iter()
            .map(|c| c.count_ones())
            .sum::<u32>()
    };
    let (one, two, three) = (dots(1), dots(2), dots(3));
    assert!(
        one < two && two < three,
        "calves add dots: {one} {two} {three}"
    );
    // The art has room for three calves; more agents draw three. A pod of
    // none draws plain work: no calf is invented.
    assert_eq!(dots(9), three);
    let busy = whale::frame(WhaleState::Busy, 32, 16).expect("fits");
    assert_eq!(
        whale::frame(WhaleState::Pod { calves: 0 }, 32, 16).expect("fits"),
        busy
    );
    // The words always carry the real count, whatever the art can show.
    for (calves, words) in [
        (0, "Working"),
        (1, "Working with 1 agent"),
        (2, "Working with 2 agents"),
        (3, "Working with 3 agents"),
        (9, "Working with 9 agents"),
        (255, "Working with 255 agents"),
    ] {
        assert_eq!(WhaleState::Pod { calves }.words(), words);
    }
}

#[test]
fn frame_handles_any_viewport_without_overflow() {
    assert!(whale::frame(WhaleState::Rest, 15, 8).is_none());
    assert!(whale::frame(WhaleState::Rest, u16::MAX, 8).is_some());
}

#[test]
fn every_exported_scene_parses_and_is_distinct() {
    let scenes = whale::scenes();
    assert_eq!(
        scenes.len(),
        38,
        "(16 actions + pods of 1, 2 and 3) x 2 optical sizes"
    );
    let mut seen: Vec<(&str, Vec<String>)> = Vec::new();
    for (name, state) in states() {
        let art = rendered(state, 32, 16);
        if let Some((twin, _)) = seen.iter().find(|(_, a)| *a == art) {
            panic!("{name} draws exactly like {twin}");
        }
        seen.push((name, art));
    }
    assert!(
        whale::frame(WhaleState::Rest, 15, 8).is_none(),
        "below 16x8 only words fit"
    );
}
