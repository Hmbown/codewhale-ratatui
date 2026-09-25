# codewhale-ratatui

Paints Codewhale's terminal interface from one set of
[ratatui](https://ratatui.rs) components.

- **Takes every color from the Codewhale design tokens**, the same file the
  desktop app and the website read, and resolves it for the terminal in
  front of it: truecolor, 256 colors, 16 colors or `NO_COLOR`, on a light or
  dark ground.
- **Paints dark terminals in the blue ombre** by default: deep navy grounds
  rising toward the logo's blue, at the same luminance as the token grounds,
  so every contrast the tokens pass still passes. It shows at truecolor;
  256-color terminals keep the token grounds, because the color cube has no
  navy fine enough. `Theme::ground(Ground::Graphite)` keeps the token
  grounds everywhere.
- **Pairs every state with a mark and a word** (`● Working`, `◆ Needs you`,
  `✕ Failed`), so nothing depends on color alone.
- **Draws the Codewhale whale in Braille** from the same contours as the
  desktop pet, in all 17 of its actions: resting, listening, thinking,
  reading, searching, editing, running, browsing, replying, using the
  computer, calling a connected app, needs you, done, stuck, asleep, and a
  pod whose calves swim with it when agents work in parallel. The art draws
  up to three calves; the words beneath always give the real count.
- **Spells keys one way**: `Ctrl+O`, `⌥V` on macOS (`Alt+V` elsewhere),
  `↑↓`, or `Up/Down` in ASCII-safe terminals.

## Use it

```rust
use codewhale_ratatui::{Depth, KeyHint, KeyHints, Paint, Panel, Picker, PickerItem, PickerState, Theme};

// Once, after enabling raw mode: measure the terminal's ground (OSC 11).
codewhale_ratatui::detect::probe_terminal_background();
let theme = Theme::detect();

// In your draw callback:
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

If nothing measures the terminal's ground (no OSC 11 reply, no
`COLORFGBG`), the theme uses the terminal's own 16 named colors and paints no
grounds, rather than guess. Set `CODEWHALE_APPEARANCE=light` or `=dark` to
tell it, or set `Caps::appearance` from your own theme setting.
`CODEWHALE_ASCII_SAFE=1` draws every mark in ASCII.

A host that already detects the terminal (the Codewhale engine does) builds
the theme from what it knows instead of probing twice:
`Theme::new(Caps { depth, ascii, appearance }).ground(Ground::Graphite)`.

Components name a `Role` (`Muted`, `Live`, `Danger`, …), never a color. The
`Theme` resolves the role when it paints, so a theme or terminal change
reaches every component on the next frame.

| Component | What it shows |
|---|---|
| `KeyHints` | `↑↓ move · Enter select · Esc cancel`; wraps instead of dropping an action |
| `StatusMark` | a state's mark and word; the mark takes the state's hue |
| `Panel` + `Depth` | deep, stage, raised and overlay grounds; an edge where grounds cannot paint |
| `HorizonRule` | the one full-width rule, above the place you type |
| `Picker` | choose-one and checklist rows; selection is a marker, bold and a ground |
| `Toasts` | one-line notices at the bottom right |
| `Spinner` | appears after 400 ms, holds still under reduced motion, shows measured time |
| `Icon` | sonar, tide, shell and kelp, one cell each, with words and ASCII forms; none borrows a state's mark |
| `Whale` | the v2 whale in Braille, any of its 17 actions, with its state in words beneath |

## See every component

```sh
cargo run --example gallery                        # browse; p changes the profile
cargo run --example gallery -- --print dark-256    # print one profile to stdout
cargo run --example gallery -- --dump out/         # write .ans and .txt for every profile
```

Profiles: `dark-truecolor` (the blue ombre), `dark-graphite`,
`light-truecolor`, `dark-256`, `light-256`, `ansi-16`, `unknown-ground`,
`no-color`, `ascii`.

## Test it

```sh
cargo test                  # unit, generated-roles, whale and snapshot tests
cargo insta review          # review changed snapshots
```

Snapshots record the role each cell was painted with, not its hex value, so
a token change does not rewrite them but painting the wrong role does. Where
two roles look the same at a depth, the snapshot names both
(`bg=Background|Surface`).

## Update the tokens

The tokens live in the `codewhale-design` repository. Vendor a new version
and regenerate the roles:

```sh
../codewhale-design/scripts/sync-to.sh .
CODEWHALE_BLESS=1 cargo test --test generated
```

`src/roles.rs` holds the roles, the truecolor tables, a 256-color table and
the blue ombre table. The 256-color table starts from the nearest fixed
index and moves an ink only where quantizing breaks the design's contrast
floors or turns a state hue gray. The ombre tints the dark grounds and the
quiet line toward the logo's `#0B48BB` and restores each one's luminance,
and the generator fails if any contrast floor breaks. `cargo test` fails if
`roles.rs` no longer matches `tokens.json`.

## Update the whale

`assets/whale-v2.scenes` holds the poster contours of all 17 actions (the
pod with one, two and three calves), exported from the whale-character-v2
kit:

```sh
node tools/export-whale.cjs <path-to-whale-character-v2>          # rewrite
node tools/export-whale.cjs <path-to-whale-character-v2> --check  # verify
```

`tests/whale.rs` checks every action against the kit's own 32×16 and 20×10
stills, dot for dot, and fails if the kit gains an action this crate cannot
draw.

## Where it came from

The color detection, OSC 11 probe, glyph charter, key labels, hint layout,
modal sizing and spinner are extracted from the Codewhale engine
([Hmbown/CodeWhale](https://github.com/Hmbown/CodeWhale) at `58b1dd3dd`);
each module names its source. The engine does not use this crate yet.

## License

MIT
