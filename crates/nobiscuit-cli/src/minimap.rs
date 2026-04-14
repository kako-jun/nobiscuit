use nobiscuit_engine::framebuffer::{Color, Framebuffer};
use nobiscuit_engine::map::{TileMap, TILE_GOAL, TILE_WALL};

const MINIMAP_SCALE: usize = 4;
const MINIMAP_MARGIN: usize = 4;

pub fn render_minimap(
    fb: &mut Framebuffer,
    map: &dyn TileMap,
    player_x: f64,
    player_y: f64,
    player_angle: f64,
) {
    let map_pixel_w = map.width() * MINIMAP_SCALE;
    let map_pixel_h = map.height() * MINIMAP_SCALE;

    // Position: bottom-right corner
    let offset_x = fb.width().saturating_sub(map_pixel_w + MINIMAP_MARGIN);
    let offset_y = fb.height().saturating_sub(map_pixel_h + MINIMAP_MARGIN);

    // Draw map tiles
    for my in 0..map.height() {
        for mx in 0..map.width() {
            let tile = map.get(mx as i32, my as i32).unwrap_or(TILE_WALL);
            let color = match tile {
                TILE_WALL => Color::rgb(40, 60, 40),
                TILE_GOAL => Color::rgb(255, 215, 0),
                _ => Color::rgb(100, 140, 100),
            };

            for py in 0..MINIMAP_SCALE {
                for px in 0..MINIMAP_SCALE {
                    let fx = offset_x + mx * MINIMAP_SCALE + px;
                    let fy = offset_y + my * MINIMAP_SCALE + py;
                    fb.set_pixel(fx, fy, color);
                }
            }
        }
    }

    // Draw player dot (red, 3x3)
    let px = offset_x + (player_x * MINIMAP_SCALE as f64) as usize;
    let py = offset_y + (player_y * MINIMAP_SCALE as f64) as usize;
    let player_color = Color::rgb(255, 50, 50);
    for dy in 0..3_usize {
        for ddx in 0..3_usize {
            let draw_x = (px + ddx).saturating_sub(1);
            let draw_y = (py + dy).saturating_sub(1);
            fb.set_pixel(draw_x, draw_y, player_color);
        }
    }

    // Draw direction indicator (line from player, 5 pixels long)
    let dir_color = Color::rgb(255, 255, 0);
    for i in 1..=5 {
        let lx = px as f64 + player_angle.cos() * i as f64;
        let ly = py as f64 + player_angle.sin() * i as f64;
        if lx >= 0.0 && ly >= 0.0 {
            fb.set_pixel(lx as usize, ly as usize, dir_color);
        }
    }
}
