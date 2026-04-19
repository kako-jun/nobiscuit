# Changelog

All notable changes to nobiscuit are documented in this file. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.1] - 2026-04-19

### Changed
- `termray` dependency bumped from 0.1 to 0.3 (#28).
- `render_walls`, `render_floor_ceiling`, `project_sprites` call sites updated
  to pass `&FlatHeightMap` + `&Camera` (and `screen_height` for sprites).
  Visual output is unchanged since nobiscuit remains a tile-flat world.
- `rust-version` bumped to 1.85.0 and workspace edition to 2024 to align with
  termray 0.3.

### Notes
- `NobiscuitMap` stays a pure `TileMap` implementation. Corner-interpolated
  heights and `Camera.pitch` are available via termray 0.3 if we ever want
  to introduce uneven floors (tatami undulation, sunken genkan), but they
  are not used in this release.
- The workspace still pins `termray` via `path = "../../2026/termray"` until
  termray 0.3.x is published to crates.io. The declared version is `"0.3"`,
  so swapping back to a crates.io-only dep once published is a one-line change.

## [0.2.0] - 2026-04-18

### Changed
- Migrated the raycasting core to the external
  [`termray`](https://github.com/kako-jun/termray) crate (#27), extracting
  the generic engine layer out of the former `nobiscuit-engine`. nobiscuit
  now keeps only game-specific code (maze generation, Japanese-house
  textures, HUD, minimap, player state).
- Tile IDs renumbered: termray reserves `0=EMPTY` / `1=WALL` / `2=VOID`, so
  nobiscuit's props moved into the `3..=11` range.

## [0.1.x]

- Early iterations of the TUI raycasting maze game. See git history for
  detail.
