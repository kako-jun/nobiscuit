use crate::framebuffer::{Color, Framebuffer};
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
            let color = wall_texture(hit.wall_x, wall_y, hit.side, brightness, th);
            fb.set_pixel(col, y, color);
        }
    }
}
