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

/// AA art definition for a sprite type.
/// Each entry is a row of the pattern; '#' = filled, '.' = transparent.
/// Patterns are defined at a canonical size and scaled at render time.
struct SpriteArt {
    pattern: &'static [&'static str],
    /// Vertical scale relative to wall height (0.5 = half wall height)
    height_scale: f64,
}

fn get_sprite_art(sprite_type: u8) -> SpriteArt {
    match sprite_type {
        // Biscuit — small round cookie shape, half wall height
        1 => SpriteArt {
            pattern: &[
                "..####..", ".######.", "########", "##.##.##", "########", ".######.", "..####..",
            ],
            height_scale: 0.25,
        },
        // Goal — floating sphere
        2 => SpriteArt {
            pattern: &[
                "..####..", ".######.", "###++###", "###+.###", "########", ".######.", "..####..",
            ],
            height_scale: 0.25,
        },
        // Stairs up — upward arrow
        3 => SpriteArt {
            pattern: &["..##..", ".####.", "######", "..##..", "..##..", "..##.."],
            height_scale: 0.3,
        },
        // Stairs down — downward arrow
        4 => SpriteArt {
            pattern: &["..##..", "..##..", "..##..", "######", ".####.", "..##.."],
            height_scale: 0.3,
        },
        // Fallback — diamond
        _ => SpriteArt {
            pattern: &["..##..", ".####.", "######", ".####.", "..##.."],
            height_scale: 0.25,
        },
    }
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

            let art = get_sprite_art(s.sprite_type);

            let screen_x = ((angle_diff + fov / 2.0) / fov * screen_width as f64) as i32;
            // Height based on sprite's own scale, not wall scale
            let screen_height = (screen_width as f64 / distance * art.height_scale) as i32;

            Some(SpriteRenderResult {
                screen_x,
                screen_height,
                distance,
                sprite_type: s.sprite_type,
            })
        })
        .collect();

    // Sort far-to-near (painter's algorithm)
    results.sort_by(|a, b| b.distance.total_cmp(&a.distance));
    results
}

/// Render projected sprites into the framebuffer with AA art patterns
pub fn render_sprites(
    fb: &mut Framebuffer,
    projected: &[SpriteRenderResult],
    rays: &[Option<RayHit>],
    color_fn: &dyn Fn(u8) -> Color,
    max_depth: f64,
) {
    let fb_height = fb.height() as f64;

    for spr in projected {
        let art = get_sprite_art(spr.sprite_type);
        let pat = art.pattern;
        let pat_h = pat.len();
        let pat_w = pat.first().map_or(0, |r| r.len());

        if pat_h == 0 || pat_w == 0 {
            continue;
        }

        let brightness = (1.0 - spr.distance / max_depth).max(0.1);
        let base_color = color_fn(spr.sprite_type);
        let color = base_color.darken(brightness);
        let shadow_color = base_color.darken(brightness * 0.5);

        // Sprite dimensions on screen
        let sprite_w = spr.screen_height * pat_w as i32 / pat_h as i32;
        let sprite_h = spr.screen_height;

        // Vertical placement:
        // - Biscuits sit on the floor (bottom at horizon)
        // - Goal floats slightly above the floor
        let center_y = (fb_height / 2.0) as i32;
        let float_offset = if spr.sprite_type == 2 {
            sprite_h / 3 // goal floats up
        } else {
            0
        };
        let y_top = center_y - sprite_h / 4 - float_offset;

        let x_left = spr.screen_x - sprite_w / 2;

        for sx in 0..sprite_w {
            let screen_x = x_left + sx;
            if screen_x < 0 || screen_x >= fb.width() as i32 {
                continue;
            }
            let col = screen_x as usize;

            // Depth test: don't draw sprite behind walls
            if let Some(Some(hit)) = rays.get(col) {
                if spr.distance > hit.distance {
                    continue;
                }
            }

            // Map screen column to pattern column
            let pat_col = (sx as f64 / sprite_w as f64 * pat_w as f64) as usize;
            let pat_col = pat_col.min(pat_w - 1);

            for sy in 0..sprite_h {
                let screen_y = y_top + sy;
                if screen_y < 0 || screen_y >= fb.height() as i32 {
                    continue;
                }

                // Map screen row to pattern row
                let pat_row = (sy as f64 / sprite_h as f64 * pat_h as f64) as usize;
                let pat_row = pat_row.min(pat_h - 1);

                let ch = pat[pat_row]
                    .as_bytes()
                    .get(pat_col)
                    .copied()
                    .unwrap_or(b'.');

                match ch {
                    b'#' => {
                        fb.set_pixel(col, screen_y as usize, color);
                    }
                    b'+' => {
                        // Secondary color (detail/shadow)
                        fb.set_pixel(col, screen_y as usize, shadow_color);
                    }
                    _ => {
                        // '.' = transparent, skip
                    }
                }
            }
        }
    }
}
