//! Codewhale's terminal-native underwater column, as a finishing pass.
//!
//! Adapted from `Hmbown/CodeWhale`'s `crates/tui/src/tui/ocean.rs` at
//! `a79ce5c4d5ed1a5f7032185710c27343a900351c`: the authored three stops,
//! quadratic depth curve, context rise, 90-second phase breath, steady
//! attention/failure tint and 800-millisecond completion breath. The host
//! supplies time and phase; this module owns no clock, event loop or theme.
//!
//! Paint ordinary components first, then apply the column to their ordinary
//! `Background` and `Sidebar` cells. Raised surfaces, selections, diff/code
//! grounds, custom fills, symbols, inks and modifiers remain theirs. The
//! authored dark field is available only on known dark truecolor Ocean
//! grounds. Graphite, light, unknown grounds and lower depths keep the theme.

use std::time::Duration;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier},
};

use crate::{
    Ground, MotionMode, Paint, Role, Theme,
    color::{ColorDepth, blend, contrast_ratio, rgb},
    detect::Appearance,
};

/// A caller-reported phase, matching the native terminal's `ShellPhase`.
/// It is visual input, never a state machine or evidence of work completing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum OceanPhase {
    #[default]
    Idle,
    Typing,
    Working,
    Verifying,
    Waiting,
    Approval,
    Done,
    Failed,
}

impl OceanPhase {
    const fn needs_attention(self) -> bool {
        matches!(self, Self::Waiting | Self::Approval | Self::Failed)
    }
}

/// The native dark underwater ramp. Semantic tints come from [`Theme`].
/// Construct with [`Self::for_theme`]; this does not replace a theme table.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct OceanRamp {
    attention: Color,
    failure: Color,
}

impl OceanRamp {
    /// The native sunlit stop, `#102a45`.
    pub const SURFACE: Color = rgb(0x102a45);
    /// The native middle control point, `#0a1e33`.
    pub const MIDDLE: Color = rgb(0x0a1e33);
    /// The native deep stop, `#061320`.
    pub const DEEP: Color = rgb(0x061320);
    /// The native ambient light, `#264866`.
    pub const AMBIENT: Color = rgb(0x264866);
    /// The authored completion pulse ends after 800 milliseconds.
    pub const COMPLETION_BREATH: Duration = Duration::from_millis(800);

    /// Resolve the native field only where the host selected known dark
    /// truecolor Ocean grounds. A terminal-owned base is never overwritten.
    #[must_use]
    pub fn for_theme(theme: &Theme) -> Option<Self> {
        if theme.depth() != ColorDepth::TrueColor
            || theme.caps().appearance != Appearance::Dark
            || theme.ground_kind() != Ground::Ocean
            || theme
                .native_palette()
                .is_some_and(|palette| palette != crate::TuiPalette::Underwater)
            || !theme.paints_base_ground()
        {
            return None;
        }
        Some(Self {
            attention: theme.color(Role::Attention)?,
            failure: theme.color(Role::Danger)?,
        })
    }

    /// The native quadratic surface → middle → deep curve. Explicit context
    /// fullness (0–100) raises the abyss; omit it at the component level when
    /// the host has no measured context value. Rows outside the column clamp.
    #[must_use]
    pub fn color_at_context(self, row: u16, height: u16, context_percent: u8) -> Color {
        let rise = f32::from(context_percent.min(100)) / 100.0;
        if height <= 1 {
            return blend(Self::DEEP, Self::SURFACE, rise);
        }
        let position = (f32::from(row.min(height - 1)) / f32::from(height - 1) + rise).min(1.0);
        let toward_middle = blend(Self::MIDDLE, Self::SURFACE, position);
        let toward_deep = blend(Self::DEEP, Self::MIDDLE, position);
        blend(toward_deep, toward_middle, position)
    }

    /// The native phase treatment at caller time. Waiting/approval are warm
    /// and failure is steady; neither depends on elapsed time.
    #[must_use]
    pub fn color_at_phase_context(
        self,
        row: u16,
        height: u16,
        elapsed: Duration,
        phase: OceanPhase,
        context_percent: u8,
    ) -> Color {
        if phase.needs_attention() {
            return self.color_at_attention_context(row, height, phase, context_percent);
        }
        let base = self.color_at_context(row, height, context_percent);
        let depth = depth_at(row, height, context_percent);
        let cycle = (elapsed.as_millis() % 90_000) as f32 / 90_000.0;
        let breath = (cycle * std::f32::consts::TAU).sin() * 0.5 + 0.5;
        let (bias, phase_depth) = match phase {
            OceanPhase::Idle => (0.035, 1.0 - depth),
            OceanPhase::Typing => (0.025, 1.0 - depth),
            OceanPhase::Working => (0.045, 0.35 + depth * 0.65),
            OceanPhase::Verifying => (0.055, 0.65 + (1.0 - depth) * 0.35),
            OceanPhase::Done => (0.018, 1.0 - depth),
            OceanPhase::Waiting | OceanPhase::Approval | OceanPhase::Failed => unreachable!(),
        };
        blend(Self::AMBIENT, base, breath * bias * phase_depth)
    }

    /// The native steady attention/failure tint, also used in reduced motion.
    #[must_use]
    pub fn color_at_attention_context(
        self,
        row: u16,
        height: u16,
        phase: OceanPhase,
        context_percent: u8,
    ) -> Color {
        let base = self.color_at_context(row, height, context_percent);
        match phase {
            OceanPhase::Waiting | OceanPhase::Approval => blend(
                self.attention,
                base,
                0.10 * (0.6 + 0.4 * (1.0 - depth_at(row, height, context_percent))),
            ),
            OceanPhase::Failed => blend(self.failure, base, 0.09),
            _ => base,
        }
    }

    /// The native completion brightness: 88% → 112% at 320 ms → 100% at
    /// 800 ms. The component uses it only for a reported `Done` phase in
    /// [`MotionMode::Full`]; a stale completion clock cannot mask failure.
    #[must_use]
    pub fn color_at_completion_context(
        self,
        row: u16,
        height: u16,
        elapsed: Duration,
        context_percent: u8,
    ) -> Color {
        let base = self.color_at_context(row, height, context_percent);
        let t = elapsed.as_millis().min(800) as f32 / 800.0;
        let brightness = if t <= 0.4 {
            0.88 + (1.12 - 0.88) * (t / 0.4)
        } else {
            1.12 + (1.0 - 1.12) * ((t - 0.4) / 0.6)
        };
        let Color::Rgb(r, g, b) = base else {
            return base;
        };
        let scale = |c| (f32::from(c) * brightness).round().clamp(0.0, 255.0) as u8;
        Color::Rgb(scale(r), scale(g), scale(b))
    }
}

fn depth_at(row: u16, height: u16, context_percent: u8) -> f32 {
    if height <= 1 {
        0.0
    } else {
        (f32::from(row.min(height - 1)) / f32::from(height - 1)
            + f32::from(context_percent.min(100)) / 100.0)
            .min(1.0)
    }
}

/// One shared native water column over caller-owned components.
///
/// The host supplies elapsed time and decides when a visible field deserves
/// another redraw. [`MotionMode::Reduced`] and [`MotionMode::Still`] ignore
/// both clocks and keep steady attention/failure tint. ASCII changes no text
/// here: this finishing pass paints backgrounds only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OceanColumn {
    elapsed: Duration,
    motion: MotionMode,
    phase: OceanPhase,
    completion_elapsed: Option<Duration>,
    viewport: Option<Rect>,
    context_percent: u8,
    presence: u16,
}

impl OceanColumn {
    #[must_use]
    pub const fn new(elapsed: Duration, motion: MotionMode) -> Self {
        Self {
            elapsed,
            motion,
            phase: OceanPhase::Idle,
            completion_elapsed: None,
            viewport: None,
            context_percent: 0,
            presence: 1000,
        }
    }

    #[must_use]
    pub const fn phase(mut self, phase: OceanPhase) -> Self {
        self.phase = phase;
        self
    }

    /// Caller-measured context fullness, clamped to 100. The default leaves
    /// the authored column at its normal depth and asserts no usage fact.
    #[must_use]
    pub const fn context_percent(mut self, context_percent: u8) -> Self {
        self.context_percent = if context_percent > 100 {
            100
        } else {
            context_percent
        };
        self
    }

    /// Caller time since an actual successful completion. Honored only in
    /// `Done` with Full motion, and only before the native 800 ms boundary.
    #[must_use]
    pub const fn completion_elapsed(mut self, elapsed: Duration) -> Self {
        self.completion_elapsed = Some(elapsed);
        self
    }

    /// Share this viewport across separate bands so the gradient continues
    /// through them. Without it, the paint area is the complete column.
    #[must_use]
    pub const fn viewport(mut self, viewport: Rect) -> Self {
        self.viewport = Some(viewport);
        self
    }

    /// Host-sampled ambient presence, 0–1000 as in the native renderer.
    /// This eases only phase breathing; attention/failure remain steady and
    /// the successful completion pulse keeps its own gated clock.
    #[must_use]
    pub const fn presence(mut self, presence: u16) -> Self {
        self.presence = if presence > 1000 { 1000 } else { presence };
        self
    }

    /// Pure sampling at an absolute row. `viewport` supplies the default
    /// column bounds unless the builder already specifies shared bounds.
    /// Returns `None` where the native authored field is unavailable.
    #[must_use]
    pub fn color_at_y(&self, y: u16, viewport: Rect, theme: &Theme) -> Option<Color> {
        let ramp = OceanRamp::for_theme(theme)?;
        let viewport = self.viewport.unwrap_or(viewport);
        Some(self.sample(y, viewport, ramp))
    }

    fn sample(&self, y: u16, viewport: Rect, ramp: OceanRamp) -> Color {
        let height = viewport.height.max(1);
        let row = y.saturating_sub(viewport.y).min(height - 1);
        if self.motion.animates()
            && self.phase == OceanPhase::Done
            && let Some(elapsed) = self
                .completion_elapsed
                .filter(|t| *t < OceanRamp::COMPLETION_BREATH)
        {
            return ramp.color_at_completion_context(row, height, elapsed, self.context_percent);
        }
        if self.phase.needs_attention() {
            return ramp.color_at_attention_context(row, height, self.phase, self.context_percent);
        }
        let base = ramp.color_at_context(row, height, self.context_percent);
        if !self.motion.animates() || self.presence == 0 {
            return base;
        }
        let phase = ramp.color_at_phase_context(
            row,
            height,
            self.elapsed,
            self.phase,
            self.context_percent,
        );
        blend(phase, base, f32::from(self.presence) / 1000.0)
    }

    /// Finish ordinary `Background` and `Sidebar` cells, clipped to the
    /// buffer. Other fills remain exact. Visible inks must retain their
    /// contrast floor; unknown terminal inks and reversed cells are spared.
    pub fn apply(&self, area: Rect, buf: &mut Buffer, theme: &Theme) {
        let Some(ramp) = OceanRamp::for_theme(theme) else {
            return;
        };
        let viewport = self.viewport.unwrap_or(area);
        let area = area.intersection(buf.area);
        let background = theme.bg(Role::Background).bg;
        let sidebar = theme.bg(Role::Sidebar).bg;
        for y in area.top()..area.bottom() {
            let water = self.sample(y, viewport, ramp);
            // Text runs repeat the same ink on one shared row ground. Reuse
            // its contrast verdict without allocating or caching theme state
            // across frames; custom colors still take the same safety path.
            let mut previous_ink = None;
            for x in area.left()..area.right() {
                let cell = &mut buf[(x, y)];
                if (Some(cell.bg) == background || Some(cell.bg) == sidebar)
                    && !cell.modifier.contains(Modifier::REVERSED)
                {
                    let safe = cell.symbol() == " " || {
                        if let Some((ink, safe)) = previous_ink
                            && ink == cell.fg
                        {
                            safe
                        } else {
                            let safe = ink_is_safe(cell.fg, water, theme);
                            previous_ink = Some((cell.fg, safe));
                            safe
                        }
                    };
                    if safe {
                        cell.set_bg(water);
                    }
                }
            }
        }
    }
}

impl Paint for OceanColumn {
    fn paint(&self, area: Rect, buf: &mut Buffer, theme: &Theme) {
        self.apply(area, buf, theme);
    }
}

fn ink_is_safe(ink: Color, water: Color, theme: &Theme) -> bool {
    let floor = if [Role::Border, Role::BorderStrong, Role::Dim]
        .into_iter()
        .any(|role| theme.color(role) == Some(ink))
    {
        // Border is purely decorative and deliberately has no text floor.
        if theme.color(Role::Border) == Some(ink) {
            1.0
        } else {
            3.0
        }
    } else {
        4.5
    };
    contrast_ratio(ink, water).is_some_and(|ratio| ratio >= floor)
}
