//! Nobiscuit-specific wall / floor / sprite texturing.
//!
//! Plugs into termray via the [`WallTexturer`], [`FloorTexturer`] and
//! [`SpriteArt`] traits.

use termray::{Color, FloorTexturer, HitSide, SpriteArt, SpriteDef, TileType, WallTexturer};

use crate::game::{SPRITE_BISCUIT, SPRITE_GOAL, SPRITE_STAIRS_DOWN, SPRITE_STAIRS_UP};
use crate::tiles::{
    TILE_DOOR_FUSUMA, TILE_DOOR_GENKAN, TILE_DOOR_KITCHEN, TILE_DOOR_TOILET, TILE_SHOJI,
    TILE_WINDOW,
};

pub struct NobiscuitTextures;

// ---------------- Wall texturing ----------------

fn wall_texture(wall_x: f64, wall_y: f64, side: HitSide, brightness: f64, tile_hash: u32) -> Color {
    let (base_r, base_g, base_b) = match side {
        HitSide::Vertical => (180.0, 140.0, 100.0),
        HitSide::Horizontal => (160.0, 125.0, 85.0),
    };
    let hue_shift = ((tile_hash % 30) as f64 - 15.0) * 0.3;

    let groove = if !(0.04..=0.96).contains(&wall_x) {
        0.55
    } else if !(0.08..=0.92).contains(&wall_x) {
        0.75
    } else {
        1.0
    };
    let nageshi = if (wall_y - 0.35).abs() < 0.015 {
        0.6
    } else if (wall_y - 0.35).abs() < 0.03 {
        0.8
    } else {
        1.0
    };
    let baseboard = if wall_y > 0.92 {
        0.65
    } else if wall_y > 0.88 {
        0.8
    } else {
        1.0
    };
    let grain_freq = 12.0 + (tile_hash % 5) as f64;
    let grain = 0.92 + 0.08 * (wall_x * grain_freq * std::f64::consts::PI).sin();
    let plank = if (wall_y * 3.0).fract() < 0.02 && wall_y > 0.05 && wall_y < 0.85 {
        0.85
    } else {
        1.0
    };

    let detail = groove * nageshi * baseboard * grain * plank;
    let r = ((base_r + hue_shift) * brightness * detail).clamp(0.0, 255.0) as u8;
    let g = ((base_g + hue_shift * 0.5) * brightness * detail).clamp(0.0, 255.0) as u8;
    let b = ((base_b - hue_shift * 0.3) * brightness * detail).clamp(0.0, 255.0) as u8;
    Color::rgb(r, g, b)
}

fn window_texture(
    wall_x: f64,
    wall_y: f64,
    side: HitSide,
    brightness: f64,
    tile_hash: u32,
) -> Color {
    if !(0.15..=0.85).contains(&wall_y) {
        return wall_texture(wall_x, wall_y, side, brightness, tile_hash);
    }
    let wy = (wall_y - 0.15) / 0.70;
    let frame_thickness = 0.12;
    let mullion_half = 0.02;

    let in_frame = wall_x < frame_thickness
        || wall_x > 1.0 - frame_thickness
        || wy < frame_thickness
        || wy > 1.0 - frame_thickness;
    let on_mullion = (wall_x - 0.5).abs() < mullion_half || (wy - 0.5).abs() < mullion_half;

    if in_frame || on_mullion {
        let base = match side {
            HitSide::Vertical => (120.0, 80.0, 50.0),
            HitSide::Horizontal => (105.0, 70.0, 42.0),
        };
        let r = (base.0 * brightness).clamp(0.0, 255.0) as u8;
        let g = (base.1 * brightness).clamp(0.0, 255.0) as u8;
        let b = (base.2 * brightness).clamp(0.0, 255.0) as u8;
        Color::rgb(r, g, b)
    } else {
        let hue_shift = ((tile_hash % 20) as f64 - 10.0) * 0.2;
        let (base_r, base_g, base_b) = (140.0, 180.0, 220.0);
        let cx = if wall_x < 0.5 {
            (wall_x - frame_thickness) / (0.5 - frame_thickness - mullion_half)
        } else {
            (1.0 - frame_thickness - wall_x) / (0.5 - frame_thickness - mullion_half)
        };
        let cy = if wy < 0.5 {
            (wy - frame_thickness) / (0.5 - frame_thickness - mullion_half)
        } else {
            (1.0 - frame_thickness - wy) / (0.5 - frame_thickness - mullion_half)
        };
        let center_glow = 1.0 + (cx.clamp(0.0, 1.0) * cy.clamp(0.0, 1.0)) * 0.15;
        let r = ((base_r + hue_shift) * brightness * center_glow).clamp(0.0, 255.0) as u8;
        let g = ((base_g + hue_shift * 0.3) * brightness * center_glow).clamp(0.0, 255.0) as u8;
        let b = ((base_b - hue_shift * 0.2) * brightness * center_glow).clamp(0.0, 255.0) as u8;
        Color::rgb(r, g, b)
    }
}

fn shoji_texture(
    wall_x: f64,
    wall_y: f64,
    side: HitSide,
    brightness: f64,
    tile_hash: u32,
) -> Color {
    if !(0.20..=0.70).contains(&wall_y) {
        return wall_texture(wall_x, wall_y, side, brightness, tile_hash);
    }
    let side_factor = match side {
        HitSide::Vertical => 1.0,
        HitSide::Horizontal => 0.88,
    };
    let sy = (wall_y - 0.20) / 0.50;
    let san_half = 0.015;
    let on_vertical_san = (wall_x - 0.25).abs() < san_half
        || (wall_x - 0.50).abs() < san_half
        || (wall_x - 0.75).abs() < san_half;
    let on_horizontal_san = (sy - 0.20).abs() < san_half * 1.5
        || (sy - 0.40).abs() < san_half * 1.5
        || (sy - 0.60).abs() < san_half * 1.5
        || (sy - 0.80).abs() < san_half * 1.5;
    let on_frame = !(0.04..=0.96).contains(&wall_x) || !(0.04..=0.96).contains(&sy);

    if on_frame || on_vertical_san || on_horizontal_san {
        let hue = ((tile_hash % 10) as f64 - 5.0) * 0.5;
        let (br, bg, bb) = (90.0 + hue, 60.0 + hue * 0.3, 35.0);
        let r = (br * brightness * side_factor).clamp(0.0, 255.0) as u8;
        let g = (bg * brightness * side_factor).clamp(0.0, 255.0) as u8;
        let b = (bb * brightness * side_factor).clamp(0.0, 255.0) as u8;
        Color::rgb(r, g, b)
    } else {
        let (base_r, base_g, base_b) = (240.0, 235.0, 220.0);
        let warmth = ((tile_hash % 15) as f64 - 7.0) * 0.4;
        let glow = 0.95 + 0.05 * ((wall_x * 3.0 + sy * 2.0) * std::f64::consts::PI).sin();
        let r = ((base_r + warmth) * brightness * side_factor * glow).clamp(0.0, 255.0) as u8;
        let g = ((base_g + warmth * 0.5) * brightness * side_factor * glow).clamp(0.0, 255.0) as u8;
        let b = ((base_b - warmth * 0.3) * brightness * side_factor * glow).clamp(0.0, 255.0) as u8;
        Color::rgb(r, g, b)
    }
}

fn fusuma_texture(
    wall_x: f64,
    wall_y: f64,
    side: HitSide,
    brightness: f64,
    _tile_hash: u32,
) -> Color {
    let side_factor = match side {
        HitSide::Vertical => 1.0,
        HitSide::Horizontal => 0.85,
    };
    if !(0.06..=0.94).contains(&wall_x) {
        let (br, bg, bb) = (100.0, 70.0, 40.0);
        let r = (br * brightness * side_factor).clamp(0.0, 255.0) as u8;
        let g = (bg * brightness * side_factor).clamp(0.0, 255.0) as u8;
        let b = (bb * brightness * side_factor).clamp(0.0, 255.0) as u8;
        return Color::rgb(r, g, b);
    }
    if (0.47..=0.53).contains(&wall_x) && (0.45..=0.55).contains(&wall_y) {
        let (br, bg, bb) = (180.0, 150.0, 50.0);
        let r = (br * brightness * side_factor).clamp(0.0, 255.0) as u8;
        let g = (bg * brightness * side_factor).clamp(0.0, 255.0) as u8;
        let b = (bb * brightness * side_factor).clamp(0.0, 255.0) as u8;
        return Color::rgb(r, g, b);
    }
    let (br, bg, bb) = (230.0, 220.0, 200.0);
    let r = (br * brightness * side_factor).clamp(0.0, 255.0) as u8;
    let g = (bg * brightness * side_factor).clamp(0.0, 255.0) as u8;
    let b = (bb * brightness * side_factor).clamp(0.0, 255.0) as u8;
    Color::rgb(r, g, b)
}

fn kitchen_door_texture(
    wall_x: f64,
    wall_y: f64,
    side: HitSide,
    brightness: f64,
    tile_hash: u32,
) -> Color {
    let side_factor = match side {
        HitSide::Vertical => 1.0,
        HitSide::Horizontal => 0.85,
    };
    if !(0.05..=0.95).contains(&wall_y) {
        let (br, bg, bb) = (110.0, 80.0, 50.0);
        let r = (br * brightness * side_factor).clamp(0.0, 255.0) as u8;
        let g = (bg * brightness * side_factor).clamp(0.0, 255.0) as u8;
        let b = (bb * brightness * side_factor).clamp(0.0, 255.0) as u8;
        return Color::rgb(r, g, b);
    }
    if (0.78..=0.85).contains(&wall_x) && (0.43..=0.50).contains(&wall_y) {
        let (br, bg, bb) = (180.0, 180.0, 170.0);
        let r = (br * brightness * side_factor).clamp(0.0, 255.0) as u8;
        let g = (bg * brightness * side_factor).clamp(0.0, 255.0) as u8;
        let b = (bb * brightness * side_factor).clamp(0.0, 255.0) as u8;
        return Color::rgb(r, g, b);
    }
    let (base_r, base_g, base_b) = (160.0, 120.0, 80.0);
    let grain_freq = 12.0 + (tile_hash % 5) as f64;
    let grain = 0.92 + 0.08 * (wall_x * grain_freq * std::f64::consts::PI).sin();
    let r = (base_r * brightness * side_factor * grain).clamp(0.0, 255.0) as u8;
    let g = (base_g * brightness * side_factor * grain).clamp(0.0, 255.0) as u8;
    let b = (base_b * brightness * side_factor * grain).clamp(0.0, 255.0) as u8;
    Color::rgb(r, g, b)
}

fn toilet_door_texture(
    wall_x: f64,
    wall_y: f64,
    side: HitSide,
    brightness: f64,
    tile_hash: u32,
) -> Color {
    let side_factor = match side {
        HitSide::Vertical => 1.0,
        HitSide::Horizontal => 0.85,
    };
    if !(0.05..=0.95).contains(&wall_y) {
        let (br, bg, bb) = (90.0, 65.0, 35.0);
        let r = (br * brightness * side_factor).clamp(0.0, 255.0) as u8;
        let g = (bg * brightness * side_factor).clamp(0.0, 255.0) as u8;
        let b = (bb * brightness * side_factor).clamp(0.0, 255.0) as u8;
        return Color::rgb(r, g, b);
    }
    if (0.35..=0.65).contains(&wall_x) && (0.15..=0.30).contains(&wall_y) {
        let (br, bg, bb) = (200.0, 210.0, 220.0);
        let r = (br * brightness * side_factor).clamp(0.0, 255.0) as u8;
        let g = (bg * brightness * side_factor).clamp(0.0, 255.0) as u8;
        let b = (bb * brightness * side_factor).clamp(0.0, 255.0) as u8;
        return Color::rgb(r, g, b);
    }
    let (base_r, base_g, base_b) = (140.0, 110.0, 70.0);
    let grain_freq = 12.0 + (tile_hash % 5) as f64;
    let grain = 0.92 + 0.08 * (wall_x * grain_freq * std::f64::consts::PI).sin();
    let r = (base_r * brightness * side_factor * grain).clamp(0.0, 255.0) as u8;
    let g = (base_g * brightness * side_factor * grain).clamp(0.0, 255.0) as u8;
    let b = (base_b * brightness * side_factor * grain).clamp(0.0, 255.0) as u8;
    Color::rgb(r, g, b)
}

fn genkan_door_texture(
    wall_x: f64,
    wall_y: f64,
    side: HitSide,
    brightness: f64,
    _tile_hash: u32,
) -> Color {
    let side_factor = match side {
        HitSide::Vertical => 1.0,
        HitSide::Horizontal => 0.85,
    };
    if !(0.07..=0.93).contains(&wall_y) {
        let (br, bg, bb) = (60.0, 40.0, 20.0);
        let r = (br * brightness * side_factor).clamp(0.0, 255.0) as u8;
        let g = (bg * brightness * side_factor).clamp(0.0, 255.0) as u8;
        let b = (bb * brightness * side_factor).clamp(0.0, 255.0) as u8;
        return Color::rgb(r, g, b);
    }
    let groove = if (wall_x - 0.33).abs() < 0.015 || (wall_x - 0.66).abs() < 0.015 {
        0.6
    } else {
        1.0
    };
    let (base_r, base_g, base_b) = (100.0, 70.0, 40.0);
    let r = (base_r * brightness * side_factor * groove).clamp(0.0, 255.0) as u8;
    let g = (base_g * brightness * side_factor * groove).clamp(0.0, 255.0) as u8;
    let b = (base_b * brightness * side_factor * groove).clamp(0.0, 255.0) as u8;
    Color::rgb(r, g, b)
}

impl WallTexturer for NobiscuitTextures {
    fn sample_wall(
        &self,
        tile: TileType,
        wall_x: f64,
        wall_y: f64,
        side: HitSide,
        brightness: f64,
        tile_hash: u32,
    ) -> Color {
        match tile {
            TILE_WINDOW => window_texture(wall_x, wall_y, side, brightness, tile_hash),
            TILE_SHOJI => shoji_texture(wall_x, wall_y, side, brightness, tile_hash),
            TILE_DOOR_FUSUMA => fusuma_texture(wall_x, wall_y, side, brightness, tile_hash),
            TILE_DOOR_KITCHEN => kitchen_door_texture(wall_x, wall_y, side, brightness, tile_hash),
            TILE_DOOR_TOILET => toilet_door_texture(wall_x, wall_y, side, brightness, tile_hash),
            TILE_DOOR_GENKAN => genkan_door_texture(wall_x, wall_y, side, brightness, tile_hash),
            _ => wall_texture(wall_x, wall_y, side, brightness, tile_hash),
        }
    }
}

// ---------------- Floor / ceiling texturing ----------------

fn floor_tile_color(wx: f64, wy: f64, base: Color, brightness: f64) -> Color {
    let tile_size = 1.0;
    let tx = (wx / tile_size).fract().abs();
    let ty = (wy / tile_size).fract().abs();
    let grid_width = 0.04;
    let on_grid =
        tx < grid_width || tx > (1.0 - grid_width) || ty < grid_width || ty > (1.0 - grid_width);
    let plank_line = (ty * 6.0).fract() < 0.08;
    let tile_ix = wx.floor() as u32;
    let tile_iy = wy.floor() as u32;
    let hash = tile_ix.wrapping_mul(2654435761) ^ tile_iy.wrapping_mul(2246822519);
    let variation = 0.9 + 0.1 * ((hash % 100) as f64 / 100.0);
    if on_grid {
        base.darken(brightness * 0.4)
    } else if plank_line {
        base.darken(brightness * variation * 0.75)
    } else {
        base.darken(brightness * variation)
    }
}

fn ceiling_tile_color(wx: f64, wy: f64, base: Color, brightness: f64) -> Color {
    let tx = (wx * 0.5).fract().abs();
    let ty = (wy * 0.5).fract().abs();
    let on_grid = !(0.02..=0.98).contains(&tx) || !(0.02..=0.98).contains(&ty);
    if on_grid {
        base.darken(brightness * 0.7)
    } else {
        base.darken(brightness)
    }
}

const FLOOR_BASE: Color = Color {
    r: 74,
    g: 60,
    b: 40,
};
const CEILING_BASE: Color = Color {
    r: 135,
    g: 206,
    b: 235,
};

impl FloorTexturer for NobiscuitTextures {
    fn sample_floor(&self, world_x: f64, world_y: f64, brightness: f64) -> Color {
        floor_tile_color(world_x, world_y, FLOOR_BASE, brightness)
    }

    fn sample_ceiling(&self, world_x: f64, world_y: f64, brightness: f64) -> Color {
        ceiling_tile_color(world_x, world_y, CEILING_BASE, brightness)
    }
}

// ---------------- Sprite art ----------------

static BISCUIT_DEF: SpriteDef = SpriteDef {
    pattern: &[
        "..####..", ".######.", "########", "##.##.##", "########", ".######.", "..####..",
    ],
    height_scale: 0.25,
    float_offset_scale: 0.0,
};

static GOAL_DEF: SpriteDef = SpriteDef {
    pattern: &[
        "..####..", ".######.", "###++###", "###+.###", "########", ".######.", "..####..",
    ],
    height_scale: 0.25,
    float_offset_scale: 1.0 / 3.0,
};

static STAIRS_UP_DEF: SpriteDef = SpriteDef {
    pattern: &["..##..", ".####.", "######", "..##..", "..##..", "..##.."],
    height_scale: 0.3,
    float_offset_scale: 0.0,
};

static STAIRS_DOWN_DEF: SpriteDef = SpriteDef {
    pattern: &["..##..", "..##..", "..##..", "######", ".####.", "..##.."],
    height_scale: 0.3,
    float_offset_scale: 0.0,
};

static FALLBACK_DEF: SpriteDef = SpriteDef {
    pattern: &["..##..", ".####.", "######", ".####.", "..##.."],
    height_scale: 0.25,
    float_offset_scale: 0.0,
};

impl SpriteArt for NobiscuitTextures {
    fn art(&self, sprite_type: u8) -> Option<&SpriteDef> {
        Some(match sprite_type {
            SPRITE_BISCUIT => &BISCUIT_DEF,
            SPRITE_GOAL => &GOAL_DEF,
            SPRITE_STAIRS_UP => &STAIRS_UP_DEF,
            SPRITE_STAIRS_DOWN => &STAIRS_DOWN_DEF,
            _ => &FALLBACK_DEF,
        })
    }

    fn color(&self, sprite_type: u8) -> Color {
        match sprite_type {
            SPRITE_BISCUIT => Color::rgb(220, 180, 80),
            SPRITE_GOAL => Color::rgb(50, 220, 50),
            SPRITE_STAIRS_UP => Color::rgb(200, 150, 50),
            SPRITE_STAIRS_DOWN => Color::rgb(150, 100, 30),
            _ => Color::rgb(255, 255, 255),
        }
    }
}
