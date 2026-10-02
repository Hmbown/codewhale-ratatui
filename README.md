# Codewhale, in the terminal

Codewhale's terminal component library: one visual language for agents, their
work, and the person steering them. Built on [Ratatui](https://ratatui.rs), with
the same design tokens as the Codewhale desktop app and website.

An ocean-blue workspace for serious work: a conversation at the center,
agents and decisions close at hand, files you can inspect, and receipts you
can read. Compose a quiet project, a focused review, or a dense fleet station
from the same parts. The **workbar** brings TODO, context, git, price and
plugins together when you summon it. The **habitat** brings the fish school,
jellyfish, bubbles and whale companion into clear water around the work.

These are working Ratatui components. They accept facts from your host and
paint through `Theme` and `Role`. The host owns the Engine, permissions,
persistence, clock and event loop. Showcase tasks and measurements are
illustrative fixture data.

## See the components

These previews are generated from the **actual Ratatui cell buffers** used by
the gallery and snapshot tests. Every gallery entry appears in the dark and
light collections below; the profile comparison shows how the same state
marks adapt to all nine terminal profiles. SVGs contain no remote assets or
scripts. Open an image to inspect it at full size.

Profiles that leave colors to the terminal use a representative palette in
these images; your terminal supplies its own defaults.

<details>
<summary>Watch the habitat move — full motion demonstration</summary>

![The fish school, jellyfish and bubbles moving around Codewhale's words](assets/readme/habitat-motion.gif)

Eighty actual terminal-buffer frames. This demonstration holds the jellyfish
visit open; the component normally makes it a rare visitor. Ambient life is
absent under `MotionMode::Reduced` and `Still`. Run `cargo run --example habitat`
to switch the motion policy and all nine terminal profiles yourself.

</details>

<!-- gallery:start -->

Generated from the real ratatui buffers. Every catalogue entry is shown below.

Jump to: [Codewhale at work](#codewhale-at-work) · [Sessions and fleets](#sessions-and-fleets) · [The workbar](#the-workbar) · [The Codewhale language](#the-codewhale-language) · [Input and selection](#input-and-selection) · [Navigation and controls](#navigation-and-controls) · [Work and receipts](#work-and-receipts) · [Motion and feedback](#motion-and-feedback) · [Life in the water](#life-in-the-water) · [A whale with a job](#a-whale-with-a-job) · [Every whale action](#every-whale-action) · [Terminal profiles](#terminal-profiles)

### Codewhale at work

![Codewhale at work — dark truecolor](<assets/readme/scenes.dark-truecolor-1.svg>)

![Codewhale at work — dark truecolor](<assets/readme/scenes.dark-truecolor-2.svg>)

![Codewhale at work — dark truecolor](<assets/readme/scenes.dark-truecolor-3.svg>)

![Codewhale at work — dark truecolor](<assets/readme/scenes.dark-truecolor-4.svg>)

![Codewhale at work — dark truecolor](<assets/readme/scenes.dark-truecolor-5.svg>)

### Sessions and fleets

![Sessions and fleets — dark truecolor](<assets/readme/components.dark-truecolor-1.svg>)

![Sessions and fleets — dark truecolor](<assets/readme/components.dark-truecolor-2.svg>)

### The workbar

![The workbar — dark truecolor](<assets/readme/workbar.dark-truecolor.svg>)

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

![Work and receipts — dark truecolor](<assets/readme/display.dark-truecolor-5.svg>)

### Motion and feedback

![Motion and feedback — dark truecolor](<assets/readme/motion.dark-truecolor.svg>)

<details>
<summary>Watch the working and verification spinners, then the receipt arrive</summary>

![Working, verifying and settling into a receipt — Ocean](<assets/readme/motion-demo.gif>)

![Working, verifying and settling into a receipt — Paper](<assets/readme/motion-demo-light.gif>)

A demonstration of the actual components at their normal cadence: the working swell,
verification tick, state ink, selection movement and detail reveal.
The demonstration supplies each state change; the component supplies its motion.
Reduced and still modes use readable static marks and settle transitions immediately.
Run `cargo run --example motion` to finish, restart, switch phases and change motion policy yourself.

</details>

### Life in the water

![Life in the water — dark truecolor](<assets/readme/habitat.dark-truecolor-1.svg>)

![Life in the water — dark truecolor](<assets/readme/habitat.dark-truecolor-2.svg>)

### A whale with a job

![A whale with a job — dark truecolor](<assets/readme/whales.dark-truecolor.svg>)

### Every whale action

![Every whale action — dark truecolor](<assets/readme/whale-actions.dark-truecolor.svg>)

<details>
<summary>Light theme</summary>

![Codewhale at work — light truecolor](<assets/readme/scenes.light-truecolor-1.svg>)

![Codewhale at work — light truecolor](<assets/readme/scenes.light-truecolor-2.svg>)

![Codewhale at work — light truecolor](<assets/readme/scenes.light-truecolor-3.svg>)

![Codewhale at work — light truecolor](<assets/readme/scenes.light-truecolor-4.svg>)

![Codewhale at work — light truecolor](<assets/readme/scenes.light-truecolor-5.svg>)

![Sessions and fleets — light truecolor](<assets/readme/components.light-truecolor-1.svg>)

![Sessions and fleets — light truecolor](<assets/readme/components.light-truecolor-2.svg>)

![The workbar — light truecolor](<assets/readme/workbar.light-truecolor.svg>)

![The Codewhale language — light truecolor](<assets/readme/foundation.light-truecolor.svg>)

![Input and selection — light truecolor](<assets/readme/input.light-truecolor-1.svg>)

![Input and selection — light truecolor](<assets/readme/input.light-truecolor-2.svg>)

![Input and selection — light truecolor](<assets/readme/input.light-truecolor-3.svg>)

![Navigation and controls — light truecolor](<assets/readme/chrome.light-truecolor.svg>)

![Work and receipts — light truecolor](<assets/readme/display.light-truecolor-1.svg>)

![Work and receipts — light truecolor](<assets/readme/display.light-truecolor-2.svg>)

![Work and receipts — light truecolor](<assets/readme/display.light-truecolor-3.svg>)

![Work and receipts — light truecolor](<assets/readme/display.light-truecolor-4.svg>)

![Work and receipts — light truecolor](<assets/readme/display.light-truecolor-5.svg>)

![Motion and feedback — light truecolor](<assets/readme/motion.light-truecolor.svg>)

![Life in the water — light truecolor](<assets/readme/habitat.light-truecolor-1.svg>)

![Life in the water — light truecolor](<assets/readme/habitat.light-truecolor-2.svg>)

![A whale with a job — light truecolor](<assets/readme/whales.light-truecolor.svg>)

![Every whale action — light truecolor](<assets/readme/whale-actions.light-truecolor.svg>)

</details>

### Terminal profiles

![Terminal profile comparison](<assets/readme/profile-comparison.svg>)

<!-- gallery:end -->

## Component catalogue

| Family | Components | What they do |
|---|---|---|
| Workspace composition | `WorkspaceFrame`, `WorkspaceAreas`, `PaneHeader`, `ContextRibbon`, `ContextItem` | Responsive conversation and dock regions, one quiet module header, composer-adjacent facts folded by priority with explicit counts |
| Workbar | `Workbar`, `WorkbarItem` | Summoned TODO, context, git, price and plugin readouts; responsive columns, caller selection, pointer geometry and explicit unknowns |
| Attention and results | `AttentionQueue`, `AttentionItem`, `ArtifactShelf`, `Artifact` | Project-aware decisions, selected action hints, review/file/run/link results and reported receipts |
| Marine life | `Habitat`, `FishSchool`, `Jellyfish`, `BubbleField`, `HabitatDensity` | Native braille poses and ASCII silhouettes, caller-clock motion, bounded populations, complete visitors and text-safe open-water collision |
| Session surfaces | `Message`, `ToolCard`, `Composer`, `AgentCard`, `Fleet` | Speaker anchors, output rails, honest omission counts, caller-owned prompts and each agent's own state, route and task |
| Identity and state | `Whale`, `WhaleState`, `Icon`, `StatusMark`, `StateWords` | The v2 whale's 17 actions and pods; marks always paired with words; localized state labels |
| Surfaces | `Panel`, `Depth`, `Dialog`, `Sheet`, `HorizonRule` | Deep, stage, raised and overlay grounds; centered decisions, edge-anchored sheets and the composer ledge |
| Navigation | `Heading`, `Tabs`, `KeyHints`, `Keymap`, `Picker`, `List` | Shared heading hierarchy, selection, scrolling, keyboard labels and caller-owned outcomes |
| Input and controls | `TextInput`, `Form`, `Toggle`, `Segmented` | Unicode-aware editing, masked fields, validation and controls that explain disabled state |
| Search and empty states | `PickerQuery`, `PickerTabs`, `PickerMatches`, fuzzy matching helpers, `EmptyState` | Ranked choices, search highlights, tabs, previews and a clear next action when there are no results |
| Work and evidence | `Receipt`, `ReceiptTable`, `Diff`, `WorkflowTree`, `CountBar` | Measured values, explicit unknowns, numbered additions/removals, workflow hierarchy and progress from known totals |
| Decisions | `ApprovalCard`, `ReviewVerdict`, `ReviewAggregate` | What will happen, where, why, and the caller's available next actions |
| Settings | `SettingRow`, `SettingDetail` | Value, source, lock reason, changed state, apply timing and reset details |
| Feedback and motion | `Toasts`, `Spinner`, `VerificationSpinner`, `MotionStep`, `MotionSet`, `FrameBudget` | Working swell, verification tick, notices, measured elapsed time, bounded transitions and reduced/still motion |

Words and data arrive from the caller, with English defaults where useful.
The kit does not calculate a diff, parse Markdown, validate credentials,
authorize a command, estimate cost or run an agent.

## Spinners and animation

`Spinner` uses Codewhale's eight-frame swell; `VerificationSpinner` uses the
Engine's distinct round verification tick. Both wait 400 ms before moving,
advance at five steps per second, and keep the caller's work verb visible.
Reduced and still motion show a static mark plus words. ASCII terminals have
their own frames.

`MotionStep` and `MotionSet` handle token-timed state ink, selection movement
and detail reveal. The caller changes the state and supplies the instant;
the state words change immediately. `FrameBudget` combines redraw deadlines
and lets the host claim one primary spinner per frame. Once transitions
settle, the host can wait for input instead of painting identical frames.

The animated demonstrations are under [Motion and feedback](#motion-and-feedback).
The normal gallery samples fixed instants; `cargo run --example motion` is
the live example.

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

Compose a workspace without creating another session or event authority:

```rust
use codewhale_ratatui::{Paint, PaneHeader, WorkspaceFrame};

let workspace = WorkspaceFrame::new("my-project")
    .branch("main")
    .footer("Local workspace");
workspace.paint(area, buf, &theme);
let regions = workspace.areas(area.intersection(buf.area));
PaneHeader::new("Conversation").paint(regions.main, buf, &theme);
if let Some(side) = regions.side {
    PaneHeader::new("Files & review").paint(side, buf, &theme);
}
// Paint your transcript, composer and caller-owned modules in these regions.
// At narrow widths the optional dock yields its space to the conversation.
```

For open water, paint foreground content first, then call `Habitat::paint`.
It protects occupied cells and their clearance; the entire jellyfish is
withheld when its silhouette cannot fit. Pass decision and overlay rectangles
to `Habitat::protected` so their blank space stays protected too. Keep a
dedicated habitat viewport separate from any decision overlay. The habitat
never requests a frame itself. Selection and pointer helpers use the same clipped
viewport passed to painting.

The whale's ordinary `Paint` implementation shows its current poster pose.
For animation, pass the packed `whale::Grid` evaluated by your existing
owner to `Whale::paint_frame(area, buf, &theme, &grid)`. The widget paints that
exact frame and its state words; it owns no Director or clock. The whole
frame must fit, with a row for the label. Invalid, narrow or ASCII frames
fall back to words. Repaint the underlying surface first because empty
cells in the frame are transparent. This adapter does not port the
authoritative Director's springs, clips or lifecycle into another runtime.

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
cargo run --example habitat                          # live fish, jellyfish, bubbles
cargo run --example motion                           # working, verification and transitions
cargo run --example gallery -- --print dark-256     # ANSI preview to stdout
cargo run --example gallery -- --dump out/          # .ans and styled .txt, all profiles
cargo run --example gallery -- --svg target/readme-buffers
python3 tools/render-gallery.py target/readme-buffers assets/readme --readme README.md
python3 tools/render-gallery.py target/readme-buffers assets/readme --readme README.md --check
```

The optional animation build needs Node, `sharp` and FFmpeg. Each animation
comes from deterministic actual-buffer frames, using one shared media builder:

```sh
cargo run --locked --example habitat -- --frames target/habitat-frames
node tools/render-animation.cjs target/habitat-frames assets/readme/habitat-motion.gif
python3 tools/check-animation.py target/habitat-frames assets/readme/habitat-motion.gif
cargo run --locked --example motion -- --frames target/motion-frames
node tools/render-animation.cjs target/motion-frames assets/readme/motion-demo.gif
python3 tools/check-animation.py target/motion-frames assets/readme/motion-demo.gif
cargo run --locked --example motion -- --frames target/motion-light-frames --profile light-truecolor
node tools/render-animation.cjs target/motion-light-frames assets/readme/motion-demo-light.gif
python3 tools/check-animation.py target/motion-light-frames assets/readme/motion-demo-light.gif
```

CI verifies both the current frame hash and the GIF file hash; it needs no
raster tools. Static previews and the live terminal example use the normal
Rust/Python toolchain.

In the interactive gallery: `↑↓` or `j/k` selects a component, `p/P` switches
terminal profile, `w/W` switches width, `PgUp/PgDn` scrolls tall previews,
`Home/End` jumps through them, and `q` or `Esc` exits. This includes the full
17-action whale sheet on an ordinary-height terminal. `f` expands the canvas
for the composed workspace scenes. The habitat example uses `p` for profile,
`m` for motion and `q` to close.
In the motion example, `Space` finishes or restarts the demonstration, `v`
switches working/verification, `r` replays, `p` changes profile, and `m`
changes motion policy. `q` or `Esc` closes it.

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
