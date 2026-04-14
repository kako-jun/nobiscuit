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
| math.rs | Vec2f, angle normalization, fisheye correction |
| ray.rs | DDA raycasting algorithm |
| map.rs | TileMap trait + GridMap implementation |
| camera.rs | Camera position/angle, cast_all_rays |
| renderer.rs | Wall column rendering |
| floor.rs | Floor/ceiling gradient shading |
| framebuffer.rs | Color + pixel buffer |
| sprite.rs | Sprite projection (stub) |

### CLI (crates/nobiscuit-cli/src/)
| File | Purpose |
|---|---|
| main.rs | Game loop (input → update → render → present) |
| terminal.rs | Half-block ANSI renderer with delta flushing |
| input.rs | Non-blocking crossterm key polling |
| maze.rs | DFS maze generation (iterative backtracking) |
| player.rs | Movement + per-axis collision detection |
| minimap.rs | 2D map overlay |
| game.rs | Game state (stub for hunger/biscuits) |
| scene.rs | Scene management (stub) |
| ui.rs | HUD (stub) |

## Future work (Issue #2)

- Hunger system (biscuits, starvation)
- Sprite rendering (biscuits as ASCII art scaled by distance)
- Infinite maze (chunk-based generation)
- Text on walls (readable when close)
- Title/ending scenes
- Sound (terminal bell or external)
