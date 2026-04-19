# nobiscuit - Specification

## Overview

nobiscuit は TUI レイキャスティング 3D 迷路ゲーム。Rust + crossterm で実装。
ハーフブロック文字（`▀`）と 24bit True Color でターミナル内に DOOM 風の一人称 3D を描画する。

## Requirements

- Terminal with 24-bit true color support (iTerm2, WezTerm, Alacritty, Windows Terminal, etc.)
- Recommended size: 80x24 or larger

## Install

```bash
# From source
cargo build --release
./target/release/nobiscuit

# Or install to PATH
cargo install --path crates/nobiscuit-cli
```

## Crate Structure

Single-crate workspace; raycasting primitives come from the external `termray` crate.

| Crate | Type | Dependencies | Purpose |
|---|---|---|---|
| termray (external) | lib | none | Generic raycasting engine, framebuffer, sprite system |
| nobiscuit-cli | bin | termray, crossterm, rand | Game binary (maze, HUD, Japanese-house textures) |

## Engine API (from termray)

### Framebuffer

```rust
Framebuffer::new(width, height) -> Framebuffer
fb.clear(color)
fb.set_pixel(x, y, color)
fb.get_pixel(x, y) -> Color
fb.blend_pixel(x, y, color, alpha)  // alpha blending
```

### Raycasting

```rust
cast_ray(map, origin, angle, max_depth) -> Option<RayHit>
camera.cast_all_rays(map, num_rays, max_depth) -> Vec<Option<RayHit>>
```

### Rendering

termray rendering functions take pluggable `WallTexturer` / `FloorTexturer` / `SpriteArt`
trait objects instead of hard-coded styling. nobiscuit implements them on its
`NobiscuitTextures` struct in `src/textures.rs`.

```rust
// termray 0.3: wall/floor renderers take a HeightMap + Camera.
// nobiscuit is a tile-flat world, so we pass &FlatHeightMap everywhere.
render_walls(fb, rays, &wall_texturer, &FlatHeightMap, camera, max_depth)
render_floor_ceiling(fb, rays, &floor_texturer, &FlatHeightMap, camera, max_depth)
project_sprites(sprites, camera, &FlatHeightMap, screen_width, screen_height) -> Vec<SpriteRenderResult>
render_sprites(fb, projected, rays, &sprite_art, max_depth)
```

### Map

termray reserves three tile IDs (0 EMPTY / 1 WALL / 2 VOID). Anything else is
user-defined; nobiscuit uses 3..=11 for its Japanese-house tiles
(see `crates/nobiscuit-cli/src/tiles.rs`).

```rust
trait TileMap {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn get(&self, x: i32, y: i32) -> Option<TileType>;
    fn is_solid(&self, x: i32, y: i32) -> bool;
}

// termray-reserved
const TILE_EMPTY: u8 = 0;
const TILE_WALL:  u8 = 1;
const TILE_VOID:  u8 = 2;
// nobiscuit-specific (src/tiles.rs)
const TILE_GOAL:         u8 = 3;
const TILE_WINDOW:       u8 = 4;
const TILE_STAIRS_UP:    u8 = 5;
const TILE_STAIRS_DOWN:  u8 = 6;
const TILE_DOOR_FUSUMA:  u8 = 7;
const TILE_DOOR_KITCHEN: u8 = 8;
const TILE_DOOR_TOILET:  u8 = 9;
const TILE_DOOR_GENKAN:  u8 = 10;
const TILE_SHOJI:        u8 = 11;
```

## Game Parameters

| Parameter | Value | Description |
|---|---|---|
| Target FPS | 30 | Frame rate |
| Max depth | 20.0 | Maximum ray distance |
| Maze size | 15x13 ~ 121x91 | Determined by galagala spins |
| Num floors | 1 ~ 12 | Determined by galagala spins |
| FOV | 60° (π/3) | Field of view |
| Move speed | 4.0 | Grid moves per second (animation) |
| Turn speed | 6.0 | 90° turns per second (animation) |
| Hunger drain | 0.02/sec | ~50 seconds to starve |
| Biscuit restore | 0.15 | Hunger restored per biscuit |
| Biscuit density | ~30% | Fraction of empty tiles with biscuits |
| Pickup distance | 0.5 | World units to pick up item |
| Minimap scale | 2 | Pixels per map tile |
| Minimap alpha | 0.4 | Overlay transparency |
| Mask coverage | 40-70% | Fraction of DFS nodes included in irregular mask |
| Seed points | 2-4 | Number of BFS seed points for mask generation |
| Room sizes | 2x2 ~ 5x4 | Interior dimensions of placed rooms |
| Room attempts | 120 | Maximum placement attempts per floor |
| Corridor ratio | 20-30% | Fraction of island DFS nodes used as corridor backbone |
| Corridor width | 2 cells | Main corridors are widened to 2 cells |

## Tile Types

| Value | Constant | Solid | Description |
|---|---|---|---|
| 0 | TILE_EMPTY | No | Walkable floor (termray-reserved) |
| 1 | TILE_WALL | Yes | Solid wall (termray-reserved) |
| 2 | TILE_VOID | Yes* | Non-existent cell (not wall, not floor). Rays return `Some(RayHit{tile: TILE_VOID})` — no wall, floor, or ceiling drawn (column stays black). termray-reserved |
| 3 | TILE_GOAL | No | Exit marker |
| 4 | TILE_WINDOW | Yes | Glass pane with wooden frame. Upper/lower 15% are wall texture |
| 5 | TILE_STAIRS_UP | No | Stairs to upper floor |
| 6 | TILE_STAIRS_DOWN | No | Stairs to lower floor |
| 7 | TILE_DOOR_FUSUMA | Yes** | Fusuma (sliding paper door). White washi + metal pull |
| 8 | TILE_DOOR_KITCHEN | Yes** | Kitchen door. Wood grain + doorknob |
| 9 | TILE_DOOR_TOILET | Yes** | Toilet door. Dark wood + frosted glass window |
| 10 | TILE_DOOR_GENKAN | Yes** | Entrance door. Heavy dark wood + panel grooves |
| 11 | TILE_SHOJI | Yes | Shoji screen. Wooden lattice + white washi paper. Upper 20% and lower 30% are wall texture |

\* VOID is solid for movement (impassable). Raycasting returns a special hit that suppresses all rendering for that column.

\*\* Doors are solid when closed. Auto-open when player is adjacent, auto-close when player moves away (manhattan distance >= 3).

## Sprite Types

| Value | Constant | Art | Height Scale | Placement |
|---|---|---|---|---|
| 1 | SPRITE_BISCUIT | Round cookie (8x7 AA) | 0.25 | Floor level |
| 2 | SPRITE_GOAL | Sphere with highlight (8x7 AA) | 0.25 | Floating |
| 3 | SPRITE_STAIRS_UP | Upward arrow (6x6 AA) | 0.3 | Floor level |
| 4 | SPRITE_STAIRS_DOWN | Downward arrow (6x6 AA) | 0.3 | Floor level |

## Wall Texture Features

| Feature | Position | Effect |
|---|---|---|
| Panel grooves | wall_x < 0.04 or > 0.96 | Brightness x 0.55 |
| Groove transition | wall_x < 0.08 or > 0.92 | Brightness x 0.75 |
| Nageshi (長押) | wall_y ≈ 0.35 | Horizontal dark rail |
| Baseboard (幅木) | wall_y > 0.88 | Dark band at bottom |
| Wood grain | Varies by tile hash | Sinusoidal vertical stripes |
| Plank lines | wall_y thirds | Faint horizontal divisions |
| Hue variation | Per tile hash | ±4.5 color shift |
