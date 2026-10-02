# codewhale-ratatui

Codewhale's terminal component library: one visual language for agents, their
work, and the person steering them. Built on [Ratatui](https://ratatui.rs), with
the same design tokens as the Codewhale desktop app and website.

The kit paints **real session surfaces**—transcripts, tool output, fleets,
receipts, diffs, decisions and the composer—alongside the controls that build
them. Components accept facts from your host and paint through `Theme` and
`Role`. The host owns the Engine, permissions, persistence and event loop.

## See the components

These previews are generated from the **actual Ratatui cell buffers** used by
the gallery and snapshot tests. Every gallery entry appears in the dark and
light collections below; the profile comparison shows how the same state
marks adapt to all nine terminal profiles. SVGs contain no remote assets or
scripts. Open an image to inspect it at full size.

Profiles that leave colors to the terminal use a representative palette in
these images; your terminal supplies its own defaults.

<!-- gallery:start -->

Generated from the real ratatui buffers. Every catalogue entry is shown below.

Jump to: [Sessions and fleets](#sessions-and-fleets) · [The Codewhale language](#the-codewhale-language) · [Input and selection](#input-and-selection) · [Navigation and controls](#navigation-and-controls) · [Work and receipts](#work-and-receipts) · [Motion and feedback](#motion-and-feedback) · [A whale with a job](#a-whale-with-a-job) · [Every whale action](#every-whale-action) · [Terminal profiles](#terminal-profiles)

### Sessions and fleets

![Sessions and fleets — dark truecolor](<assets/readme/components.dark-truecolor.svg>)

### The Codewhale language

![The Codewhale language — dark truecolor](<assets/readme/foundation.dark-truecolor.svg>)

### Input and selection

![Input and selection — dark truecolor](<assets/readme/input.dark-truecolor-1.svg>)

![Input and selection — dark truecolor](<assets/readme/input.dark-truecolor-2.svg>)

![Input and selection — dark truecolor](<assets/readme/input.dark-truecolor-3.svg>)

### Navigation and controls

![Navigation and controls — dark truecolor](<assets/readme/chrome.dark-truecolor.svg>)

### Work and receipts

![Work and receipts — dark truecolor](<assets/readme/display.dark-truecolor-1.svg>)

![Work and receipts — dark truecolor](<assets/readme/display.dark-truecolor-2.svg>)

![Work and receipts — dark truecolor](<assets/readme/display.dark-truecolor-3.svg>)

![Work and receipts — dark truecolor](<assets/readme/display.dark-truecolor-4.svg>)

### Motion and feedback

![Motion and feedback — dark truecolor](<assets/readme/motion.dark-truecolor.svg>)

### A whale with a job

![A whale with a job — dark truecolor](<assets/readme/whales.dark-truecolor.svg>)

### Every whale action

![Every whale action — dark truecolor](<assets/readme/whale-actions.dark-truecolor.svg>)

<details>
<summary>Light theme</summary>

![Sessions and fleets — light truecolor](<assets/readme/components.light-truecolor.svg>)

![The Codewhale language — light truecolor](<assets/readme/foundation.light-truecolor.svg>)

![Input and selection — light truecolor](<assets/readme/input.light-truecolor-1.svg>)

![Input and selection — light truecolor](<assets/readme/input.light-truecolor-2.svg>)

![Input and selection — light truecolor](<assets/readme/input.light-truecolor-3.svg>)

![Navigation and controls — light truecolor](<assets/readme/chrome.light-truecolor.svg>)

![Work and receipts — light truecolor](<assets/readme/display.light-truecolor-1.svg>)

![Work and receipts — light truecolor](<assets/readme/display.light-truecolor-2.svg>)

![Work and receipts — light truecolor](<assets/readme/display.light-truecolor-3.svg>)

![Work and receipts — light truecolor](<assets/readme/display.light-truecolor-4.svg>)

![Motion and feedback — light truecolor](<assets/readme/motion.light-truecolor.svg>)

![A whale with a job — light truecolor](<assets/readme/whales.light-truecolor.svg>)

![Every whale action — light truecolor](<assets/readme/whale-actions.light-truecolor.svg>)

</details>

### Terminal profiles

![Terminal profile comparison](<assets/readme/profile-comparison.svg>)

<!-- gallery:end -->

## Component catalogue

| Family | Components | What they do |
|---|---|---|
| Session surfaces | `Message`, `ToolCard`, `Composer`, `AgentCard`, `Fleet` | Speaker anchors, output rails, honest omission counts, caller-owned prompts and each agent's own state, route and task |
| Identity and state | `Whale`, `WhaleState`, `Icon`, `StatusMark`, `StateWords` | The v2 whale's 17 actions and pods; marks always paired with words; localized state labels |
| Surfaces | `Panel`, `Depth`, `Dialog`, `Sheet`, `HorizonRule` | Deep, stage, raised and overlay grounds; centered decisions, edge-anchored sheets and the composer ledge |
| Navigation | `Heading`, `Tabs`, `KeyHints`, `Keymap`, `Picker`, `List` | Shared heading hierarchy, selection, scrolling, keyboard labels and caller-owned outcomes |
| Input and controls | `TextInput`, `Form`, `Toggle`, `Segmented` | Unicode-aware editing, masked fields, validation and controls that explain disabled state |
| Search and empty states | `PickerQuery`, `PickerTabs`, `PickerMatches`, fuzzy matching helpers, `EmptyState` | Ranked choices, search highlights, tabs, previews and a clear next action when there are no results |
| Work and evidence | `Receipt`, `ReceiptTable`, `Diff`, `WorkflowTree`, `CountBar` | Measured values, explicit unknowns, numbered additions/removals, workflow hierarchy and progress from known totals |
| Decisions | `ApprovalCard`, `ReviewVerdict`, `ReviewAggregate` | What will happen, where, why, and the caller's available next actions |
| Settings | `SettingRow`, `SettingDetail` | Value, source, lock reason, changed state, apply timing and reset details |
| Feedback and motion | `Toasts`, `Spinner`, motion helpers | Notices, measured elapsed time, bounded transitions and reduced/still motion |

Words and data arrive from the caller, with English defaults where useful.
The kit does not calculate a diff, parse Markdown, validate credentials,
authorize a command, estimate cost or run an agent.

## Use it

Use this repository as a Git dependency while the crate is developed:

```toml
[dependencies]
codewhale-ratatui = { git = "https://github.com/Hmbown/codewhale-ratatui" }
ratatui = "=0.30.2"
```

Rust 1.89 or later. The kit and host must share the same Ratatui and Crossterm
versions so their buffers, styles and key events are the same types.

```rust
use codewhale_ratatui::{
    Depth, KeyHint, KeyHints, Paint, Panel, Picker, PickerItem, PickerState, Theme,
};

// Once, after enabling raw mode, if the host does not already detect it:
codewhale_ratatui::detect::probe_terminal_background();
let theme = Theme::detect();

// In your draw callback, with a Ratatui area and buffer:
let hints = KeyHints::new(vec![
    KeyHint::new("↑↓", "move"),
    KeyHint::new("Enter", "select"),
    KeyHint::new("Esc", "cancel"),
]);
let inner = Panel::new(Depth::Overlay)
    .title("Mode")
    .hints(&hints)
    .draw(area, buf, &theme);
let items = [PickerItem::new("Work").key('1'), PickerItem::new("Plan").key('2')];
Picker::new(&items, PickerState::new(0)).paint(inner, buf, &theme);
```

Every `Paint` component also becomes a Ratatui widget with `.themed(&theme)`:

```rust
use codewhale_ratatui::{Composer, Paint, Theme};
let theme = Theme::detect();
let composer = Composer::new("Review the changes")
    .context("codewhale-ratatui / main");
frame.render_widget(composer.themed(&theme), frame.area());
```

The [gallery fixtures](src/gallery/) are runnable usage examples for every
family. [Component contribution instructions](CONTRIBUTING-COMPONENTS.md)
explain the rendering and ownership contracts.

## Choose a terminal profile

- **Truecolor:** exact token inks; dark terminals use the blue ombre by default.
  `Theme::ground(Ground::Graphite)` retains graphite grounds.
- **256 colors:** audited fixed-palette colors preserve contrast and state hues.
- **16 colors or unknown ground:** named terminal colors and visible marks/edges;
  the terminal owns the background.
- **`NO_COLOR`:** words, weight and marks carry every state.
- **`CODEWHALE_ASCII_SAFE=1`:** component chrome uses ASCII glyphs; user-authored
  Unicode remains text supplied by the host.

Set `CODEWHALE_APPEARANCE=light` or `dark` if the ground cannot be measured.
A host with its own detection can pass `Theme::new(Caps { depth, ascii,
appearance })` and avoid a second probe. Changes to a theme reach components
on their next paint; components hold roles rather than cached colors.

## Browse and regenerate

```sh
cargo run --example gallery                          # interactive catalogue
cargo run --example gallery -- --print dark-256     # ANSI preview to stdout
cargo run --example gallery -- --dump out/          # .ans and styled .txt, all profiles
cargo run --example gallery -- --svg target/readme-buffers
python3 tools/render-gallery.py target/readme-buffers assets/readme --readme README.md
python3 tools/render-gallery.py target/readme-buffers assets/readme --readme README.md --check
```

In the interactive gallery: `↑↓` or `j/k` selects a component, `p/P` switches
terminal profile, `w/W` switches width, `PgUp/PgDn` scrolls tall previews,
`Home/End` jumps through them, and `q` or `Esc` exits. This includes the full
17-action whale sheet on an ordinary-height terminal.

Profiles: `dark-truecolor`, `dark-graphite`, `light-truecolor`, `dark-256`,
`light-256`, `ansi-16`, `unknown-ground`, `no-color`, `ascii`.

## Verify it

```sh
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo test --example gallery --locked
python3 vendor/codewhale-design/generate.py --check
```

Snapshots record the role each run uses, alongside its glyphs. Tests exercise
profiles and widths, Unicode input, missing data, clipped output and disabled
controls. The generated README boards can be checked separately with the
command above. GitHub CI qualifies the branch; a local pass proves local
behavior only.

## Update design assets

Tokens are vendored from the private `codewhale-design` source. Maintainers
with that checkout can sync and regenerate:

```sh
../codewhale-design/scripts/sync-to.sh .
CODEWHALE_BLESS=1 cargo test --test generated
```

`src/roles.rs` is generated from the tokens, including terminal-derived hint,
dim and diff-tint roles. Contrast tests cover truecolor and quantized colors.

`assets/whale-v2.scenes` holds contours exported from the v2 whale kit. To
update or check them with that source available:

```sh
node tools/export-whale.cjs <path-to-whale-character-v2>
node tools/export-whale.cjs <path-to-whale-character-v2> --check
```

`tests/whale.rs` checks all 17 actions against the kit's 32×16 and 20×10 stills,
dot for dot. Artwork shows up to three calves; the state label gives the true
agent count, including larger fleets. Compact or ASCII terminals keep the
state in words when the art cannot fit.

## Integration status and license

The foundation was extracted from the Codewhale Engine
([Hmbown/CodeWhale](https://github.com/Hmbown/CodeWhale) at `58b1dd3dd`) and
extended here. The Engine has not migrated onto this crate yet. Completing
this library and its gallery does not establish Engine adoption or a release.

MIT. See [LICENSE](LICENSE).
