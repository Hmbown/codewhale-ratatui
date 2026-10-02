# Changelog

All notable changes to this crate are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/). The crate has not
been published or tagged; versions below `0.1.0` are development history.

## [Unreleased]

### Added

- Components extracted from the Codewhale engine TUI, with color detection,
  OSC 11 probing, the glyph charter, key labels, hint layout, modal sizing and
  spinner.
- The Codewhale whale drawn in Braille for all 17 actions, including the pod
  with up to three calves.
- The blue ombre dark ground, with the token grounds kept as
  `Ground::Graphite`.
- A gallery example that renders every component in nine terminal profiles,
  and snapshot tests that record the design role of each cell.

### Fixed

- Pod counts are reported honestly, cards are edged where grounds collapse, and
  ASCII marks are exact.
