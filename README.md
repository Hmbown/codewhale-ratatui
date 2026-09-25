# codewhale-ratatui

Paints Codewhale's terminal interface from one set of
[ratatui](https://ratatui.rs) components.

- **Takes every color from the Codewhale design tokens**, the same file the
  desktop app and the website read, and resolves it for the terminal in
  front of it: truecolor, 256 colors, 16 colors or `NO_COLOR`, on a light or
  dark ground.
- **Pairs every state with a mark and a word** (`● Working`, `◆ Needs you`,
  `✕ Failed`), so nothing depends on color alone.
- **Draws the Codewhale whale in Braille** from the same contours as the
  desktop pet: resting, working, needs you, done, and a pod with up to three
  calves when agents work in parallel.
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
| `Icon` | sonar, tide, shell and kelp, one cell each, with words and ASCII forms |
| `Whale` | the v2 whale in Braille with its state in words beneath |

## See every component

```sh
cargo run --example gallery                        # browse; p changes the profile
cargo run --example gallery -- --print dark-256    # print one profile to stdout
cargo run --example gallery -- --dump out/         # write .ans and .txt for every profile
```

Profiles: `dark-truecolor`, `light-truecolor`, `dark-256`, `light-256`,
`ansi-16`, `unknown-ground`, `no-color`, `ascii`.

## Test it

```sh
cargo test                  # unit, generated-roles, whale and snapshot tests
cargo insta review          # review changed snapshots
```

Snapshots record the role each cell was painted with, not its hex value, so
a token change does not rewrite them but painting the wrong role does.

## Update the tokens

The tokens live in the `codewhale-design` repository. Vendor a new version
and regenerate the roles:

```sh
../codewhale-design/scripts/sync-to.sh .
CODEWHALE_BLESS=1 cargo test --test generated
```

`src/roles.rs` holds the roles, the truecolor tables and a 256-color table.
The 256-color table starts from the nearest fixed index and moves an ink
only where quantizing breaks the design's contrast floors or turns a state
hue gray. `cargo test` fails if `roles.rs` no longer matches `tokens.json`.

## Update the whale

`assets/whale-v2.scenes` holds each state's poster contours, exported from
the whale-character-v2 kit:

```sh
node tools/export-whale.cjs <path-to-whale-character-v2>          # rewrite
node tools/export-whale.cjs <path-to-whale-character-v2> --check  # verify
```

`tests/whale.rs` checks every state against the kit's own 32×16 and 20×10
stills, dot for dot.

## Where it came from

The color detection, OSC 11 probe, glyph charter, key labels, hint layout,
modal sizing and spinner are extracted from the Codewhale engine
([Hmbown/CodeWhale](https://github.com/Hmbown/CodeWhale) at `58b1dd3dd`);
each module names its source. The engine does not use this crate yet.

## License

MIT
