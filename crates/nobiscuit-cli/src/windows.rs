//! Open interior windows are planes at the middle of a shared wall tile.
//! termray's solid-ray pass sees through them; this pass adds only the frame,
//! using pixel depths so several windows and foreground sprites overlap correctly.
use termray::{Camera, Color, Framebuffer, RayHit, SpriteArt, SpriteRenderResult, TileMap};

use crate::textures::NobiscuitTextures;
use crate::tiles::TILE_WINDOW_PASS;

pub fn render_windows(
    fb: &mut Framebuffer,
    map: &dyn TileMap,
    camera: &Camera,
    rays: &[Option<RayHit>],
    sprites: &[SpriteRenderResult],
    max_depth: f64,
) {
    let width = fb.width();
    let height = fb.height();
    let mut depths = vec![f64::INFINITY; width * height];
    // Mirror termray 0.3's sprite raster bounds and transparent pattern pixels.
    // A bounding-box depth would incorrectly hide frames behind sprite holes.
    for sprite in sprites {
        let Some(art) = NobiscuitTextures.art(sprite.sprite_type) else {
            continue;
        };
        let ph = art.pattern.len();
        let pw = art.pattern.first().map_or(0, |row| row.len());
        if ph == 0 || pw == 0 {
            continue;
        }
        let sh = (sprite.screen_height * art.height_scale) as i32;
        let sw = sh * pw as i32 / ph as i32;
        if sh <= 0 || sw <= 0 {
            continue;
        }
        let left = sprite.screen_x - sw / 2;
        let top = sprite.screen_y_feet as i32 - sh - (sh as f64 * art.float_offset_scale) as i32;
        for x in left.max(0)..(left + sw).min(width as i32) {
            if rays
                .get(x as usize)
                .and_then(Option::as_ref)
                .is_some_and(|hit| sprite.distance > hit.distance)
            {
                continue;
            }
            let px = (((x - left) as f64 / sw as f64) * pw as f64) as usize;
            for y in top.max(0)..(top + sh).min(height as i32) {
                let py = (((y - top) as f64 / sh as f64) * ph as f64) as usize;
                if matches!(art.pattern[py].as_bytes().get(px), Some(b'#' | b'+')) {
                    let depth = &mut depths[y as usize * width + x as usize];
                    *depth = depth.min(sprite.distance);
                }
            }
        }
    }
    let center =
        height as f64 / 2.0 + camera.pitch.tan() * (width as f64 / 2.0) / (camera.fov / 2.0).tan();
    // Only tiles within the ray limit can contribute. This also bounds work on
    // the largest maps while keeping all visible window layers.
    let min_x = (camera.x - max_depth).floor().max(0.0) as usize;
    let min_y = (camera.y - max_depth).floor().max(0.0) as usize;
    let max_x = ((camera.x + max_depth).ceil() as usize).min(map.width());
    let max_y = ((camera.y + max_depth).ceil() as usize).min(map.height());
    for my in min_y..max_y {
        for mx in min_x..max_x {
            if map.get(mx as i32, my as i32) != Some(TILE_WINDOW_PASS) {
                continue;
            }
            // A horizontal shared wall has solid jamb tiles to left and right.
            let horizontal =
                map.is_solid(mx as i32 - 1, my as i32) && map.is_solid(mx as i32 + 1, my as i32);
            for (col, ray) in rays.iter().enumerate().take(width) {
                let angle =
                    camera.angle - camera.fov / 2.0 + camera.fov * col as f64 / width as f64;
                let (dx, dy) = (angle.cos(), angle.sin());
                let (distance, across) = if horizontal {
                    let d = (my as f64 + 0.5 - camera.y) / dy;
                    (d, camera.x + d * dx - mx as f64)
                } else {
                    let d = (mx as f64 + 0.5 - camera.x) / dx;
                    (d, camera.y + d * dy - my as f64)
                };
                if !distance.is_finite()
                    || distance <= 0.001
                    || distance > max_depth
                    || !(0.0..1.0).contains(&across)
                    || ray.as_ref().is_some_and(|hit| distance >= hit.distance)
                {
                    continue;
                }
                let scale = height as f64 * termray::WALL_HEIGHT_SCALE / distance;
                let top = center - (1.0 - camera.z) * scale;
                let bottom = center + camera.z * scale;
                for row in (top.max(0.0) as usize)..(bottom.min(height as f64).max(0.0) as usize) {
                    let v = (row as f64 - top) / scale;
                    // Broad opening, raised sill and lintel, no glass/mullion.
                    // Keep true texture coordinates near the window: stretching
                    // the clipped frame would make it close over the passage.
                    if (0.10..=0.90).contains(&across) && (0.18..=0.92).contains(&v) {
                        continue;
                    }
                    let depth = &mut depths[row * width + col];
                    if distance >= *depth {
                        continue;
                    }
                    *depth = distance;
                    let trim = (0.07..=0.93).contains(&across) && (0.14..=0.96).contains(&v);
                    let color = if trim {
                        Color::rgb(65, 165, 150)
                    } else {
                        Color::rgb(120, 80, 45)
                    };
                    let light =
                        (1.0 - distance / max_depth).max(0.0) * if horizontal { 0.88 } else { 1.0 };
                    fb.set_pixel(col, row, color.darken(light));
                }
            }
        }
    }
}
