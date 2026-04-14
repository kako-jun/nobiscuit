use crate::framebuffer::{Color, Framebuffer};
use crate::ray::{HitSide, RayHit};

/// Render wall columns into the framebuffer
pub fn render_walls(fb: &mut Framebuffer, rays: &[Option<RayHit>], max_depth: f64) {
    let fb_height = fb.height() as f64;

    for (col, ray) in rays.iter().enumerate() {
        let Some(hit) = ray else { continue };

        let distance = hit.distance.max(0.001);
        let wall_height = (fb_height / distance * 0.5).min(fb_height);
        let wall_top = ((fb_height - wall_height) / 2.0).max(0.0);

        let brightness = (1.0 - distance / max_depth).max(0.0);

        let base_color = match hit.side {
            HitSide::Vertical => Color::rgb(180, 140, 100),
            HitSide::Horizontal => Color::rgb(160, 120, 80),
        };
        let wall_color = base_color.darken(brightness);

        let y_start = wall_top as usize;
        let y_end = ((wall_top + wall_height) as usize).min(fb.height());

        for y in y_start..y_end {
            fb.set_pixel(col, y, wall_color);
        }
    }
}
