//! Generate `src/roles.rs` from the vendored design tokens, and check it.
//!
//!   cargo test --test generated                       # fails if roles.rs is stale
//!   CODEWHALE_BLESS=1 cargo test --test generated     # rewrites roles.rs
//!
//! Colors come only from `vendor/codewhale-design/tokens.json`, the one
//! token source the desktop app and the web read too. The generator adds
//! the one thing a terminal needs that the tokens cannot say: which xterm
//! 256-color index to show for each role. It starts from the nearest fixed
//! index and, where quantizing breaks the design's own contrast rules (the
//! same pairs `generate.py` enforces in truecolor), moves the ink to the
//! nearest index that holds them. Grounds are never moved.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

use codewhale_ratatui::color::{contrast_ratio, indexed_rgb, rgb, rgb_to_ansi256};
use ratatui::style::Color;

/// Every role, in enum order: variant, `tokens.json` key, doc comment.
const ROLES: &[(&str, &str, &str)] = &[
    ("Sidebar", "sidebar", "The deepest ground: rails and things put away."),
    ("Background", "background", "The stage."),
    ("Surface", "surface", "A raised panel."),
    ("Hover", "hover", "A row under the pointer."),
    ("Selected", "selected", "The selected row."),
    ("Foreground", "foreground", "Body ink."),
    ("Muted", "muted_foreground", "Ink that recedes: details, verbs in hints, asides."),
    ("Border", "border", "Quiet lines that separate."),
    ("BorderStrong", "border_strong", "Edges that identify a control or a decision."),
    ("Primary", "primary", "Actions, focus and links."),
    ("PrimaryForeground", "primary_foreground", "Ink on a `Primary` ground."),
    ("Live", "live", "Work happening now, and work done."),
    ("Attention", "attention", "Needs you."),
    ("Danger", "danger", "Failed or destructive."),
];

/// The design's contrast rules (`generate.py`): text at 4.5:1 and control
/// edges at 3:1 on every ground, and ink on a primary fill at 4.5:1.
const GROUNDS: &[&str] = &["background", "surface", "sidebar", "hover", "selected"];
const TEXT: &[&str] = &[
    "foreground",
    "muted_foreground",
    "primary",
    "live",
    "attention",
    "danger",
];
const EDGES: &[&str] = &["border_strong"];

fn hex(s: &str) -> u32 {
    u32::from_str_radix(s, 16).unwrap_or_else(|_| panic!("bad hex {s}"))
}

fn idx_color(i: u8) -> Color {
    Color::Indexed(i)
}

/// Contrast floors the ink `key` must hold, as `(ground key, floor)`.
fn floors(key: &str) -> Vec<(&'static str, f32)> {
    if TEXT.contains(&key) {
        GROUNDS.iter().map(|g| (*g, 4.5)).collect()
    } else if EDGES.contains(&key) {
        GROUNDS.iter().map(|g| (*g, 3.0)).collect()
    } else if key == "primary_foreground" {
        vec![("primary", 4.5)]
    } else {
        Vec::new()
    }
}

fn nearest(value: u32) -> u8 {
    rgb_to_ansi256((value >> 16) as u8, (value >> 8) as u8, value as u8)
}

fn dist(a: (u8, u8, u8), value: u32) -> u32 {
    let d = |x: u8, y: u8| {
        let v = i32::from(x) - i32::from(y);
        (v * v) as u32
    };
    d(a.0, (value >> 16) as u8) + d(a.1, (value >> 8) as u8) + d(a.2, value as u8)
}

/// State hues keep their hue at 256 colors: a green that quantizes to gray
/// stops saying "live".
const HUES: &[&str] = &["primary", "live", "attention", "danger"];

/// Hue in degrees and chroma (max - min channel) of an RGB triple.
fn hue_chroma((r, g, b): (u8, u8, u8)) -> (f32, u8) {
    let (rf, gf, bf) = (f32::from(r), f32::from(g), f32::from(b));
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let c = f32::from(max - min);
    if c == 0.0 {
        return (0.0, 0);
    }
    let h = if max == r {
        ((gf - bf) / c).rem_euclid(6.0)
    } else if max == g {
        (bf - rf) / c + 2.0
    } else {
        (rf - gf) / c + 4.0
    };
    (h * 60.0, max - min)
}

fn split(value: u32) -> (u8, u8, u8) {
    ((value >> 16) as u8, (value >> 8) as u8, value as u8)
}

/// Whether index `i` still reads as the hue of `value`.
fn keeps_hue(i: u8, value: u32) -> bool {
    let (want, _) = hue_chroma(split(value));
    let (got, chroma) = hue_chroma(indexed_rgb(i).expect("fixed index"));
    let diff = (want - got).abs();
    chroma >= 40 && diff.min(360.0 - diff) <= 35.0
}

/// The 256-color table for one appearance, and a note per adjusted role.
fn table_256(colors: &BTreeMap<String, u32>, mode: &str) -> (BTreeMap<String, u8>, Vec<String>) {
    let mut table: BTreeMap<String, u8> =
        colors.iter().map(|(k, v)| (k.clone(), nearest(*v))).collect();
    let mut notes = Vec::new();
    // Inks over grounds first; ink on primary last, so it sees primary's
    // final index.
    let mut order: Vec<&str> = TEXT.iter().chain(EDGES).copied().collect();
    order.push("primary_foreground");
    for key in order {
        let rules = floors(key);
        let holds = |i: u8, table: &BTreeMap<String, u8>| {
            rules.iter().all(|(g, floor)| {
                contrast_ratio(idx_color(i), idx_color(table[*g])).is_some_and(|r| r >= *floor)
            })
        };
        let hue = HUES.contains(&key);
        let start = table[key];
        if holds(start, &table) && (!hue || keeps_hue(start, colors[key])) {
            continue;
        }
        let value = colors[key];
        // Never land on an index another ink already shows: two states
        // sharing one color would read as one state.
        let taken: Vec<u8> = TEXT
            .iter()
            .chain(EDGES)
            .filter(|k| **k != key)
            .map(|k| table[*k])
            .collect();
        let best = (16..=255u8)
            .filter(|i| !taken.contains(i) && holds(*i, &table))
            .filter(|i| !hue || keeps_hue(*i, value))
            .min_by_key(|i| (dist(indexed_rgb(*i).expect("fixed index"), value), *i))
            .unwrap_or_else(|| panic!("{mode} {key}: no 256-color index holds its contrast"));
        let why = if holds(start, &table) { "lost its hue" } else { "failed its contrast floor" };
        notes.push(format!("{mode} {key}: {start} -> {best} (nearest {why})"));
        table.insert(key.to_string(), best);
    }
    let hues: Vec<u8> = ["primary", "live", "attention", "danger"]
        .iter()
        .map(|k| table[*k])
        .collect();
    for (i, a) in hues.iter().enumerate() {
        assert!(!hues[i + 1..].contains(a), "{mode}: two state hues share 256-color index {a}");
    }
    (table, notes)
}

fn render(json: &serde_json::Value) -> String {
    let version = json["version"].as_str().expect("version");
    let mut modes = BTreeMap::new();
    for mode in ["dark", "light"] {
        let obj = json["colors"][mode].as_object().expect("colors");
        let keys: Vec<&str> = obj.keys().map(String::as_str).collect();
        for key in &keys {
            assert!(
                ROLES.iter().any(|(_, k, _)| k == key),
                "tokens.json color `{key}` has no Role; add it to ROLES in tests/generated.rs"
            );
        }
        for (_, key, _) in ROLES {
            assert!(keys.contains(key), "Role token `{key}` is missing from tokens.json {mode}");
        }
        let colors: BTreeMap<String, u32> = obj
            .iter()
            .map(|(k, v)| (k.clone(), hex(v.as_str().expect("hex string"))))
            .collect();
        modes.insert(mode, colors);
    }

    let mut out = String::new();
    let w = &mut out;
    writeln!(w, "// Generated from vendor/codewhale-design/tokens.json {version} by tests/generated.rs.").unwrap();
    writeln!(w, "// Do not edit. Regenerate: CODEWHALE_BLESS=1 cargo test --test generated").unwrap();
    writeln!(w).unwrap();
    writeln!(w, "/// The design tokens version these roles were generated from.").unwrap();
    writeln!(w, "pub const TOKENS_VERSION: &str = \"{version}\";").unwrap();
    writeln!(w).unwrap();
    writeln!(w, "/// What a color is for. Components name roles; they never name colors.").unwrap();
    writeln!(w, "/// Names follow `tokens.json`, so one vocabulary covers the desktop app,").unwrap();
    writeln!(w, "/// the web and the terminal.").unwrap();
    writeln!(w, "#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]").unwrap();
    writeln!(w, "pub enum Role {{").unwrap();
    for (variant, _, doc) in ROLES {
        writeln!(w, "    /// {doc}").unwrap();
        writeln!(w, "    {variant},").unwrap();
    }
    writeln!(w, "}}").unwrap();
    writeln!(w).unwrap();
    writeln!(w, "impl Role {{").unwrap();
    writeln!(w, "    pub const COUNT: usize = {};", ROLES.len()).unwrap();
    writeln!(w, "    pub const ALL: [Role; Self::COUNT] = [").unwrap();
    for (variant, _, _) in ROLES {
        writeln!(w, "        Role::{variant},").unwrap();
    }
    writeln!(w, "    ];").unwrap();
    writeln!(w).unwrap();
    writeln!(w, "    /// The `tokens.json` key this role reads.").unwrap();
    writeln!(w, "    #[must_use]").unwrap();
    writeln!(w, "    pub const fn token_name(self) -> &'static str {{").unwrap();
    writeln!(w, "        match self {{").unwrap();
    for (variant, key, _) in ROLES {
        writeln!(w, "            Role::{variant} => \"{key}\",").unwrap();
    }
    writeln!(w, "        }}").unwrap();
    writeln!(w, "    }}").unwrap();
    writeln!(w).unwrap();
    writeln!(w, "    /// Position in [`Role::ALL`] and in the color tables.").unwrap();
    writeln!(w, "    #[must_use]").unwrap();
    writeln!(w, "    pub const fn index(self) -> usize {{").unwrap();
    writeln!(w, "        self as usize").unwrap();
    writeln!(w, "    }}").unwrap();
    writeln!(w, "}}").unwrap();

    for mode in ["dark", "light"] {
        let colors = &modes[mode];
        let upper = mode.to_uppercase();
        writeln!(w).unwrap();
        writeln!(w, "/// {mode} token colors, `0xRRGGBB`, indexed by [`Role::index`].").unwrap();
        writeln!(w, "pub(crate) const {upper}: [u32; Role::COUNT] = [").unwrap();
        for (variant, key, _) in ROLES {
            writeln!(w, "    0x{:06x}, // {variant}", colors[*key]).unwrap();
        }
        writeln!(w, "];").unwrap();
        let (table, notes) = table_256(colors, mode);
        writeln!(w).unwrap();
        writeln!(w, "/// {mode} xterm 256-color indices (16..=255 only; 0..=15 belong to the").unwrap();
        writeln!(w, "/// user's profile). Nearest index, except where contrast needed a move:").unwrap();
        if notes.is_empty() {
            writeln!(w, "/// none in this table.").unwrap();
        }
        for note in &notes {
            writeln!(w, "/// - {note}").unwrap();
        }
        writeln!(w, "pub(crate) const {upper}_256: [u8; Role::COUNT] = [").unwrap();
        for (variant, key, _) in ROLES {
            writeln!(w, "    {}, // {variant}", table[*key]).unwrap();
        }
        writeln!(w, "];").unwrap();
    }
    out
}

#[test]
fn roles_match_the_vendored_tokens() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source = std::fs::read_to_string(root.join("vendor/codewhale-design/tokens.json"))
        .expect("vendored tokens.json");
    let json: serde_json::Value = serde_json::from_str(&source).expect("tokens.json parses");
    let expected = render(&json);
    let target = root.join("src/roles.rs");
    if std::env::var_os("CODEWHALE_BLESS").is_some() {
        std::fs::write(&target, &expected).expect("write src/roles.rs");
        return;
    }
    let actual = std::fs::read_to_string(&target).unwrap_or_default();
    assert!(
        actual == expected,
        "src/roles.rs is stale for tokens {}; run CODEWHALE_BLESS=1 cargo test --test generated",
        json["version"]
    );
}

/// The vendored `tokens.rs` (from `generate.py`) and our roles agree, so the
/// two generated views of one token file cannot drift apart.
#[test]
fn vendored_tokens_rs_agrees_with_roles() {
    use codewhale_ratatui::{Role, tokens};
    let t = codewhale_ratatui::testing::Profile::DarkTrue.theme();
    assert_eq!(t.token(Role::BorderStrong), rgb(tokens::DARK.border_strong));
    let t = codewhale_ratatui::testing::Profile::LightTrue.theme();
    assert_eq!(t.token(Role::Primary), rgb(tokens::LIGHT.primary));
    assert_eq!(codewhale_ratatui::theme::TOKENS_VERSION, tokens::VERSION);
}

/// `generate.py --check` validates the vendored folder against its own
/// digest. It needs Python; where Python is missing the check says so.
#[test]
fn vendored_folder_passes_its_own_check() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let script = root.join("vendor/codewhale-design/generate.py");
    match std::process::Command::new("python3").arg(&script).arg("--check").output() {
        Ok(out) => assert!(
            out.status.success(),
            "generate.py --check failed:\n{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ),
        Err(err) => eprintln!("skipped: python3 unavailable ({err})"),
    }
}
