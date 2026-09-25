//! The whale, drawn in Braille dots.
//!
//! The C-shaped whale from the Codewhale mark, as the v2 pet
//! (`whale-character-v2`), is the one character on every surface. The desktop
//! app draws it with the v2 rig; the terminal draws the same contours as
//! Braille dots. Its states carry meaning: resting, working, needs you, done,
//! and a pod, where one to three calves swim with it while agents work in
//! parallel. The count is never invented.
//!
//! `assets/whale-v2.scenes` holds the poster pose of each state as exact
//! cubic contours, exported from the v2 kit's Director by
//! `tools/export-whale.cjs`. [`rasterize`] is a port of the kit's pure
//! vector-to-dot renderer (`braille.js`): flatten each cubic into ten
//! segments, fill scanlines by even-odd crossings at dot centres (so the
//! throat and eye holes stay open), and pack each 2x4 block into one
//! `U+2800` cell. It reproduces the kit's 32x16 and 20x10 stills exactly
//! (`tests/whale.rs`).
//!
//! The art is decoration; the state lives in the words beside it. Screen
//! readers get the label, never a stream of dot names, and ASCII-safe
//! terminals get the words alone.

use std::borrow::Cow;
use std::sync::OnceLock;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::Widget,
};

use crate::{
    Paint, Role, State, StatusMark, Theme,
    color::{ColorDepth, blend, rgb, rgb_to_ansi256},
};

/// Dot bits per cell, row-major: `01 08`, `02 10`, `04 20`, `40 80`.
pub const BITS: [[u8; 2]; 4] = [[0x01, 0x08], [0x02, 0x10], [0x04, 0x20], [0x40, 0x80]];

/// The logo's ombre stops (`#1E8FD8` to `#0B48BB`), from the design
/// direction. Only the whale wears them.
const LOGO_TOP: u32 = 0x1e8fd8;
const LOGO_BOTTOM: u32 = 0x0b48bb;

/// What the whale is doing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WhaleState {
    Rest,
    Busy,
    NeedsYou,
    Done,
    /// Working with agents in parallel. Calves are clamped to 1..=3.
    Pod {
        calves: u8,
    },
}

impl WhaleState {
    /// The key and calf count in `assets/whale-v2.scenes`.
    fn scene_key(self) -> (&'static str, u8) {
        match self {
            WhaleState::Rest => ("rest", 0),
            WhaleState::Busy => ("busy", 0),
            WhaleState::NeedsYou => ("needs", 0),
            WhaleState::Done => ("done", 0),
            WhaleState::Pod { calves } => ("pod", calves.clamp(1, 3)),
        }
    }

    /// The status this pose stands for.
    #[must_use]
    pub const fn state(self) -> State {
        match self {
            WhaleState::Rest => State::Ready,
            WhaleState::Busy | WhaleState::Pod { .. } => State::Working,
            WhaleState::NeedsYou => State::NeedsYou,
            WhaleState::Done => State::Done,
        }
    }

    /// The English words beside the art.
    #[must_use]
    pub fn words(self) -> Cow<'static, str> {
        match self {
            WhaleState::Rest => "Resting".into(),
            WhaleState::Busy => "Working".into(),
            WhaleState::NeedsYou => "Needs you".into(),
            WhaleState::Done => "Done".into(),
            WhaleState::Pod { calves } => {
                let n = calves.clamp(1, 3);
                format!("Working with {n} agent{}", if n == 1 { "" } else { "s" }).into()
            }
        }
    }
}

/// A closed contour: a start point and cubic segments `(c1, c2, end)`.
#[derive(Clone, Debug, PartialEq)]
pub struct Contour {
    pub start: [f64; 2],
    pub cubics: Vec<[f64; 6]>,
}

/// One filled shape and its compound holes, filled even-odd together.
#[derive(Clone, Debug, PartialEq)]
pub struct Shape {
    pub id: String,
    pub role: String,
    pub contours: Vec<Contour>,
}

/// A posed whale at one optical size.
#[derive(Clone, Debug, PartialEq)]
pub struct Scene {
    pub state: String,
    pub calves: u8,
    pub size: u32,
    pub shapes: Vec<Shape>,
}

const SCENES_SOURCE: &str = include_str!("../assets/whale-v2.scenes");

/// Every exported scene, parsed once.
///
/// # Panics
/// Only if the checked-in asset is malformed, which `tests/whale.rs` rules
/// out.
pub fn scenes() -> &'static [Scene] {
    static SCENES: OnceLock<Vec<Scene>> = OnceLock::new();
    SCENES.get_or_init(|| parse_scenes(SCENES_SOURCE).expect("whale-v2.scenes parses"))
}

fn parse_scenes(src: &str) -> Result<Vec<Scene>, String> {
    let mut scenes: Vec<Scene> = Vec::new();
    for (n, line) in src.lines().enumerate() {
        let err = |what: &str| format!("line {}: {what}", n + 1);
        let mut words = line.split_ascii_whitespace();
        match words.next() {
            None | Some("#") => {}
            Some(w) if w.starts_with('#') => {}
            Some("scene") => {
                let state = words.next().ok_or_else(|| err("scene state"))?.to_string();
                let calves = words
                    .next()
                    .and_then(|w| w.parse().ok())
                    .ok_or_else(|| err("calves"))?;
                let size = words
                    .next()
                    .and_then(|w| w.parse().ok())
                    .ok_or_else(|| err("size"))?;
                scenes.push(Scene {
                    state,
                    calves,
                    size,
                    shapes: Vec::new(),
                });
            }
            Some("shape") => {
                let scene = scenes.last_mut().ok_or_else(|| err("shape before scene"))?;
                scene.shapes.push(Shape {
                    id: words.next().ok_or_else(|| err("shape id"))?.to_string(),
                    role: words.next().ok_or_else(|| err("shape role"))?.to_string(),
                    contours: Vec::new(),
                });
            }
            Some("path") => {
                let shape = scenes
                    .last_mut()
                    .and_then(|s| s.shapes.last_mut())
                    .ok_or_else(|| err("path before shape"))?;
                let nums: Vec<f64> = words
                    .map(|w| w.parse::<f64>().map_err(|_| err("number")))
                    .collect::<Result<_, _>>()?;
                if nums.len() < 2 || !(nums.len() - 2).is_multiple_of(6) {
                    return Err(err("path length"));
                }
                shape.contours.push(Contour {
                    start: [nums[0], nums[1]],
                    cubics: nums[2..].as_chunks::<6>().0.to_vec(),
                });
            }
            Some(other) => return Err(err(&format!("unknown record {other}"))),
        }
    }
    Ok(scenes)
}

/// The scene for `state` whose optical size is nearest `size`.
#[must_use]
pub fn scene(state: WhaleState, size: u32) -> Option<&'static Scene> {
    let (key, calves) = state.scene_key();
    scenes()
        .iter()
        .filter(|s| s.state == key && s.calves == calves)
        .min_by_key(|s| s.size.abs_diff(size))
}

/// Packed Braille cells, row-major; `0` is an empty cell.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Grid {
    pub cols: u16,
    pub rows: u16,
    pub cells: Vec<u8>,
}

impl Grid {
    #[must_use]
    pub fn cell(&self, col: u16, row: u16) -> u8 {
        self.cells[usize::from(row) * usize::from(self.cols) + usize::from(col)]
    }

    /// The cell as text: `U+2800 + bits`, or a space.
    #[must_use]
    pub fn char_at(&self, col: u16, row: u16) -> char {
        match self.cell(col, row) {
            0 => ' ',
            bits => char::from_u32(0x2800 + u32::from(bits)).unwrap_or(' '),
        }
    }

    /// Rows of text, joined by newlines, as the kit's stills are written.
    #[must_use]
    pub fn text(&self) -> String {
        (0..self.rows)
            .map(|r| {
                (0..self.cols)
                    .map(|c| self.char_at(c, r))
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Flatten one contour into straight edges, exactly as `braille.js` does:
/// ten segments per cubic, then close back to the start.
fn edges(contour: &Contour, out: &mut Vec<[f64; 4]>) {
    let mut pen = contour.start;
    for c in &contour.cubics {
        let p = pen;
        for i in 1..=10 {
            let t = f64::from(i) / 10.0;
            let u = 1.0 - t;
            let q = [
                u * u * u * p[0]
                    + 3.0 * u * u * t * c[0]
                    + 3.0 * u * t * t * c[2]
                    + t * t * t * c[4],
                u * u * u * p[1]
                    + 3.0 * u * u * t * c[1]
                    + 3.0 * u * t * t * c[3]
                    + t * t * t * c[5],
            ];
            out.push([pen[0], pen[1], q[0], q[1]]);
            pen = q;
        }
    }
    out.push([pen[0], pen[1], contour.start[0], contour.start[1]]);
}

/// Rasterize a scene into `cols` x `rows` Braille cells. `cell_aspect` is a
/// cell's width over its height (0.5 for a typical monospace font).
#[must_use]
pub fn rasterize(scene: &Scene, cols: u16, rows: u16, cell_aspect: f64) -> Grid {
    let width = usize::from(cols) * 2;
    let height = usize::from(rows) * 4;
    let (w, h) = (width as f64, height as f64);
    let mut dots = vec![false; width * height];
    let scale = (w * cell_aspect * 2.0).min(h) / 124.0;
    let sx = scale / (cell_aspect * 2.0);
    let sy = scale;
    let mut shape_edges = Vec::new();
    let mut cross = Vec::new();
    for shape in &scene.shapes {
        shape_edges.clear();
        for contour in &shape.contours {
            edges(contour, &mut shape_edges);
        }
        for e in &mut shape_edges {
            *e = [
                w / 2.0 + e[0] * sx,
                h / 2.0 + e[1] * sy,
                w / 2.0 + e[2] * sx,
                h / 2.0 + e[3] * sy,
            ];
        }
        for y in 0..height {
            let cy = y as f64 + 0.5;
            cross.clear();
            for &[x1, y1, x2, y2] in &shape_edges {
                if (y1 > cy) != (y2 > cy) {
                    cross.push(x1 + (cy - y1) * (x2 - x1) / (y2 - y1));
                }
            }
            cross.sort_by(f64::total_cmp);
            for pair in cross.as_chunks::<2>().0 {
                let left = (pair[0] - 0.5).ceil().max(0.0);
                let right = (pair[1] - 0.5).ceil().min(w);
                if right <= left {
                    continue;
                }
                for x in left as usize..right as usize {
                    dots[y * width + x] = true;
                }
            }
        }
    }
    let mut cells = vec![0u8; usize::from(cols) * usize::from(rows)];
    for y in 0..height {
        for x in 0..width {
            if dots[y * width + x] {
                cells[(y / 4) * usize::from(cols) + x / 2] |= BITS[y % 4][x % 2];
            }
        }
    }
    Grid { cols, rows, cells }
}

/// The whale for `state` in a `cols` x `rows` viewport, as the kit's `frame`
/// sizes it, or `None` below 16x8 cells, where only the words fit.
#[must_use]
pub fn frame(state: WhaleState, cols: u16, rows: u16) -> Option<Grid> {
    if cols < 16 || rows < 8 {
        return None;
    }
    let size = u32::from(cols * 2).min(u32::from(rows) * 4);
    Some(rasterize(scene(state, size)?, cols, rows, 0.5))
}

/// The whale widget: the art, centred, with its words on the row below.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Whale {
    pub state: WhaleState,
    /// Localized words; `None` uses [`WhaleState::words`].
    pub words: Option<Cow<'static, str>>,
}

impl Whale {
    #[must_use]
    pub fn new(state: WhaleState) -> Self {
        Self { state, words: None }
    }

    #[must_use]
    pub fn words(mut self, words: impl Into<Cow<'static, str>>) -> Self {
        self.words = Some(words.into());
        self
    }

    /// The largest standard viewport (32x16, 20x10, 16x8) that fits.
    fn viewport(area: Rect) -> Option<(u16, u16)> {
        let rows = area.height.saturating_sub(1);
        [(32, 16), (20, 10), (16, 8)]
            .into_iter()
            .find(|&(c, r)| area.width >= c && rows >= r)
    }

    /// Ink for one row of art: the logo ombre in truecolor, `Primary`
    /// elsewhere, the terminal's own ink without color.
    fn ink(theme: &Theme, row: u16, rows: u16) -> Style {
        if !theme.paints_grounds() {
            return theme.fg(Role::Primary);
        }
        let (top, bottom) = match theme.caps().appearance {
            crate::detect::Appearance::Light => (rgb(LOGO_TOP), rgb(LOGO_BOTTOM)),
            _ => (theme.token(Role::Primary), rgb(LOGO_TOP)),
        };
        let t = f32::from(row) / f32::from(rows.saturating_sub(1).max(1));
        let c = blend(bottom, top, t);
        match (theme.depth(), c) {
            (ColorDepth::TrueColor, c) => Style::default().fg(c),
            (_, Color::Rgb(r, g, b)) => {
                Style::default().fg(Color::Indexed(rgb_to_ansi256(r, g, b)))
            }
            _ => theme.fg(Role::Primary),
        }
    }
}

impl Paint for Whale {
    fn paint(&self, area: Rect, buf: &mut Buffer, theme: &Theme) {
        if area.is_empty() {
            return;
        }
        let mark = StatusMark::new(self.state.state())
            .word(self.words.clone().unwrap_or_else(|| self.state.words()));
        let label = Line::from(mark.spans(theme)).centered();
        // Braille has no honest ASCII form: dot-count shading turns the whale
        // into noise. ASCII-safe terminals get the words alone.
        let viewport = if theme.ascii() {
            None
        } else {
            Self::viewport(area)
        };
        let Some((cols, rows)) = viewport else {
            label.render(Rect { height: 1, ..area }, buf);
            return;
        };
        let Some(grid) = frame(self.state, cols, rows) else {
            label.render(Rect { height: 1, ..area }, buf);
            return;
        };
        let x0 = area.x + (area.width - cols) / 2;
        let y0 = area.y + (area.height - rows - 1) / 2;
        for r in 0..rows {
            let style = Self::ink(theme, r, rows);
            for c in 0..cols {
                let ch = grid.char_at(c, r);
                if ch == ' ' {
                    continue;
                }
                let mut tmp = [0u8; 4];
                buf[(x0 + c, y0 + r)]
                    .set_symbol(ch.encode_utf8(&mut tmp))
                    .set_style(style);
            }
        }
        label.render(
            Rect {
                y: y0 + rows,
                height: 1,
                ..area
            },
            buf,
        );
    }

    fn height(&self, _width: u16, _theme: &Theme) -> u16 {
        17
    }
}
