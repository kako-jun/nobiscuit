# nobiscuit

TUI raycasting 3D maze game. DOOM-style first-person perspective rendered entirely in the terminal using half-block characters and 24-bit true color.

Inspired by Doraemon's "Home Maze" episode — wander through an endless house, eat biscuits to survive, find the exit.

## Features

- Wizardry-style grid movement with smooth animation
- Procedural wall textures (wood grain, nageshi, baseboard)
- Perspective-correct floor and ceiling rendering
- Multi-floor maze (3 floors connected by stairs)
- Window tiles with glass pane texture
- Biscuit sprites with hunger system
- Semi-transparent minimap overlay
- Floor indicator HUD

## Install

```bash
cargo install --path crates/nobiscuit-cli
```

Or build from source:

```bash
cargo build --release
./target/release/nobiscuit
```

## Controls

| Key | Action |
|---|---|
| W / Up | Move forward |
| S / Down | Move backward |
| A / Left | Turn left |
| D / Right | Turn right |
| M | Toggle minimap |
| Q / Esc | Quit |

## Requirements

- Terminal with 24-bit true color support (most modern terminals)
- Recommended size: 80x24 or larger

## Architecture

nobiscuit is a thin binary on top of [`termray`](https://github.com/kako-jun/termray),
a reusable TUI raycasting engine. termray provides the rendering skeleton
(ray casting, wall / floor / sprite drawing) and nobiscuit supplies the
Japanese-house textures (fusuma, shoji, window, doors) and tatami floor via
`WallTexturer` / `FloorTexturer` / `SpriteArt` trait implementations.

```
termray (external crate)
  └── nobiscuit (this repo)
        ├── maze generation, player, HUD, minimap
        ├── NobiscuitMap (custom is_solid for goals/stairs/doors)
        └── NobiscuitTextures (fusuma, shoji, tatami, biscuit sprites)
```

The engine renders to an abstract framebuffer. The CLI converts it to half-block characters (`▀`) where each terminal cell represents 2 vertical pixels using foreground/background colors.

## License

MIT
