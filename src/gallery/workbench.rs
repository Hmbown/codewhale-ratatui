//! Pure fixture facts for workspace composition and the summoned workbar.
use super::Entry;
use crate::{
    ContextItem, ContextRibbon, Cost, Paint, PaneHeader, ReceiptValue, Role, State, Theme, Workbar,
    WorkbarItem, WorkspaceFrame,
};
use ratatui::{buffer::Buffer, layout::Rect};

pub(crate) fn readouts() -> Workbar<'static> {
    Workbar::new(vec![
        WorkbarItem::new("TODO", ReceiptValue::text("3 / 5 complete"))
            .detail("2 in progress")
            .shortcut("1"),
        WorkbarItem::new("CONTEXT", ReceiptValue::Tokens(Some(28_400)))
            .detail("of 128k tokens")
            .shortcut("2"),
        WorkbarItem::new("GIT", ReceiptValue::text("4 files changed"))
            .detail("workspace-polish")
            .shortcut("3"),
        WorkbarItem::new("PRICE", ReceiptValue::Cost(Some(Cost::usd_cents(38))))
            .detail("reported this turn")
            .shortcut("4"),
        WorkbarItem::new("PLUGINS", ReceiptValue::Count(Some(4)))
            .detail("enabled by you")
            .shortcut("5"),
    ])
    .selected(2)
    .focused(true)
}

pub(crate) fn context() -> ContextRibbon<'static> {
    ContextRibbon::new(vec![
        ContextItem::new("", "codewhale-ratatui").priority(3),
        ContextItem::new("", "workspace-polish").priority(2),
        ContextItem::new("", "Local / selected model")
            .priority(0)
            .role(Role::Primary),
        ContextItem::new("", "3 attachments").priority(1),
    ])
}
fn frame(area: Rect, buf: &mut Buffer, theme: &Theme) {
    let frame = WorkspaceFrame::new("codewhale-ratatui")
        .branch("workspace-polish")
        .footer("Illustrative workspace / host owns sessions and actions");
    frame.paint(area, buf, theme);
    let areas = frame.areas(area);
    PaneHeader::new("Conversation")
        .meta("Today")
        .paint(areas.main, buf, theme);
    if let Some(side) = areas.side {
        PaneHeader::new("Files & review")
            .meta("4 changed")
            .paint(side, buf, theme);
    }
}
fn pane(area: Rect, buf: &mut Buffer, theme: &Theme) {
    PaneHeader::new("Files & review")
        .meta("4 changed")
        .focused(true)
        .paint(area, buf, theme);
}
fn ribbon(area: Rect, buf: &mut Buffer, theme: &Theme) {
    context().paint(area, buf, theme);
}
fn ribbon_narrow(area: Rect, buf: &mut Buffer, theme: &Theme) {
    context().paint(area, buf, theme);
}
fn workbar(area: Rect, buf: &mut Buffer, theme: &Theme) {
    readouts().paint(area, buf, theme);
}
fn workbar_narrow(area: Rect, buf: &mut Buffer, theme: &Theme) {
    readouts().paint(area, buf, theme);
}
fn workbar_unknown(area: Rect, buf: &mut Buffer, theme: &Theme) {
    Workbar::new(vec![
        WorkbarItem::new("TODO", ReceiptValue::Count(None)).detail("not reported"),
        WorkbarItem::new("PRICE", ReceiptValue::Cost(None)).detail("not reported"),
        WorkbarItem::new("RUN", ReceiptValue::text("needs decision"))
            .state(State::NeedsYou)
            .detail("open exact command"),
    ])
    .paint(area, buf, theme);
}
pub(crate) fn entries() -> Vec<Entry> {
    vec![
        Entry {
            name: "workbench-frame",
            width: 112,
            height: 12,
            draw: frame,
        },
        Entry {
            name: "pane-header",
            width: 48,
            height: 3,
            draw: pane,
        },
        Entry {
            name: "context-ribbon",
            width: 100,
            height: 2,
            draw: ribbon,
        },
        Entry {
            name: "context-ribbon-narrow",
            width: 40,
            height: 2,
            draw: ribbon_narrow,
        },
        Entry {
            name: "workbar",
            width: 112,
            height: 5,
            draw: workbar,
        },
        Entry {
            name: "workbar-narrow",
            width: 40,
            height: 11,
            draw: workbar_narrow,
        },
        Entry {
            name: "workbar-unreported",
            width: 64,
            height: 5,
            draw: workbar_unknown,
        },
    ]
}
