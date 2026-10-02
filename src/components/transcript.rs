//! A caller-authored transcript: headings, prose with semantic spans,
//! quotes, lists, tables and inset code.
//!
//! The host owns the parser, the clipboard and the scroll position. This kit
//! owns wrapping, clipping and honesty: a link target is metadata that never
//! reaches the screen, `copy_text` is the exact original source, and a
//! caller's semantic role is painted as the caller asked. No syntax parser,
//! no OSC sequence, no file access.

use std::borrow::Cow;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
};
use unicode_segmentation::UnicodeSegmentation;

use crate::{Paint, Role, Theme, text};

use super::workbench::row;

/// One atom of painted ink: a grapheme and the style it was asked to carry.
type Atom = (String, Style);

fn push_atom(row: &mut Vec<Atom>, grapheme: &str, style: Style) {
    row.push((grapheme.to_string(), style));
}

fn row_width(row: &[Atom]) -> usize {
    row.iter().map(|(text, _)| text::width(text)).sum()
}

/// Split a row at its last whitespace atom: everything before stays, the
/// remainder carries to the next row. Without whitespace the row is kept
/// whole and the caller breaks it hard.
fn split_at_space(row: &[Atom]) -> (Vec<Atom>, Vec<Atom>) {
    let Some(index) = row
        .iter()
        .rposition(|(text, _)| !text.is_empty() && text.chars().all(char::is_whitespace))
    else {
        return (row.to_vec(), Vec::new());
    };
    (row[..index].to_vec(), row[index + 1..].to_vec())
}

/// Word-aware wrapping over grapheme atoms. `first` budgets the first row and
/// `rest` every continuation; a grapheme wider than the budget still lands.
fn wrap_atoms(atoms: &[Atom], first: usize, rest: usize) -> Vec<Vec<Atom>> {
    let mut rows: Vec<Vec<Atom>> = Vec::new();
    let mut row: Vec<Atom> = Vec::new();
    let mut width = 0usize;
    let mut budget = first.max(1);
    for (grapheme, style) in atoms {
        let glyph = text::width(grapheme);
        if width > 0 && width + glyph > budget {
            let (keep, carry) = split_at_space(&row);
            if !keep.is_empty() && !carry.is_empty() {
                rows.push(keep);
                row = carry;
                width = row_width(&row);
            } else {
                rows.push(std::mem::take(&mut row));
                width = 0;
            }
            budget = rest.max(1);
        }
        push_atom(&mut row, grapheme, *style);
        width += glyph;
    }
    if !row.is_empty() {
        rows.push(row);
    }
    if rows.is_empty() {
        rows.push(Vec::new());
    }
    rows
}

fn row_line(row: &[Atom]) -> Line<'static> {
    let mut spans: Vec<Span<'static>> = Vec::new();
    for (text, style) in row {
        if let Some(last) = spans.last_mut().filter(|last| last.style == *style) {
            last.content.to_mut().push_str(text);
        } else {
            spans.push(Span::styled(text.clone(), *style));
        }
    }
    Line::from(spans)
}

fn prefixed(prefix: &str, style: Style, line: Line<'static>) -> Line<'static> {
    let mut spans = Vec::with_capacity(line.spans.len() + 1);
    spans.push(Span::styled(prefix.to_string(), style));
    spans.extend(line.spans);
    Line::from(spans)
}

/// Wrap logical lines behind a prefix. The first visual row carries `prefix`;
/// every later row carries `indent`, which is the same rail for a quote and
/// blank gutter or list padding elsewhere.
fn flow(
    logical: Vec<Vec<Atom>>,
    first: usize,
    rest: usize,
    prefix: &str,
    prefix_style: Style,
    indent: &str,
) -> Vec<Line<'static>> {
    let mut out = Vec::new();
    let mut leading = true;
    for (index, atoms) in logical.into_iter().enumerate() {
        let budget = if index == 0 { first } else { rest };
        for (row_index, atoms) in wrap_atoms(&atoms, budget, rest).into_iter().enumerate() {
            let line = row_line(&atoms);
            if leading && row_index == 0 {
                out.push(prefixed(prefix, prefix_style, line));
                leading = false;
            } else if indent.is_empty() {
                out.push(line);
            } else {
                out.push(prefixed(indent, Style::default(), line));
            }
        }
    }
    out
}

fn wrapped(logical: Vec<Vec<Atom>>, first: usize, rest: usize) -> Vec<Line<'static>> {
    flow(logical, first, rest, "", Style::default(), "")
}

/// Sanitized single-style text, split into logical lines on `\n`.
fn ink_atoms(value: &str, style: Style) -> Vec<Vec<Atom>> {
    // Split structural newlines before filtering hidden controls. The generic
    // single-line sanitizer intentionally removes every control, including LF.
    value
        .split('\n')
        .map(|line| {
            let expanded = line.replace('\t', "    ");
            let safe = text::display_safe(&expanded);
            let mut atoms = Vec::new();
            for grapheme in safe.graphemes(true) {
                push_atom(&mut atoms, grapheme, style);
            }
            atoms
        })
        .collect()
}

/// Sanitized caller spans, split into logical lines on `\n`.
fn span_atoms<'a>(spans: &[TranscriptSpan<'a>], theme: &Theme) -> Vec<Vec<Atom>> {
    let mut lines: Vec<Vec<Atom>> = vec![Vec::new()];
    for span in spans {
        let style = span.style(theme);
        for (index, piece) in span.text.split('\n').enumerate() {
            if index > 0 {
                lines.push(Vec::new());
            }
            let expanded = piece.replace('\t', "    ");
            let safe = text::display_safe(&expanded);
            for grapheme in safe.graphemes(true) {
                push_atom(
                    lines.last_mut().expect("one line is always present"),
                    grapheme,
                    style,
                );
            }
        }
    }
    lines
}

fn paint_lines(area: Rect, buf: &mut Buffer, lines: &[Line<'static>], offset: usize) {
    for (index, line) in lines
        .iter()
        .skip(offset)
        .take(usize::from(area.height))
        .enumerate()
    {
        let y = area
            .y
            .saturating_add(u16::try_from(index).unwrap_or(u16::MAX));
        if line.style.bg.is_some() {
            buf.set_style(Rect::new(area.x, y, area.width, 1), line.style);
        }
        row(Rect::new(area.x, y, area.width, 1), buf, line);
    }
}

fn table_row_line(
    cells: &[String],
    widths: &[usize],
    separator: &str,
    separator_style: Style,
    cell_style: Style,
    ascii: bool,
) -> Line<'static> {
    let mut spans = Vec::new();
    for (index, cell) in cells.iter().enumerate() {
        if index > 0 {
            spans.push(Span::styled(separator.to_string(), separator_style));
        }
        let width = widths.get(index).copied().unwrap_or(0);
        spans.push(Span::styled(text::pad(cell, width, ascii), cell_style));
    }
    Line::from(spans)
}

/// A narrow table reads as one `column: value` entry per row rather than
/// columns squeezed past legibility.
fn stacked_lines(
    header: &[String],
    body: &[Vec<String>],
    width: usize,
    theme: &Theme,
) -> Vec<Line<'static>> {
    let mut out = Vec::new();
    for cells in body {
        for (index, cell) in cells.iter().enumerate() {
            let name = header.get(index).map(String::as_str).unwrap_or("");
            let entry = if name.is_empty() {
                cell.clone()
            } else {
                format!("{name}: {cell}")
            };
            out.extend(wrapped(
                ink_atoms(&entry, theme.fg(Role::Foreground)),
                width,
                width,
            ));
        }
    }
    out
}

fn table_lines<'a>(
    columns: &[Cow<'a, str>],
    rows: &[Vec<Cow<'a, str>>],
    width: usize,
    theme: &Theme,
) -> Vec<Line<'static>> {
    let ascii = theme.ascii();
    let col_count = columns
        .len()
        .max(rows.iter().map(Vec::len).max().unwrap_or(0));
    if col_count == 0 {
        return Vec::new();
    }
    let header: Vec<String> = (0..col_count)
        .map(|index| {
            columns
                .get(index)
                .map(|cell| text::display_safe(&cell.replace(['\n', '\r', '\t'], " ")).into_owned())
                .unwrap_or_default()
        })
        .collect();
    let body: Vec<Vec<String>> = rows
        .iter()
        .map(|row| {
            (0..col_count)
                .map(|index| {
                    row.get(index)
                        .map(|cell| {
                            text::display_safe(&cell.replace(['\n', '\r', '\t'], " ")).into_owned()
                        })
                        .unwrap_or_default()
                })
                .collect()
        })
        .collect();
    let separator = if ascii { " | " } else { " │ " };
    let separator_width = text::width(separator);
    let fixed = separator_width * col_count.saturating_sub(1);
    let mut widths: Vec<usize> = (0..col_count)
        .map(|index| {
            let header_width = text::width(&header[index]);
            let body_width = body
                .iter()
                .map(|row| text::width(&row[index]))
                .max()
                .unwrap_or(0);
            header_width.max(body_width).min(32)
        })
        .collect();
    while widths.iter().sum::<usize>() + fixed > width && widths.iter().any(|w| *w > 3) {
        let widest = widths
            .iter()
            .enumerate()
            .max_by_key(|(_, width)| **width)
            .map(|(index, _)| index)
            .unwrap_or(0);
        widths[widest] = widths[widest].saturating_sub(1);
    }
    if widths.iter().sum::<usize>() + fixed > width {
        return stacked_lines(&header, &body, width, theme);
    }
    let mut out = vec![table_row_line(
        &header,
        &widths,
        separator,
        theme.fg(Role::Border),
        theme.fg(Role::Muted).add_modifier(Modifier::BOLD),
        ascii,
    )];
    for row in &body {
        out.push(table_row_line(
            row,
            &widths,
            separator,
            theme.fg(Role::Border),
            theme.fg(Role::Foreground),
            ascii,
        ));
    }
    out
}

/// The semantic role a caller gave one span. The kit paints this role; it
/// never inspects the text to guess a different one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TranscriptSpanRole {
    Text,
    Strong,
    Emphasis,
    Code,
    Link,
    Muted,
    Success,
    Warning,
}

impl TranscriptSpanRole {
    pub const ALL: [Self; 8] = [
        Self::Text,
        Self::Strong,
        Self::Emphasis,
        Self::Code,
        Self::Link,
        Self::Muted,
        Self::Success,
        Self::Warning,
    ];

    #[must_use]
    pub const fn role(self) -> Role {
        match self {
            Self::Text | Self::Strong | Self::Emphasis => Role::Foreground,
            Self::Code | Self::Success => Role::Live,
            Self::Link => Role::Primary,
            Self::Muted => Role::Muted,
            Self::Warning => Role::Attention,
        }
    }

    #[must_use]
    pub fn style(self, theme: &Theme) -> Style {
        let base = theme.fg(self.role());
        match self {
            Self::Strong => base.add_modifier(Modifier::BOLD),
            Self::Emphasis => base.add_modifier(Modifier::ITALIC),
            // Underline is the only link mark: the target is never an escape
            // sequence, an OSC 8 hyperlink or a file destination.
            Self::Link => base.add_modifier(Modifier::UNDERLINED),
            _ => base,
        }
    }
}

/// One run of text with the semantic role its author gave it. `target` is
/// link metadata for the host; it is never painted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TranscriptSpan<'a> {
    pub text: Cow<'a, str>,
    pub role: TranscriptSpanRole,
    pub target: Option<Cow<'a, str>>,
}

impl<'a> TranscriptSpan<'a> {
    #[must_use]
    pub fn new(text: impl Into<Cow<'a, str>>, role: TranscriptSpanRole) -> Self {
        Self {
            text: text.into(),
            role,
            target: None,
        }
    }

    #[must_use]
    pub fn plain(text: impl Into<Cow<'a, str>>) -> Self {
        Self::new(text, TranscriptSpanRole::Text)
    }

    #[must_use]
    pub fn strong(text: impl Into<Cow<'a, str>>) -> Self {
        Self::new(text, TranscriptSpanRole::Strong)
    }

    #[must_use]
    pub fn emphasis(text: impl Into<Cow<'a, str>>) -> Self {
        Self::new(text, TranscriptSpanRole::Emphasis)
    }

    #[must_use]
    pub fn code(text: impl Into<Cow<'a, str>>) -> Self {
        Self::new(text, TranscriptSpanRole::Code)
    }

    #[must_use]
    pub fn muted(text: impl Into<Cow<'a, str>>) -> Self {
        Self::new(text, TranscriptSpanRole::Muted)
    }

    #[must_use]
    pub fn success(text: impl Into<Cow<'a, str>>) -> Self {
        Self::new(text, TranscriptSpanRole::Success)
    }

    #[must_use]
    pub fn warning(text: impl Into<Cow<'a, str>>) -> Self {
        Self::new(text, TranscriptSpanRole::Warning)
    }

    /// A link whose visible text is painted and whose target stays out of
    /// band for the host's own dispatch.
    #[must_use]
    pub fn link(text: impl Into<Cow<'a, str>>, target: impl Into<Cow<'a, str>>) -> Self {
        let mut span = Self::new(text, TranscriptSpanRole::Link);
        span.target = Some(target.into());
        span
    }

    #[must_use]
    pub fn target(mut self, target: impl Into<Cow<'a, str>>) -> Self {
        self.target = Some(target.into());
        self
    }

    #[must_use]
    pub fn style(&self, theme: &Theme) -> Style {
        self.role.style(theme)
    }
}

/// One action a code block reports to the host. The kit never copies.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TranscriptAction {
    Copy,
}

impl TranscriptAction {
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::Copy => "Copy",
        }
    }
}

const COPY_ACTIONS: [TranscriptAction; 1] = [TranscriptAction::Copy];
const NO_ACTIONS: [TranscriptAction; 0] = [];

/// Inset code with a language, an optional gutter and exact copy text.
///
/// `source` is the original, verbatim: `source()` and `copy_text()` return it
/// unchanged, newlines included. Painting uses `display`, when given, or the
/// sanitized source, and never mutates what a host would copy.
#[derive(Clone, Debug)]
pub struct CodeBlock<'a> {
    pub language: Cow<'a, str>,
    pub source: Cow<'a, str>,
    pub display: Option<Cow<'a, str>>,
    pub gutter: bool,
    pub inset: u16,
    pub copy: bool,
    pub copy_label: Cow<'a, str>,
    /// Optional caller semantic spans over the display text.
    pub spans: Vec<TranscriptSpan<'a>>,
}

impl<'a> CodeBlock<'a> {
    #[must_use]
    pub fn new(language: impl Into<Cow<'a, str>>, source: impl Into<Cow<'a, str>>) -> Self {
        Self {
            language: language.into(),
            source: source.into(),
            display: None,
            gutter: true,
            inset: 2,
            copy: true,
            copy_label: Cow::Borrowed("Copy"),
            spans: Vec::new(),
        }
    }

    /// A separate display copy. The original `source` is still what is copied.
    #[must_use]
    pub fn display(mut self, display: impl Into<Cow<'a, str>>) -> Self {
        self.display = Some(display.into());
        self
    }

    #[must_use]
    pub fn gutter(mut self, gutter: bool) -> Self {
        self.gutter = gutter;
        self
    }

    #[must_use]
    pub fn inset(mut self, inset: u16) -> Self {
        self.inset = inset;
        self
    }

    #[must_use]
    pub fn copyable(mut self, copy: bool) -> Self {
        self.copy = copy;
        self
    }

    #[must_use]
    pub fn copy_label(mut self, label: impl Into<Cow<'a, str>>) -> Self {
        self.copy_label = label.into();
        self
    }

    /// Caller-owned semantic spans for the display text. The kit paints these
    /// roles and never classifies the code itself.
    #[must_use]
    pub fn spans(mut self, spans: Vec<TranscriptSpan<'a>>) -> Self {
        self.spans = spans;
        self
    }

    /// The exact original text, newlines included.
    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }

    /// The exact original text a host would put on the clipboard.
    #[must_use]
    pub fn copy_text(&self) -> &str {
        &self.source
    }

    /// What this block paints, sanitized at paint time.
    #[must_use]
    pub fn display_text(&self) -> &str {
        self.display.as_deref().unwrap_or(&self.source)
    }

    #[must_use]
    pub fn copy_action(&self) -> Option<TranscriptAction> {
        self.copy.then_some(TranscriptAction::Copy)
    }

    #[must_use]
    pub fn actions(&self) -> &'static [TranscriptAction] {
        if self.copy {
            &COPY_ACTIONS
        } else {
            &NO_ACTIONS
        }
    }

    /// The block's painted rows at `width`: an optional language and Copy
    /// header, then the wrapped, gutter-numbered body.
    #[must_use]
    pub fn lines(&self, width: u16, theme: &Theme) -> Vec<Line<'static>> {
        let cells = usize::from(width);
        if cells == 0 {
            return Vec::new();
        }
        let ascii = theme.ascii();
        let mut out = Vec::new();
        let language = text::display_safe(&self.language);
        if !language.is_empty() || self.copy {
            let mut spans: Vec<Span<'static>> = Vec::new();
            if !language.is_empty() {
                spans.push(Span::styled(
                    language.into_owned(),
                    theme.fg(Role::Muted).add_modifier(Modifier::BOLD),
                ));
            }
            if self.copy {
                if !spans.is_empty() {
                    spans.push(Span::raw("  "));
                }
                spans.push(Span::styled(
                    text::display_safe(&self.copy_label).into_owned(),
                    theme.fg(Role::Primary),
                ));
            }
            out.push(Line::from(spans));
        }
        let logical = if self.spans.is_empty() {
            ink_atoms(self.display_text(), theme.fg(Role::Foreground))
        } else {
            span_atoms(&self.spans, theme)
        };
        let count = logical.len().max(1);
        let digits = count.to_string().len().max(2);
        let rail = if ascii { "| " } else { "│ " };
        let gutter = self.gutter && cells >= digits + 1 + text::width(rail) + 2;
        let gutter_width = if gutter {
            digits + 1 + text::width(rail)
        } else {
            0
        };
        let inset = usize::from(self.inset).min(cells.saturating_sub(gutter_width + 2));
        let lead = " ".repeat(inset);
        let indent = " ".repeat(gutter_width + inset);
        let prefix_width = gutter_width + inset;
        let budget = cells.saturating_sub(prefix_width).max(1);
        for (index, atoms) in logical.into_iter().enumerate() {
            let prefix = if gutter {
                format!("{:>width$} {}{}", index + 1, rail, lead, width = digits)
            } else {
                lead.clone()
            };
            out.extend(flow(
                vec![atoms],
                budget,
                budget,
                &prefix,
                theme.fg(Role::Border),
                &indent,
            ));
        }
        for line in &mut out {
            line.style = line.style.patch(theme.bg(Role::Surface));
        }
        out
    }

    #[must_use]
    pub fn height(&self, width: u16, theme: &Theme) -> u16 {
        u16::try_from(self.lines(width, theme).len()).unwrap_or(u16::MAX)
    }
}

impl Paint for CodeBlock<'_> {
    fn paint(&self, area: Rect, buf: &mut Buffer, theme: &Theme) {
        let area = area.intersection(buf.area);
        if area.is_empty() {
            return;
        }
        let lines = self.lines(area.width, theme);
        paint_lines(area, buf, &lines, 0);
    }

    fn height(&self, width: u16, theme: &Theme) -> u16 {
        u16::try_from(self.lines(width, theme).len()).unwrap_or(u16::MAX)
    }
}

/// One structured block a caller authored. The host parses; the kit paints.
#[derive(Clone, Debug)]
pub enum TranscriptBlock<'a> {
    /// A heading, `1` through `6`.
    Heading { level: u8, text: Cow<'a, str> },
    /// Prose with caller semantic spans.
    Paragraph { spans: Vec<TranscriptSpan<'a>> },
    /// A quote with an optional attribution line.
    Quote {
        text: Cow<'a, str>,
        attribution: Option<Cow<'a, str>>,
    },
    /// A list; `ordered` numbers the items.
    List {
        items: Vec<TranscriptSpan<'a>>,
        ordered: bool,
    },
    /// A table whose first row is `columns`.
    Table {
        columns: Vec<Cow<'a, str>>,
        rows: Vec<Vec<Cow<'a, str>>>,
    },
    /// Inset code with a language, a gutter and copy metadata.
    Code(CodeBlock<'a>),
}

impl<'a> TranscriptBlock<'a> {
    #[must_use]
    pub fn heading(level: u8, text: impl Into<Cow<'a, str>>) -> Self {
        Self::Heading {
            level,
            text: text.into(),
        }
    }

    #[must_use]
    pub fn paragraph(spans: Vec<TranscriptSpan<'a>>) -> Self {
        Self::Paragraph { spans }
    }

    /// A paragraph of plain prose.
    #[must_use]
    pub fn prose(text: impl Into<Cow<'a, str>>) -> Self {
        Self::Paragraph {
            spans: vec![TranscriptSpan::plain(text)],
        }
    }

    #[must_use]
    pub fn quote(text: impl Into<Cow<'a, str>>) -> Self {
        Self::Quote {
            text: text.into(),
            attribution: None,
        }
    }

    #[must_use]
    pub fn quote_by(text: impl Into<Cow<'a, str>>, attribution: impl Into<Cow<'a, str>>) -> Self {
        Self::Quote {
            text: text.into(),
            attribution: Some(attribution.into()),
        }
    }

    #[must_use]
    pub fn list(items: Vec<TranscriptSpan<'a>>) -> Self {
        Self::List {
            items,
            ordered: false,
        }
    }

    #[must_use]
    pub fn ordered(items: Vec<TranscriptSpan<'a>>) -> Self {
        Self::List {
            items,
            ordered: true,
        }
    }

    #[must_use]
    pub fn table(columns: Vec<Cow<'a, str>>, rows: Vec<Vec<Cow<'a, str>>>) -> Self {
        Self::Table { columns, rows }
    }

    #[must_use]
    pub fn code(block: CodeBlock<'a>) -> Self {
        Self::Code(block)
    }

    /// The block's painted rows at `width`.
    #[must_use]
    pub fn lines(&self, width: u16, theme: &Theme) -> Vec<Line<'static>> {
        let cells = usize::from(width).max(1);
        let ascii = theme.ascii();
        match self {
            Self::Heading { level, text } => {
                let style = match level {
                    1 => theme.fg(Role::Primary).add_modifier(Modifier::BOLD),
                    2 => theme.fg(Role::Foreground).add_modifier(Modifier::BOLD),
                    _ => theme.fg(Role::Muted).add_modifier(Modifier::BOLD),
                };
                wrapped(ink_atoms(text, style), cells, cells)
            }
            Self::Paragraph { spans } => wrapped(span_atoms(spans, theme), cells, cells),
            Self::Quote { text, attribution } => {
                let rail = if ascii { "| " } else { "▏ " };
                let ink = theme.fg(Role::Foreground).add_modifier(Modifier::ITALIC);
                let inner = cells.saturating_sub(2).max(1);
                let mut out = flow(
                    ink_atoms(text, ink),
                    inner,
                    inner,
                    rail,
                    theme.fg(Role::Border),
                    rail,
                );
                if let Some(attribution) = attribution {
                    let dash = if ascii { "- " } else { "— " };
                    let line = format!("{dash}{}", text::display_safe(attribution));
                    out.extend(flow(
                        ink_atoms(&line, theme.fg(Role::Muted)),
                        inner,
                        inner,
                        rail,
                        theme.fg(Role::Border),
                        rail,
                    ));
                }
                out
            }
            Self::List { items, ordered } => {
                let mut out = Vec::new();
                for (index, item) in items.iter().enumerate() {
                    let prefix = if *ordered {
                        format!("{}. ", index + 1)
                    } else if ascii {
                        "- ".to_string()
                    } else {
                        "• ".to_string()
                    };
                    let prefix_width = text::width(&prefix);
                    let inner = cells.saturating_sub(prefix_width).max(1);
                    let indent = " ".repeat(prefix_width);
                    out.extend(flow(
                        span_atoms(std::slice::from_ref(item), theme),
                        inner,
                        inner,
                        &prefix,
                        theme.fg(Role::Muted),
                        &indent,
                    ));
                }
                out
            }
            Self::Table { columns, rows } => table_lines(columns, rows, cells, theme),
            Self::Code(block) => block.lines(width, theme),
        }
    }

    #[must_use]
    pub fn height(&self, width: u16, theme: &Theme) -> u16 {
        u16::try_from(self.lines(width, theme).len()).unwrap_or(u16::MAX)
    }
}

impl Paint for TranscriptBlock<'_> {
    fn paint(&self, area: Rect, buf: &mut Buffer, theme: &Theme) {
        let area = area.intersection(buf.area);
        if area.is_empty() {
            return;
        }
        let lines = self.lines(area.width, theme);
        paint_lines(area, buf, &lines, 0);
    }

    fn height(&self, width: u16, theme: &Theme) -> u16 {
        u16::try_from(self.lines(width, theme).len()).unwrap_or(u16::MAX)
    }
}

/// A link the host may dispatch: the visible text and the out-of-band target.
/// The target is never painted, opened or written to a file by the kit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TranscriptLink {
    pub text: String,
    pub target: String,
}

fn collect_links(spans: &[TranscriptSpan<'_>], links: &mut Vec<TranscriptLink>) {
    for span in spans {
        if let Some(target) = &span.target {
            links.push(TranscriptLink {
                text: text::display_safe(&span.text).into_owned(),
                target: target.to_string(),
            });
        }
    }
}

/// A caller-authored transcript. `offset` is the host's scroll position in
/// painted rows: paint clips to it and to the given height.
#[derive(Clone, Debug, Default)]
pub struct Transcript<'a> {
    pub blocks: Vec<TranscriptBlock<'a>>,
    pub offset: usize,
}

impl<'a> Transcript<'a> {
    #[must_use]
    pub fn new(blocks: Vec<TranscriptBlock<'a>>) -> Self {
        Self { blocks, offset: 0 }
    }

    #[must_use]
    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = offset;
        self
    }

    #[must_use]
    pub fn block(mut self, block: TranscriptBlock<'a>) -> Self {
        self.blocks.push(block);
        self
    }

    /// Every painted row at `width`, before the host's offset clips it.
    #[must_use]
    pub fn lines(&self, width: u16, theme: &Theme) -> Vec<Line<'static>> {
        let mut out = Vec::new();
        for block in &self.blocks {
            out.extend(block.lines(width, theme));
        }
        out
    }

    #[must_use]
    pub fn height(&self, width: u16, theme: &Theme) -> u16 {
        u16::try_from(self.lines(width, theme).len()).unwrap_or(u16::MAX)
    }

    /// Every link in the transcript, in reading order. Out-of-band metadata
    /// for the host; nothing here paints.
    #[must_use]
    pub fn links(&self) -> Vec<TranscriptLink> {
        let mut links = Vec::new();
        for block in &self.blocks {
            match block {
                TranscriptBlock::Paragraph { spans } => collect_links(spans, &mut links),
                TranscriptBlock::List { items, .. } => collect_links(items, &mut links),
                TranscriptBlock::Code(block) => collect_links(&block.spans, &mut links),
                _ => {}
            }
        }
        links
    }
}

impl Paint for Transcript<'_> {
    fn paint(&self, area: Rect, buf: &mut Buffer, theme: &Theme) {
        let area = area.intersection(buf.area);
        if area.is_empty() {
            return;
        }
        let lines = self.lines(area.width, theme);
        paint_lines(area, buf, &lines, self.offset);
    }

    fn height(&self, width: u16, theme: &Theme) -> u16 {
        u16::try_from(self.lines(width, theme).len()).unwrap_or(u16::MAX)
    }
}
