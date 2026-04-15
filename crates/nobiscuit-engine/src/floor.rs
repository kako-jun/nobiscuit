use crate::framebuffer::{Color, Framebuffer};
use crate::ray::RayHit;

/// Render floor and ceiling with perspective-correct tile pattern.
///
/// Floor uses a tatami/wood-plank grid pattern whose density increases
/// with distance (Mode7-style). Ceiling uses a subtle grid that fades.
pub fn render_floor_ceiling(
    fb: &mut Framebuffer,
    rays: &[Option<RayHit>],
    _max_depth: f64,
    floor_color: Color,
    ceiling_color: Color,
    camera_x: f64,
    camera_y: f64,
    camera_angle: f64,
    fov: f64,
) {
    let fb_width = fb.width();
    let fb_height = fb.height();
    let fb_h_f = fb_height as f64;
    let horizon = fb_h_f / 2.0;

    let dir_x = camera_angle.cos();
    let dir_y = camera_angle.sin();
    let plane_x = -(fov / 2.0).tan() * dir_y;
    let plane_y = (fov / 2.0).tan() * dir_x;

    // Determine wall bounds per column for gap fill
    let wall_bounds: Vec<(usize, usize)> = rays
        .iter()
        .map(|ray| {
            if let Some(hit) = ray {
                let distance = hit.distance.max(0.001);
                let wall_height = (fb_h_f / distance * 0.5).min(fb_h_f);
                let top = ((fb_h_f - wall_height) / 2.0).max(0.0);
                (top as usize, ((top + wall_height) as usize).min(fb_height))
            } else {
                (fb_height / 2, fb_height / 2)
            }
        })
        .collect();

    for y in 0..fb_height {
        let row_dist_from_horizon = (y as f64 - horizon).abs();
        if row_dist_from_horizon < 0.5 {
            continue; // skip the horizon line itself
        }

        let is_floor = y as f64 > horizon;

        // Perspective distance for this row
        let row_distance = horizon / row_dist_from_horizon;
        let brightness = (1.0 / (1.0 + row_distance * 0.15)).clamp(0.08, 1.0);

        for col in 0..fb_width {
            let (wall_top, wall_bottom) = wall_bounds[col];
            if is_floor && y < wall_bottom {
                continue;
            }
            if !is_floor && y >= wall_top {
                continue;
            }

            // World-space coordinate of this floor/ceiling pixel
            let camera_frac = (col as f64 / fb_width as f64) * 2.0 - 1.0;
            let floor_x = camera_x + (dir_x + plane_x * camera_frac) * row_distance;
            let floor_y = camera_y + (dir_y + plane_y * camera_frac) * row_distance;

            if is_floor {
                let color = floor_tile_color(floor_x, floor_y, floor_color, brightness);
                fb.set_pixel(col, y, color);
            } else {
                let color = ceiling_tile_color(floor_x, floor_y, ceiling_color, brightness);
                fb.set_pixel(col, y, color);
            }
        }
    }
}

/// Floor pattern: tatami/wood plank grid.
/// Grid lines get thicker relative to tile size as distance increases,
/// creating the Mode7-like density gradient naturally.
fn floor_tile_color(wx: f64, wy: f64, base: Color, brightness: f64) -> Color {
    let tile_size = 1.0; // 1 world unit = 1 map tile
    let tx = (wx / tile_size).fract().abs();
    let ty = (wy / tile_size).fract().abs();

    // Grid lines at tile edges
    let grid_width = 0.04;
    let on_grid = tx < grid_width || tx > (1.0 - grid_width) || ty < grid_width || ty > (1.0 - grid_width);

    // Subtle inner plank lines (horizontal grain in each tile)
    let plank_line = (ty * 6.0).fract() < 0.08;

    // Per-tile color variation using cheap hash
    let tile_ix = wx.floor() as i32;
    let tile_iy = wy.floor() as i32;
    let hash = ((tile_ix.wrapping_mul(2654435761_u32 as i32)) ^ (tile_iy.wrapping_mul(2246822519_u32 as i32))) as u32;
    let variation = 0.9 + 0.1 * ((hash % 100) as f64 / 100.0);

    if on_grid {
        // Dark grooves between tiles
        base.darken(brightness * 0.4)
    } else if plank_line {
        base.darken(brightness * variation * 0.75)
    } else {
        base.darken(brightness * variation)
    }
}

/// Ceiling pattern: subtle stucco-like grid, less pronounced than floor.
fn ceiling_tile_color(wx: f64, wy: f64, base: Color, brightness: f64) -> Color {
    let tx = (wx * 0.5).fract().abs();
    let ty = (wy * 0.5).fract().abs();

    // Very faint grid for ceiling panels
    let on_grid = tx < 0.02 || tx > 0.98 || ty < 0.02 || ty > 0.98;

    if on_grid {
        base.darken(brightness * 0.7)
    } else {
        base.darken(brightness)
    }
}
