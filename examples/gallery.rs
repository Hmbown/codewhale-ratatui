//! Browse every component in every terminal profile.
//!
//!   cargo run --example gallery                         # browse interactively
//!   cargo run --example gallery -- --print dark-256     # print every component once
//!   cargo run --example gallery -- --dump out/          # write every component x profile
//!
//! Browsing: ↑↓ choose a component, p / Shift+P change the profile, q quit.
//! `--print` writes ANSI to stdout, so you can see a profile in any terminal
//! without raw mode. `--dump` writes `<component>.<profile>.ans` (view with
//! `cat`) and `.txt` (glyphs plus the roles each run was painted with).
//!
//! Profiles: dark-truecolor, light-truecolor, dark-256, light-256, ansi-16,
//! unknown-ground, no-color, ascii.

use std::io::{self, Write as _};
use std::path::Path;

use codewhale_ratatui::{
    Depth, KeyHint, KeyHints, Paint, Panel, Picker, PickerItem, PickerState, Role, Theme, gallery,
    testing::{self, Profile},
};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::Rect,
    text::{Line, Span},
    widgets::Widget,
};

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("--dump") => dump(Path::new(args.get(1).map_or("gallery-out", String::as_str))),
        Some("--print") => print(args.get(1).map(String::as_str)),
        Some("--help" | "-h") => {
            println!("usage: gallery [--print [profile] | --dump [dir]]");
            Ok(())
        }
        _ => browse(),
    }
}

fn profile_arg(name: Option<&str>) -> io::Result<Vec<Profile>> {
    match name {
        None => Ok(Profile::ALL.to_vec()),
        Some(name) => Profile::from_name(name).map(|p| vec![p]).ok_or_else(|| {
            let names: Vec<_> = Profile::ALL.iter().map(|p| p.name()).collect();
            io::Error::other(format!(
                "unknown profile {name}; one of {}",
                names.join(", ")
            ))
        }),
    }
}

fn print(profile: Option<&str>) -> io::Result<()> {
    let mut out = io::stdout().lock();
    for profile in profile_arg(profile)? {
        let theme = profile.theme();
        for entry in gallery::entries() {
            writeln!(out, "\n{} · {}", entry.name, profile.name())?;
            out.write_all(testing::ansi(&gallery::render(&entry, &theme)).as_bytes())?;
        }
    }
    Ok(())
}

fn dump(dir: &Path) -> io::Result<()> {
    std::fs::create_dir_all(dir)?;
    let mut written = 0;
    for profile in Profile::ALL {
        let theme = profile.theme();
        for entry in gallery::entries() {
            let buf = gallery::render(&entry, &theme);
            let stem = format!("{}.{}", entry.name, profile.name());
            std::fs::write(dir.join(format!("{stem}.ans")), testing::ansi(&buf))?;
            std::fs::write(
                dir.join(format!("{stem}.txt")),
                testing::styled(&buf, &theme),
            )?;
            written += 2;
        }
    }
    println!("Wrote {written} files to {}", dir.display());
    Ok(())
}

fn browse() -> io::Result<()> {
    enable_raw_mode()?;
    // Measure the real ground before painting, as a host would.
    codewhale_ratatui::detect::probe_terminal_background();
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;
    let result = run(&mut terminal);
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    result
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    let entries = gallery::entries();
    let items: Vec<PickerItem> = entries.iter().map(|e| PickerItem::new(e.name)).collect();
    let mut state = PickerState::new(0);
    // `None` is this terminal as detected; otherwise a forced profile.
    let mut profile: Option<usize> = None;
    loop {
        let theme = profile.map_or_else(Theme::detect, |i| Profile::ALL[i].theme());
        let profile_name = profile.map_or("this terminal", |i| Profile::ALL[i].name());
        terminal.draw(|frame| {
            let area = frame.area();
            let buf = frame.buffer_mut();
            let hints = KeyHints::new(vec![
                KeyHint::new(if theme.ascii() { "Up/Down" } else { "↑↓" }, "choose"),
                KeyHint::new("p", "next profile"),
                KeyHint::new("q", "quit"),
            ]);
            let inner = Panel::new(Depth::Stage)
                .title("Codewhale components")
                .aside(profile_name)
                .hints(&hints)
                .draw(area, buf, &theme);
            let rail = Rect {
                width: 20.min(inner.width),
                ..inner
            };
            let rail_inner = Panel::new(Depth::Deep).draw(rail, buf, &theme);
            state.scroll_into_view(items.len(), rail_inner.height);
            Picker::new(&items, state).paint(rail_inner, buf, &theme);

            let stage = Rect {
                x: rail.right() + 2,
                width: inner.width.saturating_sub(rail.width + 2),
                ..inner
            };
            let entry = &entries[state.selected];
            Line::from(Span::styled(
                format!("{} · {}x{}", entry.name, entry.width, entry.height),
                theme.fg(Role::Muted),
            ))
            .render(Rect { height: 1, ..stage }, buf);
            let canvas = Rect {
                y: stage.y + 2,
                width: entry.width.min(stage.width),
                height: entry.height.min(stage.height.saturating_sub(2)),
                ..stage
            };
            (entry.draw)(canvas, buf, &theme);
        })?;
        if let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                KeyCode::Up | KeyCode::Char('k') => state.prev(items.len()),
                KeyCode::Down | KeyCode::Char('j') => state.next(items.len()),
                KeyCode::Char('p') if !key.modifiers.contains(KeyModifiers::SHIFT) => {
                    profile = match profile {
                        None => Some(0),
                        Some(i) if i + 1 < Profile::ALL.len() => Some(i + 1),
                        Some(_) => None,
                    };
                }
                KeyCode::Char('P') | KeyCode::Char('p') => {
                    profile = match profile {
                        None => Some(Profile::ALL.len() - 1),
                        Some(0) => None,
                        Some(i) => Some(i - 1),
                    };
                }
                _ => {}
            }
        }
    }
}
