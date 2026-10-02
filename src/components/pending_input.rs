//! Host-reported pending input above the composer.
//!
//! The host owns the queue, the turn and every mutation. This component owns
//! only what a person can see: caller items with an explicit status word and
//! mark, attached context with an explicit state, a compact queue summary and
//! the action metadata a host may dispatch. The kit never advances a status,
//! so nothing here can claim an item was sent, delivered or verified.

use std::borrow::Cow;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Modifier,
    text::{Line, Span},
};

use crate::{Paint, Role, Theme, glyphs, text};

use super::workbench::row;

/// What the host reports about one pending item. The word and the mark are
/// the only claims; the kit never moves an item between states.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PendingInputStatus {
    /// Typed input captured for a later turn.
    Queued,
    /// A note the host will deliver into the running turn.
    Steering,
    /// The person is editing this item now.
    Editing,
    /// The host held this item.
    Paused,
    /// The host reports delivery is already under way.
    InFlight,
}

const ACTIONS_SEND_EDIT_DROP: [PendingInputAction; 3] = [
    PendingInputAction::SendNow,
    PendingInputAction::Edit,
    PendingInputAction::Drop,
];
const ACTIONS_EDIT_DROP: [PendingInputAction; 2] =
    [PendingInputAction::Edit, PendingInputAction::Drop];
const ACTIONS_SEND_DROP: [PendingInputAction; 2] =
    [PendingInputAction::SendNow, PendingInputAction::Drop];
const ACTIONS_NONE: [PendingInputAction; 0] = [];

impl PendingInputStatus {
    pub const ALL: [Self; 5] = [
        Self::Queued,
        Self::Steering,
        Self::Editing,
        Self::Paused,
        Self::InFlight,
    ];

    /// The word shown for this status.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Steering => "steering",
            Self::Editing => "editing",
            Self::Paused => "paused",
            Self::InFlight => "in flight",
        }
    }

    #[must_use]
    pub const fn role(self) -> Role {
        match self {
            Self::Queued => Role::Primary,
            Self::Steering => Role::Live,
            Self::Editing => Role::Attention,
            Self::Paused => Role::Attention,
            Self::InFlight => Role::Muted,
        }
    }

    /// The mark for this terminal; `theme.ascii()` never widens the line.
    #[must_use]
    pub fn glyph(self, theme: &Theme) -> &'static str {
        let ascii = theme.ascii();
        match self {
            Self::Queued => {
                if ascii {
                    "o"
                } else {
                    glyphs::AVAILABLE
                }
            }
            Self::Steering => {
                if ascii {
                    "+"
                } else {
                    "↳"
                }
            }
            Self::Editing => {
                if ascii {
                    "e"
                } else {
                    "✎"
                }
            }
            Self::Paused => {
                if ascii {
                    "="
                } else {
                    glyphs::PAUSED
                }
            }
            Self::InFlight => {
                if ascii {
                    ">"
                } else {
                    glyphs::CURRENT
                }
            }
        }
    }

    /// The actions a host may dispatch for an item in this status. The kit
    /// emits them as metadata only; it never performs one.
    #[must_use]
    pub const fn actions(self) -> &'static [PendingInputAction] {
        match self {
            Self::Queued | Self::Paused => &ACTIONS_SEND_EDIT_DROP,
            Self::Steering => &ACTIONS_EDIT_DROP,
            Self::Editing => &ACTIONS_SEND_DROP,
            Self::InFlight => &ACTIONS_NONE,
        }
    }
}

/// One action a host may offer beside a pending item. The kit paints the
/// word and hands the choice back; mutation stays with the host.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PendingInputAction {
    SendNow,
    Edit,
    Drop,
}

impl PendingInputAction {
    pub const ALL: [Self; 3] = [Self::SendNow, Self::Edit, Self::Drop];

    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::SendNow => "Send now",
            Self::Edit => "Edit",
            Self::Drop => "Drop",
        }
    }

    #[must_use]
    pub const fn role(self) -> Role {
        match self {
            Self::SendNow => Role::Primary,
            Self::Edit => Role::Foreground,
            Self::Drop => Role::Danger,
        }
    }
}

/// One queued, steering, editing, paused or in-flight item.
///
/// `content` is display text; it may hold newlines and controls, and the
/// component sanitizes and flattens it. `id` is the stable caller key used
/// for selection and for `actions_for`.
#[derive(Clone, Debug)]
pub struct PendingInputItem<'a> {
    pub id: Cow<'a, str>,
    pub label: Cow<'a, str>,
    pub content: Cow<'a, str>,
    pub status: PendingInputStatus,
    pub status_word: Option<Cow<'a, str>>,
}

impl<'a> PendingInputItem<'a> {
    #[must_use]
    pub fn new(
        id: impl Into<Cow<'a, str>>,
        content: impl Into<Cow<'a, str>>,
        status: PendingInputStatus,
    ) -> Self {
        Self {
            id: id.into(),
            label: Cow::Borrowed(""),
            content: content.into(),
            status,
            status_word: None,
        }
    }

    #[must_use]
    pub fn label(mut self, label: impl Into<Cow<'a, str>>) -> Self {
        self.label = label.into();
        self
    }

    /// A caller-localized status word, replacing the default one.
    #[must_use]
    pub fn status_word(mut self, word: impl Into<Cow<'a, str>>) -> Self {
        self.status_word = Some(word.into());
        self
    }

    #[must_use]
    pub fn word(&self) -> &str {
        self.status_word
            .as_deref()
            .unwrap_or_else(|| self.status.word())
    }

    #[must_use]
    pub fn actions(&self) -> &'static [PendingInputAction] {
        self.status.actions()
    }
}

/// What the host reports about one attached context item. `Unconfirmed` is
/// deliberately not `Included`: unconfirmed context is never shown as sent.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ContextPreviewState {
    /// Known to be included with the message.
    Included,
    /// Attached, but not confirmed by the host.
    Unconfirmed,
    /// Attached and offered for removal.
    Removable,
}

impl ContextPreviewState {
    pub const ALL: [Self; 3] = [Self::Included, Self::Unconfirmed, Self::Removable];

    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::Included => "included",
            Self::Unconfirmed => "unconfirmed",
            Self::Removable => "removable",
        }
    }

    #[must_use]
    pub const fn role(self) -> Role {
        match self {
            Self::Included => Role::Live,
            Self::Unconfirmed => Role::Attention,
            Self::Removable => Role::Muted,
        }
    }

    #[must_use]
    pub fn glyph(self, theme: &Theme) -> &'static str {
        let ascii = theme.ascii();
        match self {
            Self::Included => {
                if ascii {
                    "+"
                } else {
                    glyphs::DONE
                }
            }
            Self::Unconfirmed => {
                if ascii {
                    "?"
                } else {
                    glyphs::ATTENTION
                }
            }
            Self::Removable => {
                if ascii {
                    "-"
                } else {
                    glyphs::NEUTRAL
                }
            }
        }
    }
}

/// One attached context item, with its own reported state.
#[derive(Clone, Debug)]
pub struct ContextPreviewItem<'a> {
    pub id: Cow<'a, str>,
    pub label: Cow<'a, str>,
    pub detail: Option<Cow<'a, str>>,
    pub state: ContextPreviewState,
}

impl<'a> ContextPreviewItem<'a> {
    #[must_use]
    pub fn new(
        id: impl Into<Cow<'a, str>>,
        label: impl Into<Cow<'a, str>>,
        state: ContextPreviewState,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            detail: None,
            state,
        }
    }

    #[must_use]
    pub fn detail(mut self, detail: impl Into<Cow<'a, str>>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    #[must_use]
    pub fn word(&self) -> &str {
        self.state.word()
    }
}

struct PaintedRow {
    line: Line<'static>,
    selected: bool,
}

/// Pending items and attached context over caller-owned facts.
///
/// The component holds borrowed items and an optional selected ID. It paints
/// a compact queue summary, the context row, one row per item, and an action
/// hint built from the union of the item actions. At one row of height the
/// summary and the action words share the row so an action stays
/// discoverable. An empty preview asks for zero rows.
#[derive(Clone, Debug)]
pub struct PendingInputPreview<'a> {
    pub items: Vec<PendingInputItem<'a>>,
    pub context: Vec<ContextPreviewItem<'a>>,
    pub selected: Option<Cow<'a, str>>,
    pub context_label: Cow<'a, str>,
    pub hint: Cow<'a, str>,
    pub omitted_label: Cow<'a, str>,
    pub max_rows: u16,
}

impl<'a> PendingInputPreview<'a> {
    #[must_use]
    pub fn new(items: Vec<PendingInputItem<'a>>, context: Vec<ContextPreviewItem<'a>>) -> Self {
        Self {
            items,
            context,
            selected: None,
            context_label: Cow::Borrowed("Context"),
            hint: Cow::Borrowed(""),
            omitted_label: Cow::Borrowed("more"),
            max_rows: 8,
        }
    }

    /// Select an item by its stable caller ID.
    #[must_use]
    pub fn selected(mut self, id: impl Into<Cow<'a, str>>) -> Self {
        self.selected = Some(id.into());
        self
    }

    #[must_use]
    pub fn context_label(mut self, label: impl Into<Cow<'a, str>>) -> Self {
        self.context_label = label.into();
        self
    }

    /// Replace the action hint. Empty builds one from the item actions.
    #[must_use]
    pub fn hint(mut self, hint: impl Into<Cow<'a, str>>) -> Self {
        self.hint = hint.into();
        self
    }

    #[must_use]
    pub fn omitted_label(mut self, label: impl Into<Cow<'a, str>>) -> Self {
        self.omitted_label = label.into();
        self
    }

    #[must_use]
    pub fn max_rows(mut self, rows: u16) -> Self {
        self.max_rows = rows;
        self
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty() && self.context.is_empty()
    }

    #[must_use]
    pub fn pending_count(&self) -> usize {
        self.items.len()
    }

    #[must_use]
    pub fn selected_index(&self) -> Option<usize> {
        let selected = self.selected.as_deref()?;
        self.items.iter().position(|item| &*item.id == selected)
    }

    /// The union of the actions the items report, in `Send now`, `Edit`,
    /// `Drop` order. Metadata only; the host dispatches.
    #[must_use]
    pub fn actions(&self) -> Vec<PendingInputAction> {
        PendingInputAction::ALL
            .into_iter()
            .filter(|action| {
                self.items
                    .iter()
                    .any(|item| item.actions().contains(action))
            })
            .collect()
    }

    /// The actions for one caller ID, or `None` when no item has that ID.
    #[must_use]
    pub fn actions_for(&self, id: &str) -> Option<&'static [PendingInputAction]> {
        self.items
            .iter()
            .find(|item| &*item.id == id)
            .map(PendingInputItem::actions)
    }

    /// Rows this preview wants at `width`. Empty input asks for zero.
    #[must_use]
    pub fn height(&self, width: u16, theme: &Theme) -> u16 {
        if width < 4 {
            return 0;
        }
        u16::try_from(self.needed_rows(theme))
            .unwrap_or(u16::MAX)
            .min(self.max_rows)
    }

    fn needed_rows(&self, theme: &Theme) -> usize {
        if self.is_empty() {
            return 0;
        }
        1 + self.body_count() + usize::from(!self.hint_text(theme).is_empty())
    }

    fn body_count(&self) -> usize {
        self.context.len() + self.items.len()
    }

    fn separator(theme: &Theme) -> &'static str {
        if theme.ascii() { " / " } else { " · " }
    }

    fn hint_text(&self, _theme: &Theme) -> String {
        if !self.hint.is_empty() {
            return text::display_safe(&self.hint).into_owned();
        }
        self.actions()
            .iter()
            .map(|action| action.word())
            .collect::<Vec<_>>()
            .join(" / ")
    }

    fn summary_text(&self, theme: &Theme) -> String {
        let mut parts = Vec::new();
        for status in PendingInputStatus::ALL {
            let count = self
                .items
                .iter()
                .filter(|item| item.status == status)
                .count();
            if count > 0 {
                parts.push(format!("{count} {}", status.word()));
            }
        }
        if parts.is_empty() && !self.context.is_empty() {
            parts.push(format!(
                "{} {}",
                self.context.len(),
                text::display_safe(&self.context_label)
            ));
        }
        parts.join(Self::separator(theme))
    }

    fn summary_line(&self, theme: &Theme) -> Line<'static> {
        let mut spans = Vec::new();
        let separator = Self::separator(theme);
        let mut first = true;
        for status in PendingInputStatus::ALL {
            let count = self
                .items
                .iter()
                .filter(|item| item.status == status)
                .count();
            if count == 0 {
                continue;
            }
            if !first {
                spans.push(Span::styled(separator, theme.fg(Role::BorderStrong)));
            }
            spans.push(Span::styled(
                format!("{count} {}", status.word()),
                theme.fg(status.role()),
            ));
            first = false;
        }
        if first {
            spans.push(Span::styled(
                format!(
                    "{} {}",
                    self.context.len(),
                    text::display_safe(&self.context_label)
                ),
                theme.fg(Role::Muted),
            ));
        }
        Line::from(spans)
    }

    fn context_line(&self, item: &ContextPreviewItem<'_>, theme: &Theme) -> Line<'static> {
        let mut spans = vec![
            Span::styled(item.state.glyph(theme), theme.fg(item.state.role())),
            Span::raw(" "),
            Span::styled(item.state.word(), theme.fg(item.state.role())),
            Span::raw(" / "),
            Span::styled(
                text::display_safe(&item.label).into_owned(),
                theme.fg(Role::Foreground),
            ),
        ];
        if let Some(detail) = &item.detail {
            spans.push(Span::styled(
                format!(" ({})", text::display_safe(detail)),
                theme.fg(Role::Muted),
            ));
        }
        Line::from(spans)
    }

    fn item_line(&self, index: usize, theme: &Theme) -> Line<'static> {
        let item = &self.items[index];
        let selected = self.selected.as_deref() == Some(&*item.id);
        let marker = if selected {
            if theme.ascii() { "> " } else { "▸ " }
        } else {
            "  "
        };
        let mut spans = vec![
            Span::styled(marker, theme.fg(Role::Primary)),
            Span::styled(item.status.glyph(theme), theme.fg(item.status.role())),
            Span::raw(" "),
            Span::styled(
                text::display_safe(item.word()).into_owned(),
                theme.fg(item.status.role()).add_modifier(Modifier::BOLD),
            ),
            Span::styled(": ", theme.fg(Role::BorderStrong)),
        ];
        if !item.label.is_empty() {
            spans.push(Span::styled(
                format!("{} ", text::display_safe(&item.label)),
                theme.fg(Role::Muted),
            ));
        }
        spans.push(Span::styled(
            one_line(&item.content),
            theme.fg(Role::Foreground),
        ));
        if selected {
            let actions = item.actions();
            if !actions.is_empty() {
                spans.push(Span::raw("  "));
                for (position, action) in actions.iter().enumerate() {
                    if position > 0 {
                        spans.push(Span::styled(" / ", theme.fg(Role::BorderStrong)));
                    }
                    spans.push(Span::styled(action.word(), theme.fg(action.role())));
                }
            }
        }
        Line::from(spans)
    }

    fn hint_line(&self, theme: &Theme, hidden: usize) -> Option<Line<'static>> {
        let hint = self.hint_text(theme);
        if hint.is_empty() {
            return None;
        }
        let mut spans = Vec::new();
        if hidden > 0 {
            spans.push(Span::styled(
                format!("+{hidden} {}  ", text::display_safe(&self.omitted_label)),
                theme.fg(Role::Muted),
            ));
        }
        spans.push(Span::styled(hint, theme.fg(Role::Primary)));
        Some(Line::from(spans))
    }

    fn single_line(&self, width: usize, theme: &Theme) -> Line<'static> {
        let ascii = theme.ascii();
        let hint = self.hint_text(theme);
        let summary = self.summary_text(theme);
        let separator = Self::separator(theme);
        let shown = if hint.is_empty() {
            text::truncate_words(&summary, width, ascii).into_owned()
        } else {
            let full = format!("{summary}{separator}{hint}");
            let short = format!(
                "{} pending{separator}{hint}",
                self.items.len() + self.context.len()
            );
            if text::width(&full) <= width {
                full
            } else if text::width(&short) <= width {
                short
            } else {
                text::truncate_words(&hint, width, ascii).into_owned()
            }
        };
        Line::from(Span::styled(shown, theme.fg(Role::Foreground)))
    }

    fn rows(&self, width: u16, height: u16, theme: &Theme) -> Vec<PaintedRow> {
        if width < 4 || height == 0 || self.is_empty() {
            return Vec::new();
        }
        let hint = self.hint_text(theme);
        if height == 1 {
            return vec![PaintedRow {
                line: self.single_line(usize::from(width), theme),
                selected: false,
            }];
        }
        let mut out = vec![PaintedRow {
            line: self.summary_line(theme),
            selected: false,
        }];
        let hint_rows = usize::from(!hint.is_empty());
        let body_budget = usize::from(height).saturating_sub(1 + hint_rows);
        let mut body: Vec<PaintedRow> = Vec::new();
        let selected = self.selected_index();
        if let Some(index) = selected {
            body.push(PaintedRow {
                line: self.item_line(index, theme),
                selected: true,
            });
        }
        for item in &self.context {
            body.push(PaintedRow {
                line: self.context_line(item, theme),
                selected: false,
            });
        }
        for (index, item) in self.items.iter().enumerate() {
            if selected == Some(index) {
                continue;
            }
            body.push(PaintedRow {
                line: self.item_line(index, theme),
                selected: self.selected.as_deref() == Some(&*item.id),
            });
        }
        let hidden = body.len().saturating_sub(body_budget);
        out.extend(body.into_iter().take(body_budget));
        if let Some(line) = self.hint_line(theme, hidden) {
            out.push(PaintedRow {
                line,
                selected: false,
            });
        }
        out
    }
}

fn one_line(value: &str) -> String {
    text::display_safe(&value.replace(['\n', '\r', '\t'], " ")).into_owned()
}

impl Paint for PendingInputPreview<'_> {
    fn paint(&self, area: Rect, buf: &mut Buffer, theme: &Theme) {
        let area = area.intersection(buf.area);
        if area.is_empty() {
            return;
        }
        let rows = self.rows(area.width, area.height.min(self.max_rows), theme);
        for (index, painted) in rows.iter().enumerate() {
            let rect = Rect::new(
                area.x,
                area.y
                    .saturating_add(u16::try_from(index).unwrap_or(u16::MAX)),
                area.width,
                1,
            );
            if painted.selected {
                buf.set_style(rect, theme.bg(Role::Selected));
            }
            row(rect, buf, &painted.line);
        }
    }

    fn height(&self, width: u16, theme: &Theme) -> u16 {
        if width < 4 {
            return 0;
        }
        u16::try_from(self.needed_rows(theme))
            .unwrap_or(u16::MAX)
            .min(self.max_rows)
    }
}
