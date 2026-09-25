//! One theme: color roles from the Codewhale design tokens.
//!
//! Components name a [`Role`]; they never name a color. [`Theme`] resolves a
//! role for the terminal it is painting on, every time it paints, so a theme
//! or depth change reaches every component on the next frame.
//!
//! | Depth | Grounds | Ink | State hues |
//! |---|---|---|---|
//! | TrueColor | exact token RGB | exact RGB | exact RGB |
//! | Ansi256 | nearest fixed index | generated table, audited for contrast | generated table |
//! | Ansi16 | none (the terminal owns its ground) | `Reset`; `Muted` adds `DIM` | Blue, Green, Yellow, Red |
//! | Monochrome (`NO_COLOR`) | none | `Reset`; hierarchy by `BOLD`/`DIM` only | `Reset`: the mark and word carry state |
//!
//! When nothing measured the terminal's ground ([`Appearance::Unknown`]), a
//! truecolor terminal is painted as at ANSI-16: its named colors were chosen
//! for its own ground, and ours were not.

use ratatui::style::{Color, Modifier, Style};

use crate::color::{ColorDepth, rgb};
use crate::detect::{Appearance, terminal_background};
use crate::roles;

pub use crate::roles::{Role, TOKENS_VERSION};

impl Role {
    #[must_use]
    pub const fn is_ground(self) -> bool {
        matches!(
            self,
            Role::Sidebar | Role::Background | Role::Surface | Role::Hover | Role::Selected
        )
    }

    /// The named color a 16-color terminal shows for this role, or `None`
    /// when the role takes the terminal's own foreground.
    const fn ansi16(self) -> Option<Color> {
        match self {
            Role::Primary => Some(Color::Blue),
            Role::Live => Some(Color::Green),
            Role::Attention => Some(Color::Yellow),
            Role::Danger => Some(Color::Red),
            _ => None,
        }
    }

    /// Ink that recedes. Where color is unavailable it recedes with `DIM`.
    const fn recedes(self) -> bool {
        matches!(self, Role::Muted | Role::Border)
    }
}

/// What the terminal can show.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Caps {
    pub depth: ColorDepth,
    /// `CODEWHALE_ASCII_SAFE`: draw marks from the ASCII fallbacks.
    pub ascii: bool,
    pub appearance: Appearance,
}

impl Caps {
    /// Read depth, ASCII-safety and appearance from the environment and any
    /// completed [`crate::detect::probe_terminal_background`].
    #[must_use]
    pub fn detect() -> Self {
        let ascii = std::env::var_os("CODEWHALE_ASCII_SAFE")
            .is_some_and(|v| !v.is_empty() && v != "0" && v != "false");
        Self {
            depth: ColorDepth::detect(),
            ascii,
            appearance: terminal_background().appearance(),
        }
    }

    /// Whether this terminal shows token colors at all.
    #[must_use]
    pub const fn paints_tokens(&self) -> bool {
        matches!(self.depth, ColorDepth::TrueColor | ColorDepth::Ansi256)
            && !matches!(self.appearance, Appearance::Unknown)
    }
}

/// The Codewhale theme resolved for one terminal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Theme {
    caps: Caps,
    grounds: bool,
}

impl Theme {
    #[must_use]
    pub const fn new(caps: Caps) -> Self {
        Self {
            caps,
            grounds: true,
        }
    }

    /// Detect everything from the environment.
    #[must_use]
    pub fn detect() -> Self {
        Self::new(Caps::detect())
    }

    /// Leave the terminal's own ground alone, even where token grounds would
    /// render. Raised panels and selection still paint.
    #[must_use]
    pub const fn without_base_ground(mut self) -> Self {
        self.grounds = false;
        self
    }

    #[must_use]
    pub const fn caps(&self) -> Caps {
        self.caps
    }

    #[must_use]
    pub const fn ascii(&self) -> bool {
        self.caps.ascii
    }

    #[must_use]
    pub const fn depth(&self) -> ColorDepth {
        self.caps.depth
    }

    /// Which token table this theme reads. Unknown reads the dark table,
    /// but [`Caps::paints_tokens`] is false then, so no token color reaches
    /// the screen.
    const fn light(&self) -> bool {
        matches!(self.caps.appearance, Appearance::Light)
    }

    /// The exact token color (`0xRRGGBB`), before depth adaptation.
    #[must_use]
    pub const fn token_hex(&self, role: Role) -> u32 {
        if self.light() {
            roles::LIGHT[role.index()]
        } else {
            roles::DARK[role.index()]
        }
    }

    /// The exact token color, before depth adaptation.
    #[must_use]
    pub const fn token(&self, role: Role) -> Color {
        rgb(self.token_hex(role))
    }

    /// The color this terminal shows for `role`, or `None` when it shows the
    /// terminal's own color.
    #[must_use]
    pub const fn color(&self, role: Role) -> Option<Color> {
        if self.caps.paints_tokens() {
            return Some(match self.caps.depth {
                ColorDepth::TrueColor => self.token(role),
                _ if self.light() => Color::Indexed(roles::LIGHT_256[role.index()]),
                _ => Color::Indexed(roles::DARK_256[role.index()]),
            });
        }
        match self.caps.depth {
            ColorDepth::Monochrome => None,
            _ => role.ansi16(),
        }
    }

    /// Ink: `role` as a foreground.
    #[must_use]
    pub fn fg(&self, role: Role) -> Style {
        let style = Style::default();
        match self.color(role) {
            Some(color) => style.fg(color),
            None if role.recedes() => style.add_modifier(Modifier::DIM),
            None => style,
        }
    }

    /// Ground: `role` as a background, or nothing where grounds do not paint.
    /// The base `Background` also stays unpainted after
    /// [`Theme::without_base_ground`].
    #[must_use]
    pub fn bg(&self, role: Role) -> Style {
        if !self.caps.paints_tokens() || (!self.grounds && role == Role::Background) {
            return Style::default();
        }
        match self.color(role) {
            Some(color) => Style::default().bg(color),
            None => Style::default(),
        }
    }

    /// Whether grounds paint at all. Where they do not, components must draw
    /// an edge or a mark instead of relying on a fill.
    #[must_use]
    pub fn paints_grounds(&self) -> bool {
        self.caps.paints_tokens()
    }

    /// Reverse lookup for snapshots: which role (if any) resolves to this
    /// color. Grounds are preferred for backgrounds, ink for foregrounds.
    #[must_use]
    pub fn role_of(&self, color: Color, as_ground: bool) -> Option<Role> {
        let matches: Vec<Role> = Role::ALL
            .iter()
            .copied()
            .filter(|r| self.color(*r) == Some(color))
            .collect();
        matches
            .iter()
            .copied()
            .find(|r| r.is_ground() == as_ground)
            .or_else(|| matches.first().copied())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::contrast_ratio;

    fn theme(depth: ColorDepth, appearance: Appearance) -> Theme {
        Theme::new(Caps {
            depth,
            ascii: false,
            appearance,
        })
    }

    #[test]
    fn truecolor_reads_the_token_tables() {
        let dark = theme(ColorDepth::TrueColor, Appearance::Dark);
        assert_eq!(
            dark.color(Role::Primary),
            Some(rgb(crate::tokens::DARK.primary))
        );
        let light = theme(ColorDepth::TrueColor, Appearance::Light);
        assert_eq!(
            light.color(Role::Danger),
            Some(rgb(crate::tokens::LIGHT.danger))
        );
    }

    #[test]
    fn unknown_ground_never_paints_token_colors() {
        let t = theme(ColorDepth::TrueColor, Appearance::Unknown);
        assert_eq!(t.bg(Role::Background), Style::default());
        assert_eq!(t.color(Role::Foreground), None);
        assert_eq!(t.color(Role::Live), Some(Color::Green));
        assert!(t.fg(Role::Muted).add_modifier.contains(Modifier::DIM));
    }

    #[test]
    fn no_color_uses_only_modifiers() {
        let t = theme(ColorDepth::Monochrome, Appearance::Dark);
        for role in Role::ALL {
            let fg = t.fg(role);
            assert_eq!(fg.fg, None, "{role:?}");
            assert_eq!(t.bg(role).bg, None, "{role:?}");
        }
        assert!(t.fg(Role::Muted).add_modifier.contains(Modifier::DIM));
    }

    #[test]
    fn ansi16_state_hues_are_distinct() {
        let t = theme(ColorDepth::Ansi16, Appearance::Dark);
        let hues: Vec<_> = [Role::Primary, Role::Live, Role::Attention, Role::Danger]
            .iter()
            .map(|r| t.color(*r).unwrap())
            .collect();
        for (i, a) in hues.iter().enumerate() {
            for b in &hues[i + 1..] {
                assert_ne!(a, b);
            }
        }
        assert_eq!(t.bg(Role::Selected), Style::default());
    }

    /// Contrast audit at the two depths that paint token colors, with the
    /// design's own floors (`generate.py`): every text role clears WCAG AA
    /// (4.5:1) on every ground it can sit on, and the strong border 3:1.
    #[test]
    fn contrast_holds_at_truecolor_and_ansi256() {
        let grounds = [
            Role::Background,
            Role::Surface,
            Role::Sidebar,
            Role::Hover,
            Role::Selected,
        ];
        let mut failures = Vec::new();
        for appearance in [Appearance::Dark, Appearance::Light] {
            for depth in [ColorDepth::TrueColor, ColorDepth::Ansi256] {
                let t = theme(depth, appearance);
                for ground in grounds {
                    let bg = t.color(ground).unwrap();
                    for (ink, floor) in [
                        (Role::Foreground, 4.5),
                        (Role::Muted, 4.5),
                        (Role::Primary, 4.5),
                        (Role::Live, 4.5),
                        (Role::Attention, 4.5),
                        (Role::Danger, 4.5),
                        (Role::BorderStrong, 3.0),
                    ] {
                        let ratio = contrast_ratio(t.color(ink).unwrap(), bg).unwrap();
                        if ratio < floor {
                            failures.push(format!(
                                "{appearance:?} {depth:?} {ink:?} on {ground:?}: {ratio:.2} < {floor}"
                            ));
                        }
                    }
                }
                let on_primary = contrast_ratio(
                    t.color(Role::PrimaryForeground).unwrap(),
                    t.color(Role::Primary).unwrap(),
                )
                .unwrap();
                if on_primary < 4.5 {
                    failures.push(format!(
                        "{appearance:?} {depth:?} PrimaryForeground on Primary: {on_primary:.2} < 4.5"
                    ));
                }
            }
        }
        assert!(
            failures.is_empty(),
            "contrast failures:\n{}",
            failures.join("\n")
        );
    }
}
