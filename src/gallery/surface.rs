//! Gallery: depth (panels) and the horizon rule.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::{Line, Span},
    widgets::Widget,
};

use super::Entry;
use crate::{Depth, HorizonRule, Paint, Panel, Role, Theme, glyphs};

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

pub(crate) fn entries() -> Vec<Entry> {
    vec![
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
    ]
}
