//! Portable extraction of the mounted Codewhale TUI composer and its
//! borderless workflow progress rows. Source: `crates/tui/src/tui/widgets/
//! workbar.rs`, `widgets/mod.rs`, `composer_chrome.rs`, and `ui/frame.rs`.
//! Caller data replaces App and WorkflowPanel; geometry, density, columns,
//! failure bars, prompt and send chrome retain their native grammar.
//!
//! Adapted from Codewhale, licensed under the MIT License:
//! Copyright (c) 2024-2025 DeepSeek-TUI Contributors
//!
//! Permission is hereby granted, free of charge, to any person obtaining a
//! copy of this software and associated documentation files (the "Software"),
//! to deal in the Software without restriction, including without limitation
//! the rights to use, copy, modify, merge, publish, distribute, sublicense,
//! and/or sell copies of the Software, and to permit persons to whom the
//! Software is furnished to do so, subject to the following conditions:
//!
//! The above copyright notice and this permission notice shall be included
//! in all copies or substantial portions of the Software.
//!
//! THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
//! IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
//! FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL
//! THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
//! LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
//! FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
//! DEALINGS IN THE SOFTWARE.

use std::{borrow::Cow, time::Duration};

use ratatui::{
    buffer::Buffer,
    layout::{Position, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
};
use unicode_segmentation::UnicodeSegmentation;

use crate::{Paint, Role, Theme, TuiGround, TuiInk, glyphs, text};

const BAR_CELLS: usize = 20;
const MAX_RUN_ROWS: usize = 6;
const GAP: &str = "  ";

/// The workflow lifecycle reported by the host, without inferred outcomes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkflowRunState {
    Pending,
    Running,
    Succeeded,
    Degraded,
    Failed,
    Cancelled,
}
impl WorkflowRunState {
    fn mark(self, ascii: bool) -> &'static str {
        glyphs::pick(
            match self {
                Self::Pending => "○",
                Self::Running => "•",
                Self::Succeeded => "✓",
                Self::Degraded => "◆",
                Self::Failed => "✕",
                Self::Cancelled => "⊘",
            },
            ascii,
        )
    }
    fn ink(self, theme: &Theme) -> Style {
        match self {
            Self::Pending | Self::Cancelled => theme.fg(Role::Muted),
            Self::Running => theme.tui_ink(TuiInk::Working),
            Self::Succeeded => theme.tui_ink(TuiInk::Success),
            Self::Degraded => theme.tui_ink(TuiInk::Warning),
            Self::Failed => theme.fg(Role::Danger),
        }
    }
}

/// Host-reported row outcomes and facts for one run. Successful and failed
/// counts stay separate; cancellation never contributes to the success bar.
#[derive(Clone, Debug)]
pub struct WorkflowRun<'a> {
    pub title: Cow<'a, str>,
    pub state: WorkflowRunState,
    pub succeeded: usize,
    pub failed: usize,
    pub cancelled: usize,
    pub total: usize,
    pub elapsed: Option<Duration>,
    pub tokens: Option<u64>,
    pub queued: usize,
    pub reason: Option<Cow<'a, str>>,
}
impl<'a> WorkflowRun<'a> {
    pub fn new(title: impl Into<Cow<'a, str>>, state: WorkflowRunState) -> Self {
        Self {
            title: title.into(),
            state,
            succeeded: 0,
            failed: 0,
            cancelled: 0,
            total: 0,
            elapsed: None,
            tokens: None,
            queued: 0,
            reason: None,
        }
    }
    pub const fn outcomes(
        mut self,
        succeeded: usize,
        failed: usize,
        cancelled: usize,
        total: usize,
    ) -> Self {
        self.succeeded = succeeded;
        self.failed = failed;
        self.cancelled = cancelled;
        self.total = total;
        self
    }
    pub const fn elapsed(mut self, elapsed: Duration) -> Self {
        self.elapsed = Some(elapsed);
        self
    }
    pub const fn tokens(mut self, tokens: u64) -> Self {
        self.tokens = Some(tokens);
        self
    }
    pub const fn queued(mut self, queued: usize) -> Self {
        self.queued = queued;
        self
    }
    pub fn reason(mut self, reason: impl Into<Cow<'a, str>>) -> Self {
        self.reason = Some(reason.into());
        self
    }
}

/// Localized words replace the native English defaults without changing the
/// row grammar. Count fields also accept templates: `done` may contain
/// `{done}` and `{total}`; `failed`, `cancelled`, `queued` and `more` may
/// contain `{count}`. Templates preserve a language's word and count order.
/// A host may change the shortcut shown beside folded runs.
#[derive(Clone, Debug)]
pub struct WorkflowProgressWords {
    pub done: Cow<'static, str>,
    pub failed: Cow<'static, str>,
    pub cancelled: Cow<'static, str>,
    pub queued: Cow<'static, str>,
    pub no_tasks: Cow<'static, str>,
    pub large: Cow<'static, str>,
    pub gaps: Cow<'static, str>,
    pub stopped: Cow<'static, str>,
    pub more: Cow<'static, str>,
    pub manage: Cow<'static, str>,
}
impl Default for WorkflowProgressWords {
    fn default() -> Self {
        Self {
            done: "done".into(),
            failed: "failed".into(),
            cancelled: "cancelled".into(),
            queued: "queued".into(),
            no_tasks: "No tasks yet".into(),
            large: "Large workflow".into(),
            gaps: "finished with gaps".into(),
            stopped: "stopped".into(),
            more: "more".into(),
            manage: "to manage".into(),
        }
    }
}

/// The native workbar: borderless progress, one row per workflow, directly
/// below the posture bar and above metrics in the mounted TUI compositor.
#[derive(Clone, Debug)]
pub struct WorkflowProgress<'a> {
    pub runs: Vec<WorkflowRun<'a>>,
    pub words: WorkflowProgressWords,
}
impl<'a> WorkflowProgress<'a> {
    pub fn new(runs: Vec<WorkflowRun<'a>>) -> Self {
        Self {
            runs,
            words: WorkflowProgressWords::default(),
        }
    }
    pub fn words(mut self, words: WorkflowProgressWords) -> Self {
        self.words = words;
        self
    }
    pub fn desired_rows(&self) -> u16 {
        Self::desired_rows_for(self.runs.len())
    }
    /// Reserve the native row budget before constructing the visible runs.
    pub const fn desired_rows_for(runs: usize) -> u16 {
        if runs > MAX_RUN_ROWS {
            (MAX_RUN_ROWS + 1) as u16
        } else {
            runs as u16
        }
    }
    /// Shared columns align visible runs. Narrow widths shed bar, tokens,
    /// tail reservation and elapsed in the same order as the native workbar.
    pub fn lines(&self, width: u16, max_rows: usize, theme: &Theme) -> Vec<Line<'static>> {
        let width = usize::from(width);
        let max_rows = max_rows.min(usize::from(self.desired_rows()));
        if self.runs.is_empty() || max_rows == 0 || width < 8 {
            return Vec::new();
        }
        let shown = if self.runs.len() > max_rows {
            max_rows.saturating_sub(1)
        } else {
            self.runs.len()
        };
        let cells: Vec<_> = self.runs[..shown]
            .iter()
            .map(|run| RunCells::new(run, &self.words, theme.ascii()))
            .collect();
        let layout = ProgressLayout::fit(&cells, width.saturating_sub(1), theme.ascii());
        let mut rows: Vec<_> = cells
            .iter()
            .map(|row| row.line(&layout, width, theme))
            .collect();
        let hidden = self.runs.len() - shown;
        if hidden > 0 {
            let more = if self.words.more.contains("{count}") {
                safe(&self.words.more).replace("{count}", &hidden.to_string())
            } else {
                format!("+{hidden} {}", safe(&self.words.more))
            };
            let line = format!(
                " {more} {} {} {}",
                glyphs::pick("·", theme.ascii()),
                glyphs::pick("↓", theme.ascii()),
                safe(&self.words.manage)
            );
            rows.push(Line::styled(plain_clip(&line, width), theme.fg(Role::Hint)));
        }
        rows
    }
}
impl Paint for WorkflowProgress<'_> {
    fn paint(&self, area: Rect, buf: &mut Buffer, theme: &Theme) {
        let area = area.intersection(buf.area);
        for (index, line) in self
            .lines(area.width, usize::from(area.height), theme)
            .into_iter()
            .enumerate()
        {
            buf.set_line(area.x, area.y + index as u16, &line, area.width);
        }
    }
    fn height(&self, _width: u16, _theme: &Theme) -> u16 {
        self.desired_rows()
    }
}

fn safe(value: &str) -> String {
    text::display_safe(value).into_owned()
}
fn counted(value: &str, count: usize) -> String {
    let value = safe(value);
    if value.contains("{count}") {
        value.replace("{count}", &count.to_string())
    } else {
        format!("{count} {value}")
    }
}
fn sentence(value: &str) -> String {
    let flat = value
        .split_whitespace()
        .map(safe)
        .collect::<Vec<_>>()
        .join(" ");
    let mut end = flat.len();
    let mut chars = flat.char_indices().peekable();
    while let Some((index, ch)) = chars.next() {
        if ch == ';'
            || (matches!(ch, '.' | '!' | '?') && chars.peek().is_some_and(|(_, c)| *c == ' '))
        {
            end = if matches!(ch, '.' | ';') {
                index
            } else {
                index + ch.len_utf8()
            };
            break;
        }
    }
    flat[..end].trim_end_matches(['.', ' ']).trim().to_owned()
}
fn reason(value: &str) -> String {
    let value = value.trim_start();
    let value = value
        .strip_prefix('[')
        .and_then(|v| v.split_once("] "))
        .filter(|(tag, _)| {
            !tag.is_empty()
                && tag.len() <= 24
                && tag
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        })
        .map_or(value, |(_, rest)| rest);
    sentence(value)
}
fn elapsed(value: Duration) -> String {
    if value.is_zero() {
        "0s".into()
    } else if value < Duration::from_secs(1) {
        format!("{}ms", value.as_millis())
    } else if value.as_secs() < 60 {
        format!("{}s", value.as_secs())
    } else {
        format!("{}m {:02}s", value.as_secs() / 60, value.as_secs() % 60)
    }
}
fn tokens(value: u64) -> String {
    if value >= 1_000_000 {
        format!("{:.1}M", value as f64 / 1_000_000.0)
    } else if value >= 1_000 {
        format!("{:.1}k", value as f64 / 1_000.0)
    } else {
        value.to_string()
    }
}
fn bar_cells(succeeded: usize, failed: usize, total: usize) -> (usize, usize) {
    if total == 0 {
        return (0, 0);
    }
    let cells = |n: usize| {
        let count = n as u128;
        let total = total as u128;
        let result =
            ((count * BAR_CELLS as u128 + total / 2) / total).min(BAR_CELLS as u128) as usize;
        if n > 0 { result.max(1) } else { 0 }
    };
    let failed = cells(failed);
    (cells(succeeded).min(BAR_CELLS - failed), failed)
}

struct RunCells {
    state: WorkflowRunState,
    name: String,
    done_cells: usize,
    failed_cells: usize,
    done: String,
    problems: String,
    elapsed: String,
    tokens: Option<String>,
    chips: Vec<String>,
    reason: Option<String>,
}
impl RunCells {
    fn new(run: &WorkflowRun<'_>, words: &WorkflowProgressWords, ascii: bool) -> Self {
        let sep = if ascii { " . " } else { " · " };
        let done = if run.total > 0 {
            if words.done.contains("{done}") || words.done.contains("{total}") {
                safe(&words.done)
                    .replace("{done}", &run.succeeded.to_string())
                    .replace("{total}", &run.total.to_string())
            } else {
                format!("{}/{} {}", run.succeeded, run.total, safe(&words.done))
            }
        } else if run.failed == 0 {
            safe(&words.no_tasks)
        } else {
            String::new()
        };
        let mut problems = Vec::new();
        if run.failed > 0 {
            problems.push(counted(&words.failed, run.failed));
        }
        if run.cancelled > 0 {
            problems.push(counted(&words.cancelled, run.cancelled));
        }
        let problems = if problems.is_empty() {
            String::new()
        } else {
            format!(
                "{}{text}",
                if done.is_empty() { "" } else { sep },
                text = problems.join(sep)
            )
        };
        let (done_cells, failed_cells) = bar_cells(run.succeeded, run.failed, run.total);
        let mut chips = Vec::new();
        if run.total >= 25 {
            chips.push(format!(
                "{} {}",
                if ascii { "!" } else { "⚠" },
                safe(&words.large)
            ));
        }
        if run.queued > 0 {
            chips.push(format!(
                "{} {}",
                glyphs::pick("·", ascii),
                counted(&words.queued, run.queued)
            ));
        }
        let why = run.reason.as_deref().map(reason).filter(|s| !s.is_empty());
        let reason = match run.state {
            WorkflowRunState::Failed => Some(why.unwrap_or_else(|| {
                if words.failed.contains("{count}") {
                    counted(&words.failed, run.failed)
                } else {
                    safe(&words.failed)
                }
            })),
            WorkflowRunState::Degraded => Some(why.map_or_else(
                || safe(&words.gaps),
                |why| format!("{}{sep}{why}", safe(&words.gaps)),
            )),
            WorkflowRunState::Cancelled => Some(safe(&words.stopped)),
            _ => None,
        };
        let name = sentence(&run.title);
        Self {
            state: run.state,
            name: text::truncate_words(if name.is_empty() { "workflow" } else { &name }, 40, ascii)
                .into_owned(),
            done_cells,
            failed_cells,
            done,
            problems,
            elapsed: run.elapsed.map(elapsed).unwrap_or_default(),
            tokens: run
                .tokens
                .map(|n| format!("{}{n}", glyphs::pick("↓", ascii), n = tokens(n))),
            chips,
            reason,
        }
    }
    fn progress_width(&self) -> usize {
        text::width(&self.done) + text::width(&self.problems)
    }
    fn tail_width(&self) -> usize {
        self.chips.iter().map(|s| text::width(s) + 1).sum::<usize>()
            + self.reason.as_deref().map_or(0, |s| text::width(s) + 1)
    }
    fn line(&self, layout: &ProgressLayout, width: usize, theme: &Theme) -> Line<'static> {
        let mut spans = vec![
            Span::raw(" "),
            Span::styled(
                format!("{} ", self.state.mark(theme.ascii())),
                self.state.ink(theme),
            ),
            Span::styled(
                text::pad(
                    &text::truncate_words(&self.name, layout.name_room, theme.ascii()),
                    layout.name_cols,
                    theme.ascii(),
                ),
                theme.fg(Role::Foreground).add_modifier(Modifier::BOLD),
            ),
        ];
        if layout.bar {
            spans.extend([
                Span::raw(GAP),
                Span::styled(
                    glyphs::pick("█", theme.ascii()).repeat(self.done_cells),
                    theme.tui_ink(TuiInk::Success),
                ),
                Span::styled(
                    glyphs::pick("×", theme.ascii()).repeat(self.failed_cells),
                    theme.fg(Role::Danger),
                ),
                Span::styled(
                    glyphs::pick("░", theme.ascii())
                        .repeat(BAR_CELLS - self.done_cells - self.failed_cells),
                    theme.fg(Role::Hint),
                ),
            ]);
        }
        spans.extend([
            Span::raw(GAP),
            Span::styled(self.done.clone(), theme.fg(Role::Muted)),
            Span::styled(self.problems.clone(), theme.fg(Role::Danger)),
            Span::raw(" ".repeat(layout.progress_cols.saturating_sub(self.progress_width()))),
        ]);
        if layout.elapsed_cols > 0 {
            spans.extend([
                Span::raw(GAP),
                Span::styled(
                    text::pad(&self.elapsed, layout.elapsed_cols, theme.ascii()),
                    theme.fg(Role::Muted),
                ),
            ]);
        }
        if layout.tokens_cols > 0 {
            spans.extend([
                Span::raw(GAP),
                Span::styled(
                    text::pad(
                        self.tokens.as_deref().unwrap_or(""),
                        layout.tokens_cols,
                        theme.ascii(),
                    ),
                    theme.fg(Role::Muted),
                ),
            ]);
        }
        let mut used = spans.iter().map(|s| text::width(&s.content)).sum::<usize>();
        for (index, chip) in self.chips.iter().enumerate() {
            let gap = if index == 0 { GAP } else { " " };
            spans.extend([
                Span::raw(gap),
                Span::styled(chip.clone(), theme.tui_ink(TuiInk::Warning)),
            ]);
            used += gap.len() + text::width(chip);
        }
        if let Some(reason) = &self.reason {
            let gap = if self.chips.is_empty() { GAP } else { " " };
            let room = width.saturating_sub(used + gap.len());
            if room >= 12.min(text::width(reason)) {
                spans.extend([
                    Span::raw(gap),
                    Span::styled(
                        text::truncate_words(reason, room, theme.ascii()).into_owned(),
                        self.state.ink(theme),
                    ),
                ]);
            }
        }
        clip(spans, width)
    }
}
struct ProgressLayout {
    name_room: usize,
    name_cols: usize,
    bar: bool,
    progress_cols: usize,
    elapsed_cols: usize,
    tokens_cols: usize,
}
impl ProgressLayout {
    fn fit(rows: &[RunCells], width: usize, ascii: bool) -> Self {
        let widest = |f: &dyn Fn(&RunCells) -> usize| rows.iter().map(f).max().unwrap_or(0);
        let name_want = widest(&|r| text::width(&r.name));
        let mut tail_want = widest(&RunCells::tail_width).min(24);
        let mut layout = Self {
            name_room: 0,
            name_cols: 0,
            bar: width >= 72,
            progress_cols: widest(&RunCells::progress_width),
            elapsed_cols: widest(&|r| text::width(&r.elapsed)),
            tokens_cols: widest(&|r| r.tokens.as_deref().map_or(0, text::width)),
        };
        let room = loop {
            let fixed = 2
                + if layout.bar { BAR_CELLS + GAP.len() } else { 0 }
                + GAP.len()
                + layout.progress_cols
                + if layout.elapsed_cols > 0 {
                    GAP.len() + layout.elapsed_cols
                } else {
                    0
                }
                + if layout.tokens_cols > 0 {
                    GAP.len() + layout.tokens_cols
                } else {
                    0
                };
            let room = width.saturating_sub(fixed + tail_want);
            if room >= 12.min(name_want) {
                break name_want.min(room.max(12.min(name_want)));
            }
            if layout.bar {
                layout.bar = false;
            } else if layout.tokens_cols > 0 {
                layout.tokens_cols = 0;
            } else if tail_want > 0 {
                tail_want = 0;
            } else if layout.elapsed_cols > 0 {
                layout.elapsed_cols = 0;
            } else {
                break width.saturating_sub(fixed).max(1).min(name_want.max(1));
            }
        };
        layout.name_room = room;
        layout.name_cols = widest(&|r| text::width(&text::truncate_words(&r.name, room, ascii)));
        layout
    }
}
fn clip(spans: Vec<Span<'static>>, width: usize) -> Line<'static> {
    let mut used = 0;
    let mut output = Vec::new();
    for span in spans {
        let cells = text::width(&span.content);
        if used + cells <= width {
            used += cells;
            output.push(span);
        } else {
            let room = width.saturating_sub(used);
            if room > 0 {
                output.push(Span::styled(plain_clip(&span.content, room), span.style));
            }
            break;
        }
    }
    Line::from(output)
}
fn plain_clip(value: &str, width: usize) -> String {
    let mut cells = 0;
    value
        .graphemes(true)
        .take_while(|g| {
            let next = text::width(g);
            if cells + next > width {
                false
            } else {
                cells += next;
                true
            }
        })
        .collect()
}

/// The mounted composer's content floor and native total-row cap.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NativeComposerDensity {
    Compact,
    #[default]
    Comfortable,
    Spacious,
}
impl NativeComposerDensity {
    pub const fn max_rows(self) -> u16 {
        match self {
            Self::Compact => 7,
            Self::Comfortable => 9,
            Self::Spacious => 12,
        }
    }
    const fn floor(self) -> usize {
        match self {
            Self::Compact => 1,
            Self::Comfortable => 2,
            Self::Spacious => 3,
        }
    }
}

/// Shared render and hit-test geometry. Text never claims the submit target
/// or its breathing cell. A compact/quiet enclosure has no submit rectangle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativeComposerGeometry {
    pub inner: Rect,
    pub text: Rect,
    pub submit: Option<Rect>,
    pub prompt_x: Option<u16>,
}

/// The actual mounted native composer chrome, over caller-owned text.
/// Editing, submit dispatch, menus and input-method state belong to the host.
#[derive(Clone, Debug)]
pub struct NativeComposer<'a> {
    pub text: Cow<'a, str>,
    pub placeholder: Cow<'a, str>,
    pub focused: bool,
    pub enclosed: bool,
    pub density: NativeComposerDensity,
    pub can_submit: bool,
    pub submit_hint: Option<Cow<'a, str>>,
    pub target: Option<Cow<'a, str>>,
    /// Cursor index in graphemes of the displayed text, defaulting to its end.
    pub cursor: Option<usize>,
    pub menu_rows: usize,
}
impl<'a> NativeComposer<'a> {
    pub fn new(text: impl Into<Cow<'a, str>>) -> Self {
        Self {
            text: text.into(),
            placeholder: "Write a task or use /.".into(),
            focused: false,
            enclosed: true,
            density: NativeComposerDensity::Comfortable,
            can_submit: false,
            submit_hint: None,
            target: None,
            cursor: None,
            menu_rows: 0,
        }
    }
    pub const fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }
    pub const fn enclosed(mut self, enclosed: bool) -> Self {
        self.enclosed = enclosed;
        self
    }
    pub const fn density(mut self, density: NativeComposerDensity) -> Self {
        self.density = density;
        self
    }
    pub const fn can_submit(mut self, can_submit: bool) -> Self {
        self.can_submit = can_submit;
        self
    }
    pub const fn cursor(mut self, grapheme: usize) -> Self {
        self.cursor = Some(grapheme);
        self
    }
    pub const fn menu_rows(mut self, rows: usize) -> Self {
        self.menu_rows = rows;
        self
    }
    pub fn placeholder(mut self, placeholder: impl Into<Cow<'a, str>>) -> Self {
        self.placeholder = placeholder.into();
        self
    }
    pub fn submit_hint(mut self, hint: impl Into<Cow<'a, str>>) -> Self {
        self.submit_hint = Some(hint.into());
        self
    }
    pub fn target(mut self, target: impl Into<Cow<'a, str>>) -> Self {
        self.target = Some(target.into());
        self
    }
    pub fn has_panel(&self, area: Rect) -> bool {
        self.enclosed && area.width >= 12 && area.height >= 3
    }
    pub fn geometry(&self, area: Rect) -> NativeComposerGeometry {
        let panel = self.has_panel(area);
        let (inner, submit) = if panel {
            let submit = Rect::new(
                area.x.saturating_add(area.width.saturating_sub(5)),
                area.y.saturating_add(area.height.saturating_sub(2)),
                3,
                1,
            );
            let x = area.x.saturating_add(1);
            (
                Rect::new(
                    x,
                    area.y.saturating_add(1),
                    submit.x.saturating_sub(1).saturating_sub(x),
                    area.height.saturating_sub(2),
                ),
                Some(submit),
            )
        } else if area.height >= 2 {
            (
                Rect::new(
                    area.x,
                    area.y.saturating_add(1),
                    area.width,
                    area.height - 1,
                ),
                None,
            )
        } else {
            (area, None)
        };
        let inset = if inner.width >= 3 { 2 } else { 0 };
        NativeComposerGeometry {
            inner,
            text: Rect::new(
                inner.x.saturating_add(inset),
                inner.y,
                inner.width.saturating_sub(inset),
                inner.height,
            ),
            submit,
            prompt_x: (inset > 0).then_some(inner.x),
        }
    }
    pub fn desired_height(&self, width: u16, available_height: u16) -> u16 {
        let available = available_height.max(1);
        let panel = self.enclosed && width >= 12 && available >= 3;
        let measure = self.geometry(Rect::new(0, 0, width, if panel { 3 } else { 1 }));
        let content = input_rows(&self.text, usize::from(measure.text.width.max(1)))
            .len()
            .max(self.density.floor());
        let border = if panel {
            2
        } else {
            usize::from(available >= 2)
        };
        content
            .saturating_add(self.menu_rows)
            .saturating_add(border)
            .clamp(1, usize::from(available.min(self.density.max_rows())))
            .try_into()
            .unwrap_or(1)
    }
    /// Cursor geometry uses the same wrapped content, scroll and native
    /// padding as painting. The terminal host owns showing the actual caret.
    pub fn cursor_position(&self, area: Rect) -> Option<Position> {
        if !self.focused {
            return None;
        }
        let (geometry, _, row, column, top, padding) = self.layout(area);
        if geometry.text.width == 0
            || row < top
            || row - top + padding >= usize::from(geometry.text.height)
        {
            return None;
        }
        let x = geometry.text.x.saturating_add(u16::try_from(column).ok()?);
        let y = geometry
            .text
            .y
            .saturating_add(u16::try_from(row - top + padding).ok()?);
        geometry
            .text
            .contains(Position::new(x, y))
            .then_some(Position::new(x, y))
    }
    fn layout(
        &self,
        area: Rect,
    ) -> (
        NativeComposerGeometry,
        Vec<InputRow>,
        usize,
        usize,
        usize,
        usize,
    ) {
        let geometry = self.geometry(area);
        let rows = input_rows(&self.text, usize::from(geometry.text.width.max(1)));
        let cursor = self
            .cursor
            .unwrap_or_else(|| multiline_safe(&self.text).graphemes(true).count());
        let row = rows
            .iter()
            .rposition(|line| cursor >= line.start)
            .unwrap_or(0);
        let column = text::width(
            &rows[row]
                .text
                .graphemes(true)
                .take(cursor.saturating_sub(rows[row].start))
                .collect::<String>(),
        );
        let budget = usize::from(geometry.inner.height)
            .saturating_sub(self.menu_rows)
            .max(1);
        let top = (row + 1).saturating_sub(budget);
        let visible = rows.len().saturating_sub(top).min(budget).max(1);
        let padding = budget.saturating_sub(visible) / 2;
        (geometry, rows, row, column, top, padding)
    }
}
impl Paint for NativeComposer<'_> {
    fn paint(&self, area: Rect, buf: &mut Buffer, theme: &Theme) {
        let area = area.intersection(buf.area);
        if area.is_empty() {
            return;
        }
        let background = theme.tui_ground(TuiGround::Composer);
        buf.set_style(area, background);
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                buf[(x, y)].set_symbol(" ");
            }
        }
        let panel = self.has_panel(area);
        let border = if panel && self.focused {
            Role::Primary
        } else {
            Role::Border
        };
        let border_style = background.patch(theme.fg(border));
        if area.height >= 2 {
            let line = Line::styled(
                glyphs::pick("─", theme.ascii()).repeat(usize::from(area.width)),
                border_style,
            );
            buf.set_line(area.x, area.y, &line, area.width);
            if panel {
                buf.set_line(area.x, area.bottom() - 1, &line, area.width);
                for y in area.y + 1..area.bottom() - 1 {
                    buf[(area.x, y)]
                        .set_symbol(glyphs::pick("│", theme.ascii()))
                        .set_style(border_style);
                    buf[(area.right() - 1, y)]
                        .set_symbol(glyphs::pick("│", theme.ascii()))
                        .set_style(border_style);
                }
                for (x, y, mark) in [
                    (area.x, area.y, "╭"),
                    (area.right() - 1, area.y, "╮"),
                    (area.x, area.bottom() - 1, "╰"),
                    (area.right() - 1, area.bottom() - 1, "╯"),
                ] {
                    buf[(x, y)]
                        .set_symbol(glyphs::pick(mark, theme.ascii()))
                        .set_style(border_style);
                }
            }
            if let Some(hint) = &self.submit_hint
                && !self.text.trim().is_empty()
            {
                let y = if panel { area.bottom() - 1 } else { area.y };
                let label = format!(" {} ", safe(hint));
                buf.set_stringn(
                    area.x.saturating_add(u16::from(panel)),
                    y,
                    text::truncate_words(
                        &label,
                        usize::from(area.width.saturating_sub(u16::from(panel) * 2)),
                        theme.ascii(),
                    )
                    .as_ref(),
                    usize::from(area.width.saturating_sub(u16::from(panel) * 2)),
                    background.patch(theme.fg(Role::Primary)),
                );
            }
            if let Some(target) = &self.target {
                let label = format!(" {} ", safe(target));
                let room = usize::from(area.width.saturating_sub(2));
                let label = text::truncate_words(&label, room, theme.ascii());
                let cells = u16::try_from(text::width(&label)).unwrap_or(0);
                buf.set_stringn(
                    area.right().saturating_sub(cells + 1),
                    area.y,
                    &label,
                    usize::from(cells),
                    background
                        .patch(theme.fg(Role::Attention))
                        .add_modifier(Modifier::BOLD),
                );
            }
        }
        let (geometry, rows, cursor_row, _, top, padding) = self.layout(area);
        let budget = usize::from(geometry.inner.height)
            .saturating_sub(self.menu_rows)
            .max(1);
        let placeholder = self.text.is_empty();
        let hint_rows = if placeholder {
            input_rows(&self.placeholder, usize::from(geometry.text.width.max(1)))
        } else {
            Vec::new()
        };
        let visible: Vec<_> = if placeholder {
            hint_rows.iter().collect()
        } else {
            rows.iter().skip(top).collect()
        };
        for (index, line) in visible
            .into_iter()
            .take(budget.saturating_sub(padding))
            .enumerate()
        {
            let offset = padding + index;
            if offset >= usize::from(geometry.text.height) {
                break;
            }
            let y = geometry.text.y + offset as u16;
            buf.set_stringn(
                geometry.text.x,
                y,
                &line.text,
                usize::from(geometry.text.width),
                background.patch(if placeholder {
                    theme.tui_ink(TuiInk::Soft)
                } else {
                    theme.fg(Role::Foreground)
                }),
            );
        }
        if let Some(x) = geometry.prompt_x
            && cursor_row >= top
        {
            let offset = cursor_row - top + padding;
            if offset < usize::from(geometry.inner.height) {
                buf[(x, geometry.inner.y + offset as u16)]
                    .set_symbol(glyphs::pick("❯", theme.ascii()))
                    .set_style(background.patch(theme.fg(Role::Primary)));
            }
        }
        if let Some(submit) = geometry.submit {
            let mark = match (self.can_submit, theme.ascii()) {
                (true, false) => "[↵]",
                (true, true) => "[>]",
                (false, false) => "[·]",
                (false, true) => "[.]",
            };
            let style = background.patch(if self.can_submit {
                theme.tui_ink(TuiInk::Info)
            } else {
                theme.fg(Role::Dim)
            });
            buf.set_stringn(
                submit.x,
                submit.y,
                mark,
                3,
                if self.can_submit {
                    style.add_modifier(Modifier::BOLD)
                } else {
                    style
                },
            );
        }
    }
    fn height(&self, width: u16, _theme: &Theme) -> u16 {
        self.desired_height(width, self.density.max_rows())
    }
}

struct InputRow {
    text: String,
    start: usize,
}
fn multiline_safe(value: &str) -> String {
    value.split('\n').map(safe).collect::<Vec<_>>().join("\n")
}
fn input_rows(value: &str, width: usize) -> Vec<InputRow> {
    let value = multiline_safe(value);
    let mut rows = Vec::new();
    let mut start = 0;
    for raw in value.split('\n') {
        let mut current = String::new();
        let mut cells = 0;
        let mut break_at = None;
        let mut output = Vec::new();
        for g in raw.graphemes(true) {
            let w = text::width(g);
            if cells + w > width && cells != 0 {
                flush_input(&mut current, &mut cells, &mut break_at, &mut output);
            }
            current.push_str(g);
            cells += w;
            if g == " " && !current.trim_start().is_empty() {
                break_at = Some(current.len());
            }
            if cells >= width {
                flush_input(&mut current, &mut cells, &mut break_at, &mut output);
            }
        }
        output.push(current);
        for line in output {
            let count = line.graphemes(true).count();
            rows.push(InputRow { text: line, start });
            start += count;
        }
        start += 1;
    }
    rows
}
fn flush_input(
    current: &mut String,
    cells: &mut usize,
    break_at: &mut Option<usize>,
    rows: &mut Vec<String>,
) {
    match break_at.take() {
        Some(byte) if byte < current.len() => {
            let remainder = current.split_off(byte);
            rows.push(std::mem::replace(current, remainder));
            *cells = text::width(current);
        }
        _ => {
            rows.push(std::mem::take(current));
            *cells = 0;
        }
    }
}
