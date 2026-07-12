# Changelog

All notable changes to nobiscuit are documented in this file. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- Maze generation rewritten from mask-based per-island DFS to BSP space partition
  (家の間取り化, #31). Each island's bounding box is recursively split (even-coordinate
  split lines) into rectangular rooms; adjacent rooms are joined by fusuma doors via a
  spanning tree plus ~15% loop doors, and up to two straight width-3 corridors are carved
  per island. Removed `generate_corridors`, `carve_island`, and `place_rooms`; added
  `bsp_layout`/`bsp_split`, `connect_rooms`, `verify_connectivity`, and `generate_goal_floor`.
- The top floor now uses a fixed template (descend-stairs → vertical corridor → fusuma →
  Nobita's room with the GOAL centered) instead of a generated maze.
- `generate_floor` verifies reachability with a flood fill and regenerates (up to 10 tries)
  when a walkable cell is unreachable, walling off any stragglers as a last resort. The spawn
  corner `(1,1)` is always made walkable and wired into the layout.

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
