use crate::framebuffer::{Color, Framebuffer};
use crate::map::{
    TILE_DOOR_FUSUMA, TILE_DOOR_GENKAN, TILE_DOOR_KITCHEN, TILE_DOOR_TOILET, TILE_SHOJI, TILE_VOID,
    TILE_WINDOW,
};
use crate::ray::{HitSide, RayHit};

/// Compute textured wall color for a given point on the wall surface.
///
/// `wall_x`: horizontal position on the wall tile (0.0..1.0)
/// `wall_y`: vertical position on the wall column (0.0 = top, 1.0 = bottom)
/// `side`: which face was hit (affects base color)
/// `brightness`: distance-based dimming (0.0..1.0)
/// `tile_hash`: hash derived from map coordinates for per-tile variation
fn wall_texture(wall_x: f64, wall_y: f64, side: HitSide, brightness: f64, tile_hash: u32) -> Color {
    // Base wall colors — slightly different per side for depth perception
    let (base_r, base_g, base_b) = match side {
        HitSide::Vertical => (180.0, 140.0, 100.0),
        HitSide::Horizontal => (160.0, 125.0, 85.0),
    };

    // Per-tile hue shift so not every wall looks identical
    let hue_shift = ((tile_hash % 30) as f64 - 15.0) * 0.3;

    // --- Vertical grooves (panel edges at tile boundaries) ---
    let groove = if !(0.04..=0.96).contains(&wall_x) {
        0.55
    } else if !(0.08..=0.92).contains(&wall_x) {
        0.75
    } else {
        1.0
    };

    // --- Horizontal features ---
    // Nageshi (長押) — a horizontal rail at ~35% from top
    let nageshi = if (wall_y - 0.35).abs() < 0.015 {
        0.6
    } else if (wall_y - 0.35).abs() < 0.03 {
        0.8
    } else {
        1.0
    };

    // Baseboard (幅木) at bottom ~5%
    let baseboard = if wall_y > 0.92 {
        0.65
    } else if wall_y > 0.88 {
        0.8
    } else {
        1.0
    };

    // --- Subtle vertical wood grain ---
    // Use wall_x to create thin vertical stripes simulating wood grain
    let grain_freq = 12.0 + (tile_hash % 5) as f64;
    let grain = 0.92 + 0.08 * (wall_x * grain_freq * std::f64::consts::PI).sin();

    // --- Horizontal plank lines (subtle) ---
    // Two horizontal lines dividing the wall into thirds
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

/// Compute window texture color.
///
/// The window is embedded in a wall: top 15% and bottom 15% are wall texture (frame),
/// and the central area has a wooden frame with glass panes divided by a cross mullion.
fn window_texture(
    wall_x: f64,
    wall_y: f64,
    side: HitSide,
    brightness: f64,
    tile_hash: u32,
) -> Color {
    // Upper and lower wall frame
    if !(0.15..=0.85).contains(&wall_y) {
        return wall_texture(wall_x, wall_y, side, brightness, tile_hash);
    }

    // Remap wall_y into the window region (0.15..0.85 -> 0.0..1.0)
    let wy = (wall_y - 0.15) / 0.70;

    let frame_thickness = 0.12;
    let mullion_half = 0.02;

    let in_frame = wall_x < frame_thickness
        || wall_x > 1.0 - frame_thickness
        || wy < frame_thickness
        || wy > 1.0 - frame_thickness;

    let on_mullion = (wall_x - 0.5).abs() < mullion_half || (wy - 0.5).abs() < mullion_half;

    if in_frame || on_mullion {
        // Wooden frame / mullion — darker brown than walls
        let base = match side {
            HitSide::Vertical => (120.0, 80.0, 50.0),
            HitSide::Horizontal => (105.0, 70.0, 42.0),
        };
        let r = (base.0 * brightness).clamp(0.0, 255.0) as u8;
        let g = (base.1 * brightness).clamp(0.0, 255.0) as u8;
        let b = (base.2 * brightness).clamp(0.0, 255.0) as u8;
        Color::rgb(r, g, b)
    } else {
        // Glass pane — blueish tint with subtle variation
        let hue_shift = ((tile_hash % 20) as f64 - 10.0) * 0.2;
        let (base_r, base_g, base_b) = (140.0, 180.0, 220.0);
        // Slight gradient: lighter toward center of each pane
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

/// Shoji (障子) texture: translucent washi paper with wooden lattice grid,
/// embedded in a wall. Upper 20% and lower 30% are wall texture.
fn shoji_texture(
    wall_x: f64,
    wall_y: f64,
    side: HitSide,
    brightness: f64,
    tile_hash: u32,
) -> Color {
    // Upper and lower wall frame
    if !(0.20..=0.70).contains(&wall_y) {
        return wall_texture(wall_x, wall_y, side, brightness, tile_hash);
    }

    let side_factor = match side {
        HitSide::Vertical => 1.0,
        HitSide::Horizontal => 0.88,
    };

    // Remap wall_y into the shoji region (0.20..0.70 -> 0.0..1.0)
    let sy = (wall_y - 0.20) / 0.50;

    // Wooden lattice (桟): 3 vertical bars + 4 horizontal bars
    let san_half = 0.015;
    let on_vertical_san = (wall_x - 0.25).abs() < san_half
        || (wall_x - 0.50).abs() < san_half
        || (wall_x - 0.75).abs() < san_half;
    let on_horizontal_san = (sy - 0.20).abs() < san_half * 1.5
        || (sy - 0.40).abs() < san_half * 1.5
        || (sy - 0.60).abs() < san_half * 1.5
        || (sy - 0.80).abs() < san_half * 1.5;
    // Outer frame of the shoji panel
    let on_frame = !(0.04..=0.96).contains(&wall_x) || !(0.04..=0.96).contains(&sy);

    if on_frame || on_vertical_san || on_horizontal_san {
        // Dark brown wooden lattice with per-tile variation
        let hue = ((tile_hash % 10) as f64 - 5.0) * 0.5;
        let (br, bg, bb) = (90.0 + hue, 60.0 + hue * 0.3, 35.0);
        let r = (br * brightness * side_factor).clamp(0.0, 255.0) as u8;
        let g = (bg * brightness * side_factor).clamp(0.0, 255.0) as u8;
        let b = (bb * brightness * side_factor).clamp(0.0, 255.0) as u8;
        Color::rgb(r, g, b)
    } else {
        // White washi paper with subtle mura (unevenness)
        let (base_r, base_g, base_b) = (240.0, 235.0, 220.0);
        // Per-tile variation in paper warmth
        let warmth = ((tile_hash % 15) as f64 - 7.0) * 0.4;
        // Subtle light variation simulating backlight glow
        let glow = 0.95 + 0.05 * ((wall_x * 3.0 + sy * 2.0) * std::f64::consts::PI).sin();

        let r = ((base_r + warmth) * brightness * side_factor * glow).clamp(0.0, 255.0) as u8;
        let g = ((base_g + warmth * 0.5) * brightness * side_factor * glow).clamp(0.0, 255.0) as u8;
        let b = ((base_b - warmth * 0.3) * brightness * side_factor * glow).clamp(0.0, 255.0) as u8;
        Color::rgb(r, g, b)
    }
}

/// Fusuma (襖) texture: white washi paper with wooden frame and pull handle.
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

    // Wooden frame at left/right edges
    if !(0.06..=0.94).contains(&wall_x) {
        let (br, bg, bb) = (100.0, 70.0, 40.0);
        let r = (br * brightness * side_factor).clamp(0.0, 255.0) as u8;
        let g = (bg * brightness * side_factor).clamp(0.0, 255.0) as u8;
        let b = (bb * brightness * side_factor).clamp(0.0, 255.0) as u8;
        return Color::rgb(r, g, b);
    }

    // Pull handle (引手) in center
    if (0.47..=0.53).contains(&wall_x) && (0.45..=0.55).contains(&wall_y) {
        let (br, bg, bb) = (180.0, 150.0, 50.0);
        let r = (br * brightness * side_factor).clamp(0.0, 255.0) as u8;
        let g = (bg * brightness * side_factor).clamp(0.0, 255.0) as u8;
        let b = (bb * brightness * side_factor).clamp(0.0, 255.0) as u8;
        return Color::rgb(r, g, b);
    }

    // White washi paper base
    let (br, bg, bb) = (230.0, 220.0, 200.0);
    let r = (br * brightness * side_factor).clamp(0.0, 255.0) as u8;
    let g = (bg * brightness * side_factor).clamp(0.0, 255.0) as u8;
    let b = (bb * brightness * side_factor).clamp(0.0, 255.0) as u8;
    Color::rgb(r, g, b)
}

/// Kitchen door texture: wood grain with door knob.
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

    // Frame at top/bottom
    if !(0.05..=0.95).contains(&wall_y) {
        let (br, bg, bb) = (110.0, 80.0, 50.0);
        let r = (br * brightness * side_factor).clamp(0.0, 255.0) as u8;
        let g = (bg * brightness * side_factor).clamp(0.0, 255.0) as u8;
        let b = (bb * brightness * side_factor).clamp(0.0, 255.0) as u8;
        return Color::rgb(r, g, b);
    }

    // Door knob
    if (0.78..=0.85).contains(&wall_x) && (0.43..=0.50).contains(&wall_y) {
        let (br, bg, bb) = (180.0, 180.0, 170.0);
        let r = (br * brightness * side_factor).clamp(0.0, 255.0) as u8;
        let g = (bg * brightness * side_factor).clamp(0.0, 255.0) as u8;
        let b = (bb * brightness * side_factor).clamp(0.0, 255.0) as u8;
        return Color::rgb(r, g, b);
    }

    // Wood grain base
    let (base_r, base_g, base_b) = (160.0, 120.0, 80.0);
    let grain_freq = 12.0 + (tile_hash % 5) as f64;
    let grain = 0.92 + 0.08 * (wall_x * grain_freq * std::f64::consts::PI).sin();

    let r = (base_r * brightness * side_factor * grain).clamp(0.0, 255.0) as u8;
    let g = (base_g * brightness * side_factor * grain).clamp(0.0, 255.0) as u8;
    let b = (base_b * brightness * side_factor * grain).clamp(0.0, 255.0) as u8;
    Color::rgb(r, g, b)
}

/// Toilet door texture: dark wood with frosted glass window.
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

    // Frame at top/bottom
    if !(0.05..=0.95).contains(&wall_y) {
        let (br, bg, bb) = (90.0, 65.0, 35.0);
        let r = (br * brightness * side_factor).clamp(0.0, 255.0) as u8;
        let g = (bg * brightness * side_factor).clamp(0.0, 255.0) as u8;
        let b = (bb * brightness * side_factor).clamp(0.0, 255.0) as u8;
        return Color::rgb(r, g, b);
    }

    // Frosted glass window
    if (0.35..=0.65).contains(&wall_x) && (0.15..=0.30).contains(&wall_y) {
        let (br, bg, bb) = (200.0, 210.0, 220.0);
        let r = (br * brightness * side_factor).clamp(0.0, 255.0) as u8;
        let g = (bg * brightness * side_factor).clamp(0.0, 255.0) as u8;
        let b = (bb * brightness * side_factor).clamp(0.0, 255.0) as u8;
        return Color::rgb(r, g, b);
    }

    // Dark wood base
    let (base_r, base_g, base_b) = (140.0, 110.0, 70.0);
    let grain_freq = 12.0 + (tile_hash % 5) as f64;
    let grain = 0.92 + 0.08 * (wall_x * grain_freq * std::f64::consts::PI).sin();

    let r = (base_r * brightness * side_factor * grain).clamp(0.0, 255.0) as u8;
    let g = (base_g * brightness * side_factor * grain).clamp(0.0, 255.0) as u8;
    let b = (base_b * brightness * side_factor * grain).clamp(0.0, 255.0) as u8;
    Color::rgb(r, g, b)
}

/// Genkan (玄関) door texture: heavy dark wood with panel grooves.
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

    // Thick frame at top/bottom
    if !(0.07..=0.93).contains(&wall_y) {
        let (br, bg, bb) = (60.0, 40.0, 20.0);
        let r = (br * brightness * side_factor).clamp(0.0, 255.0) as u8;
        let g = (bg * brightness * side_factor).clamp(0.0, 255.0) as u8;
        let b = (bb * brightness * side_factor).clamp(0.0, 255.0) as u8;
        return Color::rgb(r, g, b);
    }

    // Vertical panel grooves at 1/3 and 2/3
    let groove = if (wall_x - 0.33).abs() < 0.015 || (wall_x - 0.66).abs() < 0.015 {
        0.6
    } else {
        1.0
    };

    // Heavy dark wood base
    let (base_r, base_g, base_b) = (100.0, 70.0, 40.0);
    let r = (base_r * brightness * side_factor * groove).clamp(0.0, 255.0) as u8;
    let g = (base_g * brightness * side_factor * groove).clamp(0.0, 255.0) as u8;
    let b = (base_b * brightness * side_factor * groove).clamp(0.0, 255.0) as u8;
    Color::rgb(r, g, b)
}

/// Simple hash for tile coordinates to give per-tile variation
fn tile_hash(x: i32, y: i32) -> u32 {
    let mut h = (x as u32).wrapping_mul(374761393);
    h = h.wrapping_add((y as u32).wrapping_mul(668265263));
    h ^= h >> 13;
    h = h.wrapping_mul(1274126177);
    h ^= h >> 16;
    h
}

/// Wall height scale factor. Controls how tall walls appear relative to distance.
const WALL_HEIGHT_SCALE: f64 = 0.5;

/// Render wall columns into the framebuffer
pub fn render_walls(fb: &mut Framebuffer, rays: &[Option<RayHit>], max_depth: f64) {
    let fb_height = fb.height() as f64;

    for (col, ray) in rays.iter().enumerate() {
        let Some(hit) = ray else { continue };

        // VOID tiles produce no visible wall — column stays black
        if hit.tile == TILE_VOID {
            continue;
        }

        let distance = hit.distance.max(0.001);
        let wall_height = (fb_height / distance * WALL_HEIGHT_SCALE).min(fb_height);
        let wall_top = ((fb_height - wall_height) / 2.0).max(0.0);

        let brightness = (1.0 - distance / max_depth).max(0.0);
        let th = tile_hash(hit.map_x, hit.map_y);

        let y_start = wall_top as usize;
        let y_end = ((wall_top + wall_height) as usize).min(fb.height());

        for y in y_start..y_end {
            let wall_y = if wall_height > 0.0 {
                (y as f64 - wall_top) / wall_height
            } else {
                0.5
            };
            let color = match hit.tile {
                TILE_WINDOW => window_texture(hit.wall_x, wall_y, hit.side, brightness, th),
                TILE_SHOJI => shoji_texture(hit.wall_x, wall_y, hit.side, brightness, th),
                TILE_DOOR_FUSUMA => fusuma_texture(hit.wall_x, wall_y, hit.side, brightness, th),
                TILE_DOOR_KITCHEN => {
                    kitchen_door_texture(hit.wall_x, wall_y, hit.side, brightness, th)
                }
                TILE_DOOR_TOILET => {
                    toilet_door_texture(hit.wall_x, wall_y, hit.side, brightness, th)
                }
                TILE_DOOR_GENKAN => {
                    genkan_door_texture(hit.wall_x, wall_y, hit.side, brightness, th)
                }
                _ => wall_texture(hit.wall_x, wall_y, hit.side, brightness, th),
            };
            fb.set_pixel(col, y, color);
        }
    }
}
