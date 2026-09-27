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
        ├── maze.rs          # Non-overlapping BSP house plan and region connection graph
        ├── player.rs        # Grid-based movement with animation interpolation
        ├── minimap.rs       # Semi-transparent 2D map overlay
        ├── game.rs          # Game state, World (multi-floor), hunger, pickups, stairs
        ├── ui.rs            # HUD (hunger bar, floor indicator, bitmap font messages)
        ├── tiles.rs         # Nobiscuit tile IDs (3..=12 — termray reserves 0..=2)
        ├── nobiscuit_map.rs       # NobiscuitMap: TileMap impl with nobiscuit-aware is_solid
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
`NobiscuitMap::is_solid` は nobiscuit固有ルールを持つ — EMPTY/GOAL/STAIRS_UP/STAIRS_DOWN/WINDOW_PASS は歩ける、
WINDOW/SHOJI/DOORS は実体あり。

```rust
pub trait TileMap {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn get(&self, x: i32, y: i32) -> Option<TileType>;
    fn is_solid(&self, x: i32, y: i32) -> bool;
}
```

termray が予約するタイルIDは `0` EMPTY / `1` WALL / `2` VOID の3つのみ。nobiscuit は `3..=12` を自前で定義（`src/tiles.rs`）。

### House Plan Generation

通常階は重複しない1つのBSP間取りとして生成する。島の外接矩形を重ねて彫る方式は使わない。

1. **主廊下の予約**: 長辺を分割し、幅3セルの長い廊下を壁2枚の間に確保。25×19以上では長さ9以上、左右各2室以上が面する。最小マップも収まる範囲で縮小構成にする。
2. **部屋分割**: 両側の領域を内寸3〜9セルになるまで再帰分割。固定深度で打ち切らない。各セルの領域所有者を記録し、床の重複を禁止する。
3. **接続候補**: 壁1枚の両側にある領域のペアごとに候補を集約。角・T字接合を避け、開口の両脇に壁を残す。同じペアには1開口だけ置く。
4. **接続グラフ**: 主廊下に面する各室へ入口を確保し、全域木で残りを接続。残りの領域ペアの約15%をループにし、可能なら部屋間の直接接続を最低2つ確保する。
5. **扉と窓**: 接続を増やす後処理は行わず、選ばれた接続に扉の種類を割り当てる。部屋間接続の最初を通り抜け窓、次を扉、それ以降の一部を窓にする。残った壁にはsolidな装飾窓・障子を配置する。
6. **階段**: 通常階の廊下に上り・下りを各1つ（必要な方向のみ）配置。接続が最初から連結しているため再生成や未到達セル壁化に頼らない。単一始点のflood fillで検証する。
7. **ゴール階**: 最上階は固定テンプレート（下り階段→縦廊下→ふすま→のび太の部屋）。周囲VOIDと歩行空間の境界は壁で封止する。

通り抜け窓は衝突・通常rayでは非solid。`windows.rs` が共有壁の中央面にある窓枠を列ごとに追跡し、奥行きを持つ画素として合成する。複数の窓枠・壁・spriteの前後関係を保ち、開口の中央からは隣室が見える。装飾窓は従来通りsolidである。

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
