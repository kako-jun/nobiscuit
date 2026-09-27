# nobiscuit

## Overview

TUI raycasting 3D maze game. Rust + crossterm. Half-block character rendering (`▀`) with 24-bit true color.

## Architecture

Single-crate workspace depending on the external [`termray`](https://github.com/kako-jun/termray) crate:
- `nobiscuit-cli` — Game binary. Terminal rendering, input, maze generation, game state. Plus nobiscuit-specific `NobiscuitMap`, `NobiscuitTextures` (wall/floor/sprite art) and tile IDs layered on top of termray.

termray (extracted from the former `nobiscuit-engine`) owns the generic raycasting: DDA, camera, framebuffer, TileMap trait, wall/floor/sprite rendering skeletons. nobiscuit injects Japanese-house visuals through `WallTexturer` / `FloorTexturer` / `SpriteArt` trait impls.

## Key concepts

- **Framebuffer**: termray writes `Color` pixels to a width x height buffer. Height = terminal rows * 2 (half-block doubles vertical resolution).
- **DDA raycasting**: One ray per screen column. Grid traversal to find wall hits. Fisheye correction applied by camera.
- **Delta flushing**: Terminal renderer double-buffers. Only changed cells emit ANSI escape sequences. Critical for 30fps.
- **TileMap trait**: termray operates on `&dyn TileMap`. nobiscuit provides `NobiscuitMap` with custom `is_solid` (goals, stairs and pass-through windows are walkable; closed doors/decorative windows/shoji are solid).
- **Trait-based textures**: `NobiscuitTextures` implements termray's `WallTexturer`, `FloorTexturer`, and `SpriteArt`. All Japanese-house styling lives in `crates/nobiscuit-cli/src/textures.rs`.

## Build & run

```bash
cargo run -p nobiscuit-cli          # debug
cargo run --release -p nobiscuit-cli # release (recommended)
cargo clippy                        # lint
```

## Module map

### CLI (crates/nobiscuit-cli/src/)
| File | Purpose |
|---|---|
| main.rs | Game loop (input → update → render → present) |
| terminal.rs | Half-block ANSI renderer with delta flushing |
| input.rs | Non-blocking crossterm key polling |
| maze.rs | Non-overlapping BSP house plan and region-pair connections |
| player.rs | Grid-based Wizardry-style movement with animation |
| minimap.rs | Semi-transparent 2D map overlay |
| game.rs | Game state (hunger, biscuit pickup, escape) |
| ui.rs | HUD (hunger bar, bitmap font messages) |
| tiles.rs | Nobiscuit tile IDs (GOAL, WINDOW, STAIRS, DOORS, SHOJI) |
| nobiscuit_map.rs | NobiscuitMap: TileMap impl with nobiscuit-aware is_solid |
| windows.rs | Walk-through window frames with per-pixel depth compositing |
| textures.rs | WallTexturer / FloorTexturer / SpriteArt implementations (fusuma/shoji/tatami/biscuit) |

### External engine
Raycasting primitives live in the [termray](https://github.com/kako-jun/termray) crate (extracted from the former `nobiscuit-engine`).

## Current features

- Multi-floor maze (1–12 floors connected by stairs)
- BSP house plans: bounded rectangular rooms (interior 3–9 cells), one reserved width-3 main hallway with entrances on both sides, and one opening per adjacent region pair. Room-to-room doors/windows plus a spanning tree and limited loops keep partitions intact. One connected floor; no overlapping island bounds or wall-off fallback. Fixed top-floor goal template
- Doors (fusuma, kitchen, toilet, genkan) with auto-open/close and corridor-hub structure
- Solid decorative windows and walk-through window openings (TILE_WINDOW_PASS=12), with depth-composited frames visible from both rooms
- Shoji tiles (wooden lattice + washi paper, embedded in wall — upper 20% / lower 30% wall frame)
- Stair sprites (up/down arrows) with floor transition
- Floor indicator HUD (e.g. "2F" with dot indicators)
- Per-floor independent maze generation with biscuits
- Minimap visibility restriction: fog of war, timed display (M key: 3s with hunger cost, biscuit: 2s full reveal), debug mode via NOBISCUIT_DEBUG=1
- Game over/clear presentation: 3s fade-out, collapse animation (game over), staged title reveal (clear), score display, Y/N retry
- Galagala opening: spin count determines maze size (15x13~121x91) and floors (1~12). Camera shake, color-coded spin counter, retry returns to galagala

## Future work
- Flatten single-crate workspace (rename `nobiscuit-cli` → `nobiscuit`, drop `crates/`)
- Movable walls/windows (home maze dynamic rearrangement)
- Infinite maze (chunk-based generation)
- Sound (terminal bell or external)

## Reproducible verification

`NOBISCUIT_DEBUG=1 NOBISCUIT_SEED=42 NOBISCUIT_SPINS=5 cargo run --release`
starts directly on a generated floor with the full minimap. Seed is a u64; spins
is a positive u32. Invalid/unset values preserve random generation/the lottery.
Retry returns to the lottery without resetting the seeded random stream.
