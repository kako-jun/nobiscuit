use crate::framebuffer::{Color, Framebuffer};
use crate::ray::RayHit;

/// Render floor and ceiling gradients
///
/// Near the horizon (center of screen) = darker (far away).
/// Near top/bottom edges = brighter (closer).
pub fn render_floor_ceiling(
    fb: &mut Framebuffer,
    rays: &[Option<RayHit>],
    _max_depth: f64,
    floor_color: Color,
    ceiling_color: Color,
) {
    let fb_height = fb.height() as f64;
    let horizon = fb_height / 2.0;

    for (col, ray) in rays.iter().enumerate() {
        // Determine where the wall starts and ends
        let (wall_top, wall_bottom) = if let Some(hit) = ray {
            let distance = hit.distance.max(0.001);
            let wall_height = (fb_height / distance * 0.5).min(fb_height);
            let top = ((fb_height - wall_height) / 2.0).max(0.0);
            (top as usize, ((top + wall_height) as usize).min(fb.height()))
        } else {
            (fb.height() / 2, fb.height() / 2)
        };

        // Ceiling: rows 0..wall_top
        for y in 0..wall_top {
            let dist_from_horizon = (horizon - y as f64).abs();
            let brightness = (dist_from_horizon / horizon).clamp(0.1, 1.0);
            fb.set_pixel(col, y, ceiling_color.darken(brightness));
        }

        // Floor: rows wall_bottom..fb_height
        for y in wall_bottom..fb.height() {
            let dist_from_horizon = (y as f64 - horizon).abs();
            let brightness = (dist_from_horizon / horizon).clamp(0.1, 1.0);
            fb.set_pixel(col, y, floor_color.darken(brightness));
        }
    }
}
