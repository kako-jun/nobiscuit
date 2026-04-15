use nobiscuit_engine::framebuffer::{Color, Framebuffer};
use nobiscuit_engine::map::{
    TileMap, TILE_GOAL, TILE_STAIRS_DOWN, TILE_STAIRS_UP, TILE_VOID, TILE_WALL, TILE_WINDOW,
};

const MINIMAP_SCALE: usize = 2;
const MINIMAP_ALPHA: f64 = 0.4;

pub fn render_minimap(
    fb: &mut Framebuffer,
    map: &dyn TileMap,
    player_x: f64,
    player_y: f64,
    player_angle: f64,
) {
    let map_pixel_w = map.width() * MINIMAP_SCALE;
    let map_pixel_h = map.height() * MINIMAP_SCALE;

    // Position: flush right, vertically centered
    let offset_x = fb.width().saturating_sub(map_pixel_w);
    let offset_y = fb.height().saturating_sub(map_pixel_h) / 2;

    // Draw map tiles (semi-transparent)
    for my in 0..map.height() {
        for mx in 0..map.width() {
            let tile = map.get(mx as i32, my as i32).unwrap_or(TILE_WALL);

            // VOID tiles are not drawn on the minimap (stay black)
            if tile == TILE_VOID {
                continue;
            }

            let color = match tile {
                TILE_WALL => Color::rgb(40, 60, 40),
                TILE_WINDOW => Color::rgb(80, 120, 180),
                TILE_GOAL => Color::rgb(255, 215, 0),
                TILE_STAIRS_UP => Color::rgb(200, 150, 50),
                TILE_STAIRS_DOWN => Color::rgb(150, 100, 30),
                _ => Color::rgb(100, 140, 100),
            };

            for py in 0..MINIMAP_SCALE {
                for px in 0..MINIMAP_SCALE {
                    let fx = offset_x + mx * MINIMAP_SCALE + px;
                    let fy = offset_y + my * MINIMAP_SCALE + py;
                    fb.blend_pixel(fx, fy, color, MINIMAP_ALPHA);
                }
            }
        }
    }

    // Draw player dot (red, 3x3, slightly more opaque)
    let px = offset_x + (player_x * MINIMAP_SCALE as f64) as usize;
    let py = offset_y + (player_y * MINIMAP_SCALE as f64) as usize;
    let player_color = Color::rgb(255, 50, 50);
    for dy in 0..3_usize {
        for ddx in 0..3_usize {
            let draw_x = (px + ddx).saturating_sub(1);
            let draw_y = (py + dy).saturating_sub(1);
            fb.blend_pixel(draw_x, draw_y, player_color, 0.8);
        }
    }

    // Draw direction indicator (line from player, 5 pixels long)
    let dir_color = Color::rgb(255, 255, 0);
    for i in 1..=5 {
        let lx = px as f64 + player_angle.cos() * i as f64;
        let ly = py as f64 + player_angle.sin() * i as f64;
        if lx >= 0.0 && ly >= 0.0 {
            fb.blend_pixel(lx as usize, ly as usize, dir_color, 0.8);
        }
    }
}
