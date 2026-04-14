use crate::framebuffer::{Color, Framebuffer};
use crate::math::normalize_angle;
use crate::ray::RayHit;

#[derive(Debug, Clone)]
pub struct Sprite {
    pub x: f64,
    pub y: f64,
    pub sprite_type: u8,
}

#[derive(Debug, Clone)]
pub struct SpriteRenderResult {
    pub screen_x: i32,
    pub screen_height: i32,
    pub distance: f64,
    pub sprite_type: u8,
}

/// Project sprites into screen space, sorted far-to-near
pub fn project_sprites(
    sprites: &[Sprite],
    camera_x: f64,
    camera_y: f64,
    camera_angle: f64,
    fov: f64,
    screen_width: usize,
) -> Vec<SpriteRenderResult> {
    let mut results: Vec<SpriteRenderResult> = sprites
        .iter()
        .filter_map(|s| {
            let dx = s.x - camera_x;
            let dy = s.y - camera_y;
            let distance = (dx * dx + dy * dy).sqrt();

            if distance < 0.3 {
                return None; // too close
            }

            let sprite_angle = dy.atan2(dx);
            let angle_diff = normalize_angle(sprite_angle - camera_angle + std::f64::consts::PI)
                - std::f64::consts::PI;

            // Only render if within FOV (with some margin)
            if angle_diff.abs() > fov * 0.6 {
                return None;
            }

            let screen_x =
                ((angle_diff + fov / 2.0) / fov * screen_width as f64) as i32;
            let screen_height = (screen_width as f64 / distance * 0.4) as i32;

            Some(SpriteRenderResult {
                screen_x,
                screen_height,
                distance,
                sprite_type: s.sprite_type,
            })
        })
        .collect();

    // Sort far-to-near (painter's algorithm)
    results.sort_by(|a, b| b.distance.partial_cmp(&a.distance).unwrap());
    results
}

/// Render projected sprites into the framebuffer, respecting wall depth buffer
pub fn render_sprites(
    fb: &mut Framebuffer,
    projected: &[SpriteRenderResult],
    rays: &[Option<RayHit>],
    color_fn: &dyn Fn(u8) -> Color,
    max_depth: f64,
) {
    let fb_height = fb.height() as f64;

    for spr in projected {
        let half_w = spr.screen_height / 2;
        let half_h = spr.screen_height / 2;
        let center_y = (fb_height / 2.0) as i32;

        let brightness = (1.0 - spr.distance / max_depth).max(0.1);
        let base_color = color_fn(spr.sprite_type);
        let color = base_color.darken(brightness);

        let x_start = (spr.screen_x - half_w).max(0) as usize;
        let x_end = ((spr.screen_x + half_w) as usize).min(fb.width());
        let y_start = ((center_y - half_h).max(0)) as usize;
        let y_end = ((center_y + half_h) as usize).min(fb.height());

        for x in x_start..x_end {
            // Depth test: don't draw sprite behind walls
            if let Some(Some(hit)) = rays.get(x) {
                if spr.distance > hit.distance {
                    continue;
                }
            }

            // Draw a filled column for the sprite
            // Shape: leave top/bottom 20% empty for a rounded look
            let sprite_col_h = y_end - y_start;
            let margin = sprite_col_h / 5;
            for y in (y_start + margin)..(y_end.saturating_sub(margin)) {
                fb.set_pixel(x, y, color);
            }
        }
    }
}
