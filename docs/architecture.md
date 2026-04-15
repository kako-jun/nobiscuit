# nobiscuit - Architecture

## Project Structure

```
crates/
├── nobiscuit-engine/        # Reusable raycasting engine (zero external deps)
│   └── src/
│       ├── lib.rs           # Public API re-exports
│       ├── math.rs          # Vec2f, angle normalization, fisheye correction
│       ├── ray.rs           # DDA raycasting algorithm
│       ├── map.rs           # TileMap trait + GridMap implementation
│       ├── camera.rs        # Camera position/angle, cast_all_rays
│       ├── renderer.rs      # Wall column rendering with procedural textures
│       ├── floor.rs         # Floor/ceiling with perspective-correct patterns
│       ├── sprite.rs        # Sprite projection + AA art rendering
│       └── framebuffer.rs   # Color + pixel buffer + alpha blending
│
└── nobiscuit-cli/           # Game binary
    └── src/
        ├── main.rs          # Game loop (30fps: input → update → render → present)
        ├── terminal.rs      # Half-block ANSI renderer with delta flushing
        ├── input.rs         # Non-blocking crossterm key polling
        ├── maze.rs          # Mask-based irregular maze generation (per-island DFS)
        ├── player.rs        # Grid-based movement with animation interpolation
        ├── minimap.rs       # Semi-transparent 2D map overlay
        ├── game.rs          # Game state, World (multi-floor), hunger, pickups, stairs
        └── ui.rs            # HUD (hunger bar, floor indicator, bitmap font messages)
```

## Tech Stack

- **Rust** (edition 2021)
- **crossterm** 0.28 — Terminal rendering, raw mode, key input
- **rand** 0.8 — Maze generation, biscuit placement

## Key Concepts

### Framebuffer

Engine は `Color` ピクセルを width x height のバッファに書き込む。
height = ターミナル行数 x 2（ハーフブロック `▀` で垂直解像度が倍）。

```
Terminal: 80 cols x 24 rows
→ Framebuffer: 80 x 48 pixels
→ Half-block: each cell = 2 vertical pixels (fg color + bg color)
```

### DDA Raycasting

1 列に 1 本の ray をグリッド上で飛ばし、壁に当たるまで DDA（Digital Differential Analysis）で走査。

- `RayHit` に距離・面（東西/南北）・タイル座標・`wall_x`（壁面上の水平位置 0.0..1.0）を記録
- Fisheye 補正は `camera.rs` で適用

### Delta Flushing

ターミナルレンダラーはダブルバッファ方式。前フレームと比較し、変更があったセルだけ ANSI エスケープシーケンスを出力。30fps 維持に必須。

### Grid Movement

- 位置はタイル中央（x.5, y.5）に固定
- 向きは 4 方向（東/南/西/北）、90 度ずつ
- 移動・回転はイージング付きアニメーション補間
- アニメーション中は新しい入力を受け付けない

### TileMap Trait

Engine は `&dyn TileMap` で動作。CLI 側が `GridMap`（迷路生成器の出力）を渡す。

```rust
pub trait TileMap {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn get(&self, x: i32, y: i32) -> Option<TileType>;
    fn is_solid(&self, x: i32, y: i32) -> bool;
}
```

### Irregular Map Generation

迷路は不定形マスクベースで生成される:

1. **マスク生成**: 2-4 個のシード点から BFS でアメーバ状に拡張。全 DFS ノードの 40-70% を選択
2. **VOID 設定**: マスク外の内部セルを `TILE_VOID` に設定（外周は壁のまま）
3. **島検出**: マスク内の連結成分（島）を BFS で特定
4. **島ごと DFS**: 各島で独立に迷路を生成。島間は同一階では移動不可
5. **階段配置**: 各島に階段を配置し、異なる階の異なる島に遷移させることで迷子感を演出
6. **レイキャスティング**: VOID に ray が到達すると `Some(RayHit{tile: TILE_VOID})` を返す。壁描画なし・床天井も抑制され、列全体が黒に

### Sprite System

1. `project_sprites`: ワールド座標 → スクリーン座標に投影。FOV カリング、距離ソート
2. `render_sprites`: AA パターンをスクリーン上にスケーリング描画。壁との深度テスト付き
3. パターン文字: `#` = 不透明、`+` = 影/ハイライト、`.` = 透明

### Alpha Blending

`Framebuffer::blend_pixel` で既存ピクセルと新しい色をアルファブレンド。ミニマップの半透明オーバーレイに使用。

## Data Flow

```
Input (crossterm)
  → Player (grid move / turn)
    → Camera (position + angle)
      → Ray casting (DDA per column)
        → Floor/Ceiling renderer (perspective-correct world coords)
        → Wall renderer (procedural texture)
        → Sprite renderer (AA art + depth test)
          → Minimap overlay (alpha blend)
            → HUD (hunger bar, messages)
              → Terminal renderer (delta flush)
                → ANSI half-block output
```

## Performance

- **ターゲット**: 30fps
- **フレーム時間**: 33ms
- **最大描画深度**: 20.0 world units
- **ボトルネック**: 床・天井のピクセルごとの座標計算（列×行のループ）
- **最適化**: デルタフラッシュで ANSI 出力を最小化
