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
//! Native chrome can join the same column with [`OceanColumn::apply_matching`]
//! over its own region and explicit base ground.

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
/// [`Self::for_theme`] resolves the guarded default; [`Self::new`] supplies
/// exact caller colors for pure math. Neither replaces a theme table.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct OceanRamp {
    surface: Color,
    middle: Color,
    deep: Color,
    ambient: Color,
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

    /// An explicit caller-owned ramp. Pure sampling may use these exact
    /// colors; guarded column painting still requires the normal theme and
    /// capability gates and preserves its contrast policy.
    #[must_use]
    pub const fn new(
        surface: Color,
        middle: Color,
        deep: Color,
        ambient: Color,
        attention: Color,
        failure: Color,
    ) -> Self {
        Self {
            surface,
            middle,
            deep,
            ambient,
            attention,
            failure,
        }
    }

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
        Some(Self::new(
            Self::SURFACE,
            Self::MIDDLE,
            Self::DEEP,
            Self::AMBIENT,
            theme.color(Role::Attention)?,
            theme.color(Role::Danger)?,
        ))
    }

    /// The native quadratic surface → middle → deep curve. Explicit context
    /// fullness (0–100) raises the abyss; omit it at the component level when
    /// the host has no measured context value. Rows outside the column clamp.
    #[must_use]
    pub fn color_at_context(self, row: u16, height: u16, context_percent: u8) -> Color {
        let rise = f32::from(context_percent.min(100)) / 100.0;
        if height <= 1 {
            return mix_toward(self.surface, self.deep, rise);
        }
        let position = (f32::from(row.min(height - 1)) / f32::from(height - 1) + rise).min(1.0);
        let toward_middle = mix_toward(self.surface, self.middle, position);
        let toward_deep = mix_toward(self.middle, self.deep, position);
        mix_toward(toward_middle, toward_deep, position)
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
        mix_toward(base, self.ambient, breath * bias * phase_depth)
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
            OceanPhase::Waiting | OceanPhase::Approval => mix_toward(
                base,
                self.attention,
                0.10 * (0.6 + 0.4 * (1.0 - depth_at(row, height, context_percent))),
            ),
            OceanPhase::Failed => mix_toward(base, self.failure, 0.09),
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

// Explicit native ramps may carry a terminal-owned or indexed tint. There
// is no RGB interpolation evidence in that case; retain the source color.
fn mix_toward(from: Color, to: Color, amount: f32) -> Color {
    if matches!((from, to), (Color::Rgb(..), Color::Rgb(..))) {
        blend(to, from, amount)
    } else {
        from
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
    ramp: Option<OceanRamp>,
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
            ramp: None,
            elapsed,
            motion,
            phase: OceanPhase::Idle,
            completion_elapsed: None,
            viewport: None,
            context_percent: 0,
            presence: 1000,
        }
    }

    /// Supply the host's actual ramp without granting paint capability.
    /// `color_at_y`, `apply` and `apply_matching` keep their existing theme,
    /// terminal, semantic-surface and contrast guards.
    #[must_use]
    pub const fn ramp(mut self, ramp: OceanRamp) -> Self {
        self.ramp = Some(ramp);
        self
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
        Some(self.color_at_y_with_ramp(y, viewport, ramp))
    }

    /// Pure sampling under a caller-owned rendering policy.
    /// Like the ramp's math helpers, this paints nothing and has no terminal
    /// capability gate; guarded sampling and painting keep their own gates.
    /// A builder-supplied `ramp` overrides this fallback ramp; `viewport`
    /// likewise retains the shared absolute column. Motion, completion,
    /// phase, context and presence follow the same kernel as guarded paint.
    /// Hosts must keep their existing paint/terminal guards around the result.
    #[must_use]
    pub fn color_at_y_with_ramp(&self, y: u16, viewport: Rect, ramp: OceanRamp) -> Color {
        let viewport = self.viewport.unwrap_or(viewport);
        let ramp = self.ramp.unwrap_or(ramp);
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
        mix_toward(base, phase, f32::from(self.presence) / 1000.0)
    }

    /// Finish ordinary `Background` and `Sidebar` cells, clipped to the
    /// buffer. Other fills remain exact. Visible inks must retain their
    /// contrast floor; unknown terminal inks and reversed cells are spared.
    pub fn apply(&self, area: Rect, buf: &mut Buffer, theme: &Theme) {
        self.apply_grounds(
            area,
            buf,
            theme,
            &[theme.bg(Role::Background).bg, theme.bg(Role::Sidebar).bg],
        );
    }

    /// Continue the column through cells with the caller's explicit base
    /// `ground`, restricted to `area`. Use [`Self::viewport`] to share one
    /// absolute water column across conversation, composer and footer bands.
    ///
    /// Other grounds, symbols, inks and modifiers remain untouched. The
    /// existing theme, clipping and contrast policy still applies, including
    /// protection for unknown inks and reversed cells. The caller chooses a
    /// region containing ordinary chrome: a semantic surface using this same
    /// base color must be kept outside that region.
    pub fn apply_matching(&self, area: Rect, buf: &mut Buffer, theme: &Theme, ground: Color) {
        self.apply_grounds(area, buf, theme, &[Some(ground)]);
    }

    fn apply_grounds(
        &self,
        area: Rect,
        buf: &mut Buffer,
        theme: &Theme,
        grounds: &[Option<Color>],
    ) {
        let Some(ramp) = OceanRamp::for_theme(theme) else {
            return;
        };
        let viewport = self.viewport.unwrap_or(area);
        let area = area.intersection(buf.area);
        for y in area.top()..area.bottom() {
            let water = self.color_at_y_with_ramp(y, viewport, ramp);
            // Text runs repeat the same ink on one shared row ground. Reuse
            // its contrast verdict without allocating or caching theme state
            // across frames; custom colors still take the same safety path.
            let mut previous_ink = None;
            for x in area.left()..area.right() {
                let cell = &mut buf[(x, y)];
                if grounds.contains(&Some(cell.bg)) && !cell.modifier.contains(Modifier::REVERSED) {
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
