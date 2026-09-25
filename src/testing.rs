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

use crate::{Caps, Theme, color::ColorDepth, detect::Appearance, text, theme::Ground};

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
