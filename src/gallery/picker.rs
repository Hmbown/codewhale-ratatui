//! Gallery: the mode and status-line pickers, rebuilt from kit parts. They
//! double as usage examples for [`Picker`] inside a [`Panel`].

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::{Line, Span},
    widgets::Widget,
};

use super::{Entry, arrows};
use crate::{
    Depth, KeyHint, KeyHints, Paint, Panel, Picker, PickerItem, PickerState, Role, Theme, centered,
};

/// The engine's `/mode` rows (`AppMode::Agent`, `Plan`, `Operate`; English
/// names from `locales/en.json`), with the hints rewritten verbs first.
fn mode_items() -> Vec<PickerItem> {
    vec![
        PickerItem::new("Work")
            .key('1')
            .detail("Works in this session; asks before edits and commands"),
        PickerItem::new("Plan")
            .key('2')
            .detail("Researches without changing files, then proposes a plan"),
        PickerItem::new("Operate")
            .key('3')
            .detail("Runs parallel agents on your goal and checks their work"),
    ]
}

fn mode_picker(area: Rect, buf: &mut Buffer, theme: &Theme) {
    let items = mode_items();
    let hints = KeyHints::new(vec![
        KeyHint::new(arrows(theme), "move"),
        KeyHint::new("Enter", "select"),
        KeyHint::new("Esc", "cancel"),
    ]);
    let popup = centered(area, 72, items.len() as u16 + 8, 44, 8);
    let inner = Panel::new(Depth::Overlay)
        .title("Mode")
        .focused(true)
        .hints(&hints)
        .draw(popup, buf, theme);
    Picker::new(&items, PickerState::new(0)).paint(inner, buf, theme);
}

/// The engine's `/statusline` rows (`StatusItem::all()`, labels and hints
/// from `crates/tui/src/config.rs`), with the default footer checked.
fn status_items() -> Vec<PickerItem> {
    let row = |label: &'static str, on: bool, detail: &'static str| {
        PickerItem::new(label).checked(on).detail(detail)
    };
    vec![
        row("Mode", true, "Work, Plan or Operate"),
        row("Model", true, "The model the next message goes to"),
        row(
            "Context window %",
            true,
            "Tokens used of the model's context window",
        ),
        row("Session cost", true, "Running total for this session"),
        row(
            "Account balance",
            false,
            "Prepaid credit left with the active provider",
        ),
        row(
            "Prompt cache hit rate",
            true,
            "Share of the prompt served from cache",
        ),
        row(
            "Output tokens",
            true,
            "Output tokens of the live or last turn",
        ),
        row(
            "Time to first token",
            true,
            "Average wait for the first token",
        ),
        row("Output rate", true, "Average tokens per second"),
        row("Workspace", false, "The directory this session writes to"),
        PickerItem::new("Git branch")
            .checked(false)
            .disabled("Not a git repository"),
    ]
}

fn status_picker(area: Rect, buf: &mut Buffer, theme: &Theme) {
    let items = status_items();
    let shown = items.iter().filter(|i| i.checked == Some(true)).count();
    let hints = KeyHints::new(vec![
        KeyHint::new("Space", "toggle"),
        KeyHint::new("a", "all"),
        KeyHint::new("n", "none"),
        KeyHint::new("Enter", "save"),
        KeyHint::new("Esc", "cancel"),
    ]);
    let popup = centered(area, 72, 18, 40, 8);
    let aside = format!("{shown} of {} shown", items.len());
    let inner = Panel::new(Depth::Overlay)
        .title("Status line")
        .aside(aside)
        .hints(&hints)
        .draw(popup, buf, theme);
    Line::from(Span::styled(
        "Choose what the footer shows.",
        theme.fg(Role::Muted),
    ))
    .render(Rect { height: 1, ..inner }, buf);
    let list = Rect {
        y: inner.y + 2,
        height: inner.height.saturating_sub(2),
        ..inner
    };
    let mut state = PickerState::new(9);
    state.scroll_into_view(items.len(), list.height);
    Picker::new(&items, state).paint(list, buf, theme);
}

pub(crate) fn entries() -> Vec<Entry> {
    vec![
        Entry {
            name: "mode-picker",
            width: 80,
            height: 13,
            draw: mode_picker,
        },
        Entry {
            name: "status-picker",
            width: 80,
            height: 20,
            draw: status_picker,
        },
    ]
}
