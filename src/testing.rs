//! Render components off-screen for snapshots, the gallery and host tests.
//!
//! Styles are recorded by role, not by hex, so a token value change does not
//! rewrite every snapshot, but painting `Muted` where `Danger` belongs does.

use std::fmt::Write as _;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
};

use crate::{Caps, Role, Theme, color::ColorDepth, detect::Appearance, text, theme::Ground};

/// A terminal to render for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Profile {
    /// Truecolor on a dark ground, in the blue ombre (the default).
    DarkTrue,
    /// Truecolor on a dark ground, with the graphite token grounds.
    DarkGraphite,
    LightTrue,
    Dark256,
    Light256,
    /// 16 colors on a measured dark ground.
    Ansi16,
    /// Truecolor, but nothing measured the ground.
    UnknownGround,
    /// `NO_COLOR`.
    NoColor,
    /// `NO_COLOR` plus `CODEWHALE_ASCII_SAFE`.
    Ascii,
}

impl Profile {
    pub const ALL: [Profile; 9] = [
        Profile::DarkTrue,
        Profile::DarkGraphite,
        Profile::LightTrue,
        Profile::Dark256,
        Profile::Light256,
        Profile::Ansi16,
        Profile::UnknownGround,
        Profile::NoColor,
        Profile::Ascii,
    ];

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Profile::DarkTrue => "dark-truecolor",
            Profile::DarkGraphite => "dark-graphite",
            Profile::LightTrue => "light-truecolor",
            Profile::Dark256 => "dark-256",
            Profile::Light256 => "light-256",
            Profile::Ansi16 => "ansi-16",
            Profile::UnknownGround => "unknown-ground",
            Profile::NoColor => "no-color",
            Profile::Ascii => "ascii",
        }
    }

    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|p| p.name() == name)
    }

    #[must_use]
    pub const fn caps(self) -> Caps {
        let (depth, appearance, ascii) = match self {
            Profile::DarkTrue | Profile::DarkGraphite => {
                (ColorDepth::TrueColor, Appearance::Dark, false)
            }
            Profile::LightTrue => (ColorDepth::TrueColor, Appearance::Light, false),
            Profile::Dark256 => (ColorDepth::Ansi256, Appearance::Dark, false),
            Profile::Light256 => (ColorDepth::Ansi256, Appearance::Light, false),
            Profile::Ansi16 => (ColorDepth::Ansi16, Appearance::Dark, false),
            Profile::UnknownGround => (ColorDepth::TrueColor, Appearance::Unknown, false),
            Profile::NoColor => (ColorDepth::Monochrome, Appearance::Dark, false),
            Profile::Ascii => (ColorDepth::Monochrome, Appearance::Dark, true),
        };
        Caps {
            depth,
            ascii,
            appearance,
        }
    }

    #[must_use]
    pub const fn theme(self) -> Theme {
        let theme = Theme::new(self.caps());
        match self {
            Profile::DarkGraphite => theme.ground(Ground::Graphite),
            _ => theme,
        }
    }
}

/// The widths a component is checked at: a narrow terminal, the classic 80
/// columns, and a wide one.
pub const WIDTHS: [u16; 3] = [40, 80, 120];

/// A component painted for one profile at one width: what a component test
/// looks at.
pub struct Frame {
    pub name: String,
    pub profile: Profile,
    pub theme: Theme,
    pub buf: Buffer,
}

impl Frame {
    #[must_use]
    pub fn new(name: impl Into<String>, profile: Profile, buf: Buffer) -> Self {
        Self {
            name: name.into(),
            profile,
            theme: profile.theme(),
            buf,
        }
    }

    #[must_use]
    pub fn width(&self) -> u16 {
        self.buf.area.width
    }

    #[must_use]
    pub fn height(&self) -> u16 {
        self.buf.area.height
    }

    /// `name · profile · 80x3` (no name when the frame has none), for failure
    /// messages and dump headers.
    #[must_use]
    pub fn label(&self) -> String {
        let (profile, width, height) = (self.profile.name(), self.width(), self.height());
        if self.name.is_empty() {
            format!("{profile} · {width}x{height}")
        } else {
            format!("{} · {profile} · {width}x{height}", self.name)
        }
    }

    /// The glyphs, one line per row ([`text`]).
    #[must_use]
    pub fn text(&self) -> String {
        text(&self.buf)
    }

    /// The glyphs and the role each run was painted with ([`styled`]).
    #[must_use]
    pub fn styled(&self) -> String {
        styled(&self.buf, &self.theme)
    }

    /// Every rule this frame breaks; empty when it keeps them all. See
    /// [`rule_violations`].
    #[must_use]
    pub fn violations(&self) -> Vec<String> {
        rule_violations(self)
    }
}

/// Paint `paint` for each of `profiles` at each of `widths`, `height` rows
/// tall, on the theme's `Background` as the gallery paints it.
pub fn frames_for(
    name: &str,
    profiles: &[Profile],
    widths: &[u16],
    height: u16,
    paint: impl Fn(Rect, &mut Buffer, &Theme),
) -> Vec<Frame> {
    let mut frames = Vec::new();
    for profile in profiles {
        let theme = profile.theme();
        for width in widths {
            let buf = render(*width, height, |area, buf| {
                buf.set_style(area, theme.bg(Role::Background));
                paint(area, buf, &theme);
            });
            frames.push(Frame::new(name, *profile, buf));
        }
    }
    frames
}

/// The one-call render for component tests: `paint` at 40, 80 and 120
/// columns ([`WIDTHS`]) in every [`Profile`], `height` rows tall.
pub fn frames(height: u16, paint: impl Fn(Rect, &mut Buffer, &Theme)) -> Vec<Frame> {
    frames_for("", &Profile::ALL, &WIDTHS, height, paint)
}

/// Fail, listing every broken rule, if any frame of `paint` (as [`frames`])
/// breaks a rule ([`rule_violations`]).
pub fn assert_rules(height: u16, paint: impl Fn(Rect, &mut Buffer, &Theme)) {
    assert_frames_keep_the_rules(&frames(height, paint));
}

/// Fail, listing every broken rule, if any of `frames` breaks one.
pub fn assert_frames_keep_the_rules(frames: &[Frame]) {
    let mut broken: Vec<String> = frames.iter().flat_map(Frame::violations).collect();
    broken.dedup();
    assert!(broken.is_empty(), "{}", broken.join("\n"));
}

/// The text to hand `insta::assert_snapshot!` for a component: the design's
/// matrix. `DarkTrue` at every width with the role of each run (`styled`),
/// then `NoColor` and `Ascii` at every width as glyphs only (`text`).
/// Painting the wrong role changes it; a token value change does not.
pub fn snapshot(height: u16, paint: impl Fn(Rect, &mut Buffer, &Theme)) -> String {
    let mut out = String::new();
    for frame in frames_for("", &[Profile::DarkTrue], &WIDTHS, height, &paint) {
        let _ = writeln!(out, "== {}\n{}", frame.label(), frame.styled());
    }
    for frame in frames_for(
        "",
        &[Profile::NoColor, Profile::Ascii],
        &WIDTHS,
        height,
        &paint,
    ) {
        let _ = writeln!(out, "== {}\n{}\n", frame.label(), frame.text());
    }
    out
}

/// Like [`snapshot`], but every profile styled: what the gallery's own
/// snapshots record. Large; prefer [`snapshot`] for a new component.
pub fn snapshot_all_profiles(height: u16, paint: impl Fn(Rect, &mut Buffer, &Theme)) -> String {
    let mut out = String::new();
    for frame in frames(height, paint) {
        let _ = writeln!(out, "== {}\n{}\n", frame.label(), frame.styled());
    }
    out
}

/// What each profile may put on the screen.
#[must_use]
pub fn color_allowed(profile: Profile, color: Color) -> bool {
    match (profile, color) {
        (_, Color::Reset) => true,
        (Profile::NoColor | Profile::Ascii, _) => false,
        (Profile::Ansi16 | Profile::UnknownGround, Color::Rgb(..) | Color::Indexed(_)) => false,
        (Profile::Dark256 | Profile::Light256, Color::Rgb(..)) => false,
        (Profile::Dark256 | Profile::Light256, Color::Indexed(i)) => i >= 16,
        // Truecolor on a known ground paints exact RGB: tokens, and the
        // whale's ombre and the horizon's fade blended from them.
        (Profile::DarkTrue | Profile::DarkGraphite | Profile::LightTrue, c) => {
            matches!(c, Color::Rgb(..))
        }
        (Profile::Dark256 | Profile::Light256, _) => false,
        _ => true,
    }
}

/// The rules every frame keeps, as the list of those `frame` breaks:
///
/// - at most one horizon rule (a row of `─` from the left edge covering more
///   than half the width), and no `═`, `≈` or `∿` rule;
/// - no `...`: the one ellipsis is `…` (in ASCII-safe output `...` is the
///   honest form, and everything must be ASCII);
/// - only the colors the profile may show ([`color_allowed`]), and no ground
///   painted where the terminal cannot show one.
#[must_use]
pub fn rule_violations(frame: &Frame) -> Vec<String> {
    let mut broken = Vec::new();
    let (profile, theme) = (frame.profile, &frame.theme);
    let text = frame.text();
    let at = frame.label();
    // ASCII has no `…`; there `...` is the honest ellipsis.
    if profile != Profile::Ascii && text.contains("...") {
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
        // A horizon runs from the left edge; a panel's edge starts with a
        // corner.
        .filter(|l| {
            l.starts_with('─')
                && l.chars().filter(|c| *c == '─').count() * 2 > usize::from(frame.width())
        })
        .count();
    if rules > 1 {
        broken.push(format!("{at}: {rules} horizons; one per frame"));
    }
    for cell in frame.buf.content() {
        for color in [cell.fg, cell.bg] {
            if !color_allowed(profile, color) {
                broken.push(format!("{at}: {color:?} is not allowed here"));
                break;
            }
        }
    }
    if !theme.paints_grounds() && frame.buf.content().iter().any(|c| c.bg != Color::Reset) {
        broken.push(format!("{at}: painted a ground the terminal cannot show"));
    }
    broken
}

/// Paint into a fresh `width` x `height` buffer.
pub fn render(width: u16, height: u16, paint: impl FnOnce(Rect, &mut Buffer)) -> Buffer {
    let area = Rect::new(0, 0, width, height);
    let mut buf = Buffer::empty(area);
    paint(area, &mut buf);
    buf
}

/// Visit each drawn cell once, skipping the cells a wide glyph covers.
fn cells(buf: &Buffer, mut visit: impl FnMut(u16, u16, &str, Style)) {
    let area = buf.area;
    for y in area.top()..area.bottom() {
        let mut x = area.left();
        while x < area.right() {
            let cell = &buf[(x, y)];
            let symbol = cell.symbol();
            visit(x, y, symbol, cell.style());
            x += u16::try_from(text::width(symbol).max(1)).unwrap_or(1);
        }
    }
}

/// The glyphs, one line per row, trailing spaces trimmed.
#[must_use]
pub fn text(buf: &Buffer) -> String {
    let mut rows = vec![String::new(); usize::from(buf.area.height)];
    cells(buf, |_, y, symbol, _| {
        rows[usize::from(y - buf.area.y)].push_str(symbol)
    });
    rows.iter()
        .map(|r| r.trim_end())
        .collect::<Vec<_>>()
        .join("\n")
}

/// The role a color was painted as. Roles that collapse to one color at
/// this depth are all named (`Background|Surface`), so a collapse shows in
/// the snapshot instead of hiding behind whichever role comes first.
fn color_name(color: Color, theme: &Theme, ground: bool) -> String {
    let roles = theme.roles_of(color, ground);
    if !roles.is_empty() {
        return roles
            .iter()
            .map(|r| format!("{r:?}"))
            .collect::<Vec<_>>()
            .join("|");
    }
    match color {
        Color::Rgb(r, g, b) => format!("#{r:02x}{g:02x}{b:02x}"),
        Color::Indexed(i) => format!("idx{i}"),
        other => format!("{other:?}"),
    }
}

fn describe(style: Style, theme: &Theme) -> String {
    let mut parts = Vec::new();
    if let Some(fg) = style.fg.filter(|c| *c != Color::Reset) {
        parts.push(format!("fg={}", color_name(fg, theme, false)));
    }
    if let Some(bg) = style.bg.filter(|c| *c != Color::Reset) {
        parts.push(format!("bg={}", color_name(bg, theme, true)));
    }
    for (m, name) in [
        (Modifier::BOLD, "bold"),
        (Modifier::DIM, "dim"),
        (Modifier::ITALIC, "italic"),
        (Modifier::UNDERLINED, "underline"),
        (Modifier::REVERSED, "reversed"),
    ] {
        if style.add_modifier.contains(m) {
            parts.push(name.to_string());
        }
    }
    parts.join(" ")
}

/// The glyphs, then one line per run of identical style:
/// `r3 c2..9 fg=Live bold`. Unstyled runs are left out.
#[must_use]
pub fn styled(buf: &Buffer, theme: &Theme) -> String {
    let mut out = text(buf);
    out.push_str("\n---\n");
    let mut run: Option<(u16, u16, u16, String)> = None;
    let flush = |run: &mut Option<(u16, u16, u16, String)>, out: &mut String| {
        if let Some((y, x0, x1, d)) = run.take()
            && !d.is_empty()
        {
            let _ = writeln!(out, "r{y} c{x0}..{x1} {d}");
        }
    };
    cells(buf, |x, y, symbol, style| {
        let d = describe(style, theme);
        let w = u16::try_from(text::width(symbol).max(1)).unwrap_or(1);
        match &mut run {
            Some((ry, _, x1, rd)) if *ry == y && *rd == d && *x1 == x => *x1 = x + w,
            _ => {
                flush(&mut run, &mut out);
                run = Some((y, x, x + w, d));
            }
        }
    });
    flush(&mut run, &mut out);
    out
}

fn sgr_color(color: Color, ground: bool) -> String {
    let base = if ground { 40 } else { 30 };
    match color {
        Color::Reset => format!("{}", base + 9),
        Color::Rgb(r, g, b) => format!("{};2;{r};{g};{b}", base + 8),
        Color::Indexed(i) => format!("{};5;{i}", base + 8),
        named => {
            let (n, bright) = match named {
                Color::Black => (0, false),
                Color::Red => (1, false),
                Color::Green => (2, false),
                Color::Yellow => (3, false),
                Color::Blue => (4, false),
                Color::Magenta => (5, false),
                Color::Cyan => (6, false),
                Color::Gray => (7, false),
                Color::DarkGray => (0, true),
                Color::LightRed => (1, true),
                Color::LightGreen => (2, true),
                Color::LightYellow => (3, true),
                Color::LightBlue => (4, true),
                Color::LightMagenta => (5, true),
                Color::LightCyan => (6, true),
                _ => (7, true),
            };
            format!("{}", if bright { base + 60 + n } else { base + n })
        }
    }
}

/// The buffer as text with ANSI SGR escapes, for `cat` in any terminal.
#[must_use]
pub fn ansi(buf: &Buffer) -> String {
    let mut out = String::new();
    let mut last: Option<Style> = None;
    let mut row = buf.area.y;
    cells(buf, |_, y, symbol, style| {
        if y != row {
            out.push_str("\x1b[0m\n");
            row = y;
            last = None;
        }
        if last != Some(style) {
            let mut codes = vec!["0".to_string()];
            for (m, code) in [
                (Modifier::BOLD, "1"),
                (Modifier::DIM, "2"),
                (Modifier::ITALIC, "3"),
                (Modifier::UNDERLINED, "4"),
                (Modifier::REVERSED, "7"),
            ] {
                if style.add_modifier.contains(m) {
                    codes.push(code.into());
                }
            }
            if let Some(fg) = style.fg {
                codes.push(sgr_color(fg, false));
            }
            if let Some(bg) = style.bg {
                codes.push(sgr_color(bg, true));
            }
            let _ = write!(out, "\x1b[{}m", codes.join(";"));
            last = Some(style);
        }
        out.push_str(symbol);
    });
    out.push_str("\x1b[0m\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{HorizonRule, Paint};

    #[test]
    fn frames_cover_every_profile_at_every_width() {
        let frames = frames(2, |_, _, _| {});
        assert_eq!(frames.len(), Profile::ALL.len() * WIDTHS.len());
        for profile in Profile::ALL {
            let widths: Vec<u16> = frames
                .iter()
                .filter(|f| f.profile == profile)
                .map(Frame::width)
                .collect();
            assert_eq!(widths, WIDTHS, "{}", profile.name());
        }
        assert!(frames.iter().all(|f| f.height() == 2));
    }

    #[test]
    fn the_rules_catch_what_they_name() {
        let two_horizons = |area: Rect, buf: &mut Buffer, theme: &Theme| {
            for y in 0..2 {
                HorizonRule::new().paint(
                    Rect {
                        y,
                        height: 1,
                        ..area
                    },
                    buf,
                    theme,
                );
            }
        };
        let broken: Vec<String> = frames(2, two_horizons)
            .iter()
            .flat_map(Frame::violations)
            .collect();
        assert!(
            broken.iter().any(|b| b.contains("2 horizons")),
            "{broken:?}"
        );

        let dots = |area: Rect, buf: &mut Buffer, _: &Theme| {
            buf.set_string(area.x, area.y, "Loading...", Style::default());
        };
        let broken: Vec<String> = frames(1, dots).iter().flat_map(Frame::violations).collect();
        assert!(broken.iter().any(|b| b.contains("ellipsis")), "{broken:?}");
        // In ASCII-safe output `...` is honest.
        assert!(
            !broken
                .iter()
                .any(|b| b.contains("ascii") && b.contains("ellipsis")),
            "{broken:?}"
        );

        let raw = |area: Rect, buf: &mut Buffer, _: &Theme| {
            buf.set_string(
                area.x,
                area.y,
                "x",
                Style::default().fg(Color::Rgb(1, 2, 3)),
            );
        };
        let broken: Vec<String> = frames(1, raw).iter().flat_map(Frame::violations).collect();
        assert!(broken.iter().any(|b| b.contains("no-color")), "{broken:?}");
    }

    #[test]
    fn the_snapshot_text_names_every_width_and_profile_it_records() {
        let text = snapshot(1, |area, buf, theme| {
            buf.set_string(area.x, area.y, "hi", theme.fg(Role::Live));
        });
        for expected in [
            "== dark-truecolor · 40x1",
            "== dark-truecolor · 80x1",
            "== dark-truecolor · 120x1",
            "== no-color · 40x1",
            "== ascii · 120x1",
            "fg=Live",
        ] {
            assert!(text.contains(expected), "missing {expected}:\n{text}");
        }
    }
}
