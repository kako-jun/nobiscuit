# nobiscuit - Architecture

## Project Structure

Raycasting primitives live in the external [termray](https://github.com/kako-jun/termray)
crate. This repo only contains the game-specific code.

```
crates/
└── nobiscuit-cli/           # Game binary
    └── src/
        ├── main.rs          # Game loop (30fps: input → update → render → present)
        ├── terminal.rs      # Half-block ANSI renderer with delta flushing
        ├── input.rs         # Non-blocking crossterm key polling
        ├── maze.rs          # Mask-based irregular maze generation (per-island DFS)
        ├── player.rs        # Grid-based movement with animation interpolation
        ├── minimap.rs       # Semi-transparent 2D map overlay
        ├── game.rs          # Game state, World (multi-floor), hunger, pickups, stairs
        ├── ui.rs            # HUD (hunger bar, floor indicator, bitmap font messages)
        ├── tiles.rs         # Nobiscuit tile IDs (3..=11 — termray reserves 0..=2)
        ├── nob_map.rs       # NobiscuitMap: TileMap impl with nobiscuit-aware is_solid
        └── textures.rs      # WallTexturer/FloorTexturer/SpriteArt (fusuma/shoji/tatami)
```

termray supplies: `Camera`, `Framebuffer`, `Color`, `Sprite`, `TileMap` trait, `HeightMap`
trait (+ `FlatHeightMap` for tile-flat worlds), `HitSide`, `HitFace`, `RayHit`, plus the
render skeletons `render_walls`, `render_floor_ceiling`, `project_sprites`, `render_sprites`.
nobiscuit plugs its visuals into those skeletons via the trait implementations in
`textures.rs`, and passes `&FlatHeightMap` to keep its floors / ceilings flat.

## Tech Stack

- **Rust** (edition 2024, MSRV 1.85.0)
- **termray** — Generic TUI raycasting engine (extracted from the former `nobiscuit-engine`)
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

termray は `&dyn TileMap` で動作。nobiscuit は自前の `NobiscuitMap`（迷路生成器の出力）を渡す。
`NobiscuitMap::is_solid` は nobiscuit固有ルールを持つ — EMPTY/GOAL/STAIRS_UP/STAIRS_DOWN は歩ける、
WINDOW/SHOJI/DOORS は実体あり。

```rust
pub trait TileMap {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn get(&self, x: i32, y: i32) -> Option<TileType>;
    fn is_solid(&self, x: i32, y: i32) -> bool;
}
```

termray が予約するタイルIDは `0` EMPTY / `1` WALL / `2` VOID の3つのみ。nobiscuit は `3..=11` を自前で定義（`src/tiles.rs`）。

### Irregular Map Generation

迷路は不定形マスクベースで生成される:

1. **マスク生成**: 2-4 個のシード点から BFS でアメーバ状に拡張。全 DFS ノードの 40-70% を選択
2. **VOID 設定**: マスク外の内部セルを `TILE_VOID` に設定（外周は壁のまま）
3. **島検出**: マスク内の連結成分（島）を BFS で特定
4. **廊下骨格生成**: 各島の DFS ノードの 20-30% をランダムウォークで選び、2セル幅の廊下骨格を生成。直交方向の隣接セルも掘ることで幅を確保
5. **部屋配置**: マスク内に 2x2〜5x4 の部屋を廊下隣接優先で配置（重複禁止、試行120回）
6. **島ごと DFS**: 各島で独立に迷路を生成。部屋内ノードと廊下ノードは pre-carved として扱い、残りの細い通路で接続
7. **ドア配置**: 部屋外周の壁セルで反対側が廊下のものを候補にドアを配置。部屋サイズで種類決定。廊下ハブ構造を実現
8. **VOID 境界封止**: 歩行可能セルに隣接する VOID セルを WALL に変換。マスク境界で VOID が直接見える黒線バグを防止
9. **階段配置**: 廊下にのみ配置（部屋内は除外）。各島に階段を配置し、異なる階の異なる島に遷移させることで迷子感を演出
10. **レイキャスティング**: VOID に ray が到達すると `Some(RayHit{tile: TILE_VOID})` を返す。壁描画なし・床天井も抑制され、列全体が黒に

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
