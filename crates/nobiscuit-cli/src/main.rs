mod game;
mod input;
mod maze;
mod minimap;
mod player;
mod terminal;
mod ui;

use std::time::{Duration, Instant};

use nobiscuit_engine::floor;
use nobiscuit_engine::framebuffer::{Color, Framebuffer};
use nobiscuit_engine::renderer;
use nobiscuit_engine::sprite;

use crate::game::{GameState, SPRITE_BISCUIT, SPRITE_GOAL};
use crate::input::{poll_input, GameInput};
use crate::player::Player;
use crate::terminal::TerminalRenderer;

const MAX_DEPTH: f64 = 20.0;
const TARGET_FPS: u64 = 30;
const FRAME_DURATION: Duration = Duration::from_millis(1000 / TARGET_FPS);

const FLOOR_COLOR: Color = Color { r: 74, g: 60, b: 40 };
const CEILING_COLOR: Color = Color {
    r: 135,
    g: 206,
    b: 235,
};

fn sprite_color(sprite_type: u8) -> Color {
    match sprite_type {
        SPRITE_BISCUIT => Color::rgb(220, 180, 80), // golden biscuit
        SPRITE_GOAL => Color::rgb(50, 220, 50),     // green exit
        _ => Color::rgb(255, 255, 255),
    }
}

fn main() {
    let mut term = TerminalRenderer::new();
    let (cols, rows) = term.size();
    let fb_width = cols;
    let fb_height = rows * 2;

    let mut fb = Framebuffer::new(fb_width, fb_height);

    // Generate maze
    let mut rng = rand::thread_rng();
    let maze_w = 31;
    let maze_h = 25;
    let map = maze::generate_maze(maze_w, maze_h, &mut rng);

    // Player starts at (1.5, 1.5) facing right
    let mut player = Player::new(1.5, 1.5);
    let mut state = GameState::new();
    state.place_items(&map, &mut rng);

    let mut last_frame = Instant::now();

    loop {
        let now = Instant::now();
        let dt = now.duration_since(last_frame).as_secs_f64().min(0.1); // cap at 100ms
        last_frame = now;

        // Input
        let input = poll_input(Duration::from_millis(5));

        // Handle quit and toggles
        match &input {
            Some(GameInput::Quit) => break,
            Some(GameInput::ToggleMinimap) => {
                state.show_minimap = !state.show_minimap;
            }
            _ => {}
        }

        if state.is_alive && !state.escaped {
            // Update player
            player.update(input.as_ref(), &map, dt);

            // Update game state (hunger, pickups)
            state.update(player.camera.x, player.camera.y, dt);
        } else {
            // Dead or escaped: any key to quit
            if input.is_some() {
                break;
            }
        }

        // Check terminal resize
        term.resize();
        let (cols, rows) = term.size();
        let new_w = cols;
        let new_h = rows * 2;
        if fb.width() != new_w || fb.height() != new_h {
            fb = Framebuffer::new(new_w, new_h);
        }

        // Render
        fb.clear(Color::default());

        let num_rays = fb.width();
        let rays = player.camera.cast_all_rays(&map, num_rays, MAX_DEPTH);

        // Floor and ceiling
        floor::render_floor_ceiling(
            &mut fb,
            &rays,
            FLOOR_COLOR,
            CEILING_COLOR,
            &player.camera,
        );

        // Walls
        renderer::render_walls(&mut fb, &rays, MAX_DEPTH);

        // Sprites (biscuits + goal)
        let projected = sprite::project_sprites(
            &state.sprites,
            player.camera.x,
            player.camera.y,
            player.camera.angle,
            player.camera.fov,
            fb.width(),
        );
        sprite::render_sprites(&mut fb, &projected, &rays, &sprite_color, MAX_DEPTH);

        // Minimap overlay
        if state.show_minimap {
            minimap::render_minimap(
                &mut fb,
                &map,
                player.camera.x,
                player.camera.y,
                player.camera.angle,
            );
        }

        // HUD: hunger bar
        ui::render_hunger_bar(&mut fb, state.hunger);

        // Message display
        if let Some((ref text, _)) = state.message {
            let msg_color = if state.is_alive {
                Color::rgb(255, 255, 200)
            } else {
                Color::rgb(255, 80, 80)
            };
            ui::render_message(&mut fb, text, msg_color);
        }

        // Present to terminal
        term.present(&fb);

        // Frame timing
        let elapsed = last_frame.elapsed();
        if elapsed < FRAME_DURATION {
            std::thread::sleep(FRAME_DURATION - elapsed);
        }
    }

    term.cleanup();
}
