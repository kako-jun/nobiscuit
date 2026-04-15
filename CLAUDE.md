# nobiscuit

## Overview

TUI raycasting 3D maze game. Rust + crossterm. Half-block character rendering (`▀`) with 24-bit true color.

## Architecture

2-crate workspace:
- `nobiscuit-engine` — Pure raycasting engine. Zero external dependencies. Renders to abstract `Framebuffer` (Color pixel grid). Designed for reuse by gemma-mon.
- `nobiscuit-cli` — Game binary. Terminal rendering, input, maze generation, game state.

## Key concepts

- **Framebuffer**: Engine writes `Color` pixels to a width x height buffer. Height = terminal rows * 2 (half-block doubles vertical resolution).
- **DDA raycasting**: One ray per screen column. Grid traversal to find wall hits. Fisheye correction applied by camera.
- **Delta flushing**: Terminal renderer double-buffers. Only changed cells emit ANSI escape sequences. Critical for 30fps.
- **TileMap trait**: Engine operates on `&dyn TileMap`. Game provides concrete implementation (`GridMap` from maze generation).

## Build & run

```bash
cargo run -p nobiscuit-cli          # debug
cargo run --release -p nobiscuit-cli # release (recommended)
cargo clippy                        # lint
```

## Module map

### Engine (crates/nobiscuit-engine/src/)
| File | Purpose |
|---|---|
| math.rs | Vec2f, angle normalization |
| ray.rs | DDA raycasting algorithm |
| map.rs | TileMap trait + GridMap implementation |
| camera.rs | Camera position/angle, cast_all_rays |
| renderer.rs | Wall column rendering with procedural textures |
| floor.rs | Floor/ceiling with perspective-correct tile patterns |
| framebuffer.rs | Color + pixel buffer + alpha blending |
| sprite.rs | Sprite projection + AA art rendering |

### CLI (crates/nobiscuit-cli/src/)
| File | Purpose |
|---|---|
| main.rs | Game loop (input → update → render → present) |
| terminal.rs | Half-block ANSI renderer with delta flushing |
| input.rs | Non-blocking crossterm key polling |
| maze.rs | Mask-based irregular maze generation (per-island DFS) |
| player.rs | Grid-based Wizardry-style movement with animation |
| minimap.rs | Semi-transparent 2D map overlay |
| game.rs | Game state (hunger, biscuit pickup, escape) |
| ui.rs | HUD (hunger bar, bitmap font messages) |

## Current features

- Multi-floor maze (3 floors connected by stairs)
- Irregular maze shapes (VOID tiles + mask-based generation, per-island DFS)
- Variable corridor width + room placement (2x2~4x3 rooms, flood-fill guarded wall removal)
- Doors (fusuma, kitchen, toilet, genkan) with auto-open/close and corridor-hub structure
- Window tiles (glass pane with wooden frame, distinct from walls)
- Stair sprites (up/down arrows) with floor transition
- Floor indicator HUD (e.g. "2F" with dot indicators)
- Per-floor independent maze generation with biscuits
- Minimap visibility restriction: fog of war, timed display (M key: 3s with hunger cost, biscuit: 2s full reveal), debug mode via NOBISCUIT_DEBUG=1

## Future work
- #7 Shoji (translucent window variant)
- #8 Galagala opening sequence (difficulty selection + goal)
- #9 Game over/clear presentation (fade, delay, retry)
- Movable walls/windows (home maze dynamic rearrangement)
- Infinite maze (chunk-based generation)
- Sound (terminal bell or external)
