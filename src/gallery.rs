//! Every component with fixture data, for `examples/gallery.rs` and the
//! snapshot tests. The fixtures double as usage examples: the mode and
//! status-line pickers here are the engine's `/mode` and `/statusline`
//! pickers rebuilt from kit parts.

use std::time::Duration;

use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::{Line, Span},
    widgets::Widget,
};

use crate::{
    Depth, HorizonRule, Icon, KeyHint, KeyHints, MotionMode, Paint, Panel, Picker, PickerItem,
    PickerState, Role, Spinner, State, StatusMark, Theme, Toast, Toasts, Whale, WhaleState,
    centered, glyphs,
    keys::{Platform, pair_label},
};

/// One gallery entry.
pub struct Entry {
    pub name: &'static str,
    pub width: u16,
    pub height: u16,
    pub draw: fn(Rect, &mut Buffer, &Theme),
}

fn arrows(theme: &Theme) -> String {
    pair_label(KeyCode::Up, KeyCode::Down, Platform::current(theme.ascii()))
}

fn key_hints(area: Rect, buf: &mut Buffer, theme: &Theme) {
    let hints = KeyHints::new(vec![
        KeyHint::new(arrows(theme), "move"),
        KeyHint::new("Enter", "change"),
        KeyHint::new("r", "reset"),
        KeyHint::new("/", "search"),
        KeyHint::new("Ctrl+S", "save").disabled(),
        KeyHint::new("Esc", "close"),
    ]);
    hints.paint(area, buf, theme);
}

fn status_marks(area: Rect, buf: &mut Buffer, theme: &Theme) {
    for (row, state) in State::ALL.iter().enumerate() {
        let rect = Rect {
            y: area.y + row as u16,
            height: 1,
            ..area
        };
        StatusMark::new(*state).paint(rect, buf, theme);
    }
}

fn depths(area: Rect, buf: &mut Buffer, theme: &Theme) {
    let w = area.width / 4;
    for (i, (depth, title, body)) in [
        (Depth::Deep, "Deep", "Put away"),
        (Depth::Stage, "Stage", "Where work sits"),
        (Depth::Raised, "Raised", "A card"),
        (Depth::Overlay, "Overlay", "A decision"),
    ]
    .into_iter()
    .enumerate()
    {
        let rect = Rect {
            x: area.x + w * i as u16,
            width: w,
            ..area
        };
        let inner = Panel::new(depth).title(title).draw(rect, buf, theme);
        Line::from(Span::styled(body, theme.fg(Role::Muted))).render(inner, buf);
    }
}

fn horizon(area: Rect, buf: &mut Buffer, theme: &Theme) {
    Line::from(Span::styled(
        "Edited summary.md",
        theme.fg(Role::Foreground),
    ))
    .render(Rect { height: 1, ..area }, buf);
    HorizonRule::new().aside("12% of context used").paint(
        Rect {
            y: area.y + 1,
            height: 1,
            ..area
        },
        buf,
        theme,
    );
    Line::from(vec![
        Span::styled(
            format!("{} ", glyphs::pick("›", theme.ascii())),
            theme.fg(Role::Primary),
        ),
        Span::styled("Ask Codewhale to do something", theme.fg(Role::Muted)),
    ])
    .render(
        Rect {
            y: area.y + 2,
            height: 1,
            ..area
        },
        buf,
    );
}

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

fn toasts(area: Rect, buf: &mut Buffer, theme: &Theme) {
    Toasts::new(vec![
        Toast::new(State::Done, "Theme set to Shoreline"),
        Toast::new(State::NeedsYou, "A command is waiting for your approval").opens(),
        Toast::new(
            State::Failed,
            "Could not save: the settings file is read-only",
        )
        .opens(),
    ])
    .paint(area, buf, theme);
}

fn icons(area: Rect, buf: &mut Buffer, theme: &Theme) {
    let rows: [(Icon, &str); 4] = [
        (
            Icon::Sonar {
                elapsed: Duration::from_millis(600),
                motion: MotionMode::Full,
            },
            "Searching the codebase",
        ),
        (Icon::tide(3, 5).expect("known total"), "3 of 5 phases done"),
        (Icon::Shell, "Receipt: edited summary.md"),
        (Icon::Kelp, "Divider"),
    ];
    for (row, (icon, words)) in rows.iter().enumerate() {
        Line::from(vec![
            icon.span(theme),
            Span::raw(" "),
            Span::styled(*words, theme.fg(Role::Foreground)),
            Span::styled(format!("  ({})", icon.label()), theme.fg(Role::Muted)),
        ])
        .render(
            Rect {
                y: area.y + row as u16,
                height: 1,
                ..area
            },
            buf,
        );
    }
}

fn spinners(area: Rect, buf: &mut Buffer, theme: &Theme) {
    for (row, (elapsed, motion)) in [
        (Duration::from_millis(200), MotionMode::Full),
        (Duration::from_millis(1400), MotionMode::Full),
        (Duration::from_secs(246), MotionMode::Reduced),
    ]
    .into_iter()
    .enumerate()
    {
        Spinner::new("Running cargo test", elapsed, motion).paint(
            Rect {
                y: area.y + row as u16,
                height: 1,
                ..area
            },
            buf,
            theme,
        );
    }
}

fn whale(state: WhaleState) -> impl Fn(Rect, &mut Buffer, &Theme) {
    move |area, buf, theme| Whale::new(state).paint(area, buf, theme)
}

/// All 17 actions of the v2 whale at the compact size, three to a row.
fn whale_actions(area: Rect, buf: &mut Buffer, theme: &Theme) {
    const CELL_W: u16 = 28;
    const CELL_H: u16 = 11;
    for (i, state) in WhaleState::ALL.into_iter().enumerate() {
        let i = u16::try_from(i).unwrap_or(u16::MAX);
        let cell = Rect {
            x: area.x + (i % 3) * CELL_W,
            y: area.y + (i / 3) * CELL_H,
            width: CELL_W,
            height: CELL_H,
        }
        .intersection(area);
        Whale::new(state).paint(cell, buf, theme);
    }
}

/// Every entry, in gallery order.
#[must_use]
pub fn entries() -> Vec<Entry> {
    vec![
        Entry {
            name: "key-hints",
            width: 60,
            height: 2,
            draw: key_hints,
        },
        Entry {
            name: "status-marks",
            width: 20,
            height: 7,
            draw: status_marks,
        },
        Entry {
            name: "depth",
            width: 80,
            height: 6,
            draw: depths,
        },
        Entry {
            name: "horizon",
            width: 60,
            height: 3,
            draw: horizon,
        },
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
        Entry {
            name: "toasts",
            width: 60,
            height: 3,
            draw: toasts,
        },
        Entry {
            name: "icons",
            width: 50,
            height: 4,
            draw: icons,
        },
        Entry {
            name: "spinner",
            width: 40,
            height: 3,
            draw: spinners,
        },
        Entry {
            name: "whale-rest",
            width: 36,
            height: 17,
            draw: |a, b, t| whale(WhaleState::Rest)(a, b, t),
        },
        Entry {
            name: "whale-busy",
            width: 36,
            height: 17,
            draw: |a, b, t| whale(WhaleState::Busy)(a, b, t),
        },
        Entry {
            name: "whale-needs",
            width: 36,
            height: 17,
            draw: |a, b, t| whale(WhaleState::NeedsYou)(a, b, t),
        },
        Entry {
            name: "whale-done",
            width: 36,
            height: 17,
            draw: |a, b, t| whale(WhaleState::Done)(a, b, t),
        },
        Entry {
            name: "whale-pod-1",
            width: 36,
            height: 17,
            draw: |a, b, t| whale(WhaleState::Pod { calves: 1 })(a, b, t),
        },
        Entry {
            name: "whale-pod-3",
            width: 36,
            height: 17,
            draw: |a, b, t| whale(WhaleState::Pod { calves: 3 })(a, b, t),
        },
        Entry {
            name: "whale-actions",
            width: 84,
            height: 66,
            draw: whale_actions,
        },
        Entry {
            name: "whale-compact",
            width: 24,
            height: 11,
            draw: |a, b, t| whale(WhaleState::Busy)(a, b, t),
        },
        Entry {
            name: "whale-words-only",
            width: 14,
            height: 3,
            draw: |a, b, t| whale(WhaleState::NeedsYou)(a, b, t),
        },
    ]
}

/// Render one entry for one theme.
#[must_use]
pub fn render(entry: &Entry, theme: &Theme) -> Buffer {
    crate::testing::render(entry.width, entry.height, |area, buf| {
        buf.set_style(area, theme.bg(Role::Background));
        (entry.draw)(area, buf, theme);
    })
}
