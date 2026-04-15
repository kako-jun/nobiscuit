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
use nobiscuit_engine::map::TileMap;
use nobiscuit_engine::renderer;
use nobiscuit_engine::sprite;

use crate::game::{
    EndingPhase, GameState, World, SPRITE_BISCUIT, SPRITE_GOAL, SPRITE_STAIRS_DOWN,
    SPRITE_STAIRS_UP,
};
use crate::input::{poll_input, GameInput};
use crate::player::Player;
use crate::terminal::TerminalRenderer;

const MAX_DEPTH: f64 = 20.0;
const TARGET_FPS: u64 = 30;
const FRAME_DURATION: Duration = Duration::from_millis(1000 / TARGET_FPS);

const FLOOR_COLOR: Color = Color {
    r: 74,
    g: 60,
    b: 40,
};
const CEILING_COLOR: Color = Color {
    r: 135,
    g: 206,
    b: 235,
};

const NUM_FLOORS: usize = 3;

fn sprite_color(sprite_type: u8) -> Color {
    match sprite_type {
        SPRITE_BISCUIT => Color::rgb(220, 180, 80), // golden biscuit
        SPRITE_GOAL => Color::rgb(50, 220, 50),     // green exit
        SPRITE_STAIRS_UP => Color::rgb(200, 150, 50), // warm stairs up
        SPRITE_STAIRS_DOWN => Color::rgb(150, 100, 30), // dark stairs down
        _ => Color::rgb(255, 255, 255),
    }
}

fn main() {
    let mut term = TerminalRenderer::new();
    let (cols, rows) = term.size();
    let fb_width = cols;
    let fb_height = rows * 2;

    let mut fb = Framebuffer::new(fb_width, fb_height);

    // Generate world with multiple floors
    let mut rng = rand::thread_rng();
    let maze_w = 31;
    let maze_h = 25;
    let mut world = World::new(NUM_FLOORS, maze_w, maze_h, &mut rng);

    // Player starts at (1.5, 1.5) facing right on ground floor
    let mut player = Player::new(1.5, 1.5);
    let mut state = GameState::new();
    state.init_visited(&world);

    let mut last_frame = Instant::now();

    loop {
        let now = Instant::now();
        let dt = now.duration_since(last_frame).as_secs_f64().min(0.1); // cap at 100ms
        last_frame = now;

        // Input
        let input = poll_input(Duration::from_millis(5));

        if state.is_alive && !state.escaped {
            // Handle quit and toggles during normal play
            match &input {
                Some(GameInput::Quit) => break,
                Some(GameInput::ToggleMinimap) => {
                    state.activate_minimap();
                }
                _ => {}
            }

            // Update player
            player.update(input.as_ref(), world.current_map(), dt);

            // Update fog of war
            let map_w = world.current_map().width();
            let map_h = world.current_map().height();
            state.update_visited(
                world.current_floor,
                player.camera.x.max(0.0) as usize,
                player.camera.y.max(0.0) as usize,
                map_w,
                map_h,
            );

            // Update game state (hunger, pickups, stairs)
            state.update(&mut world, player.camera.x, player.camera.y, dt);

            // Handle floor transition
            if let Some(transition) = state.floor_transition.take() {
                // Restore all open doors on the current floor before switching
                state.restore_all_doors(&mut world);
                let (nx, ny) = world.change_floor(transition.target_floor, transition.direction);
                player.teleport(nx, ny);
                state.mark_on_stair();
            }
        } else {
            // Dead or escaped: advance ending phases
            match state.ending_phase {
                EndingPhase::FadeOut(timer) => {
                    let new_timer = timer - dt;
                    if new_timer <= 0.0 {
                        state.ending_phase = EndingPhase::Result(0.0);
                    } else {
                        state.ending_phase = EndingPhase::FadeOut(new_timer);
                    }
                    // Ignore all input during fade
                }
                EndingPhase::Result(timer) => {
                    state.ending_phase = EndingPhase::Result(timer + dt);
                    match &input {
                        Some(GameInput::Retry) => {
                            // Restart the game
                            world = World::new(NUM_FLOORS, maze_w, maze_h, &mut rng);
                            player = Player::new(1.5, 1.5);
                            state = GameState::new();
                            state.init_visited(&world);
                            continue;
                        }
                        Some(GameInput::Quit) => break,
                        _ => {}
                    }
                }
                EndingPhase::None => {
                    // Shouldn't happen, but handle gracefully
                    if input.is_some() {
                        break;
                    }
                }
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

        match state.ending_phase {
            EndingPhase::Result(timer) => {
                // Black screen with result text
                if !state.is_alive {
                    ui::render_game_over_screen(&mut fb, timer);
                } else {
                    ui::render_clear_screen(
                        &mut fb,
                        timer,
                        state.biscuits_eaten,
                        state.elapsed_time,
                        state.floors_visited.len(),
                    );
                }
            }
            _ => {
                // Normal 3D rendering (also used during FadeOut)
                let current_map = world.current_map();
                let num_rays = fb.width();
                let rays = player
                    .camera
                    .cast_all_rays(current_map, num_rays, MAX_DEPTH);

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

                // Sprites (biscuits + goal + stairs)
                let projected = sprite::project_sprites(
                    world.current_sprites(),
                    player.camera.x,
                    player.camera.y,
                    player.camera.angle,
                    player.camera.fov,
                    fb.width(),
                );
                sprite::render_sprites(&mut fb, &projected, &rays, &sprite_color, MAX_DEPTH);

                // Minimap overlay
                if state.show_minimap {
                    let visited_floor = if state.debug_mode {
                        &[] as &[Vec<bool>]
                    } else {
                        state
                            .visited
                            .get(world.current_floor)
                            .map(|v| v.as_slice())
                            .unwrap_or(&[])
                    };
                    let reveal_all = state.debug_mode || state.minimap_reveal_all;
                    minimap::render_minimap(
                        &mut fb,
                        current_map,
                        player.camera.x,
                        player.camera.y,
                        player.camera.angle,
                        visited_floor,
                        reveal_all,
                    );
                }

                // HUD: hunger bar (always visible) + floor indicator (only with minimap)
                ui::render_hunger_bar(&mut fb, state.hunger);
                if state.show_minimap {
                    ui::render_floor_indicator(&mut fb, world.current_floor + 1, NUM_FLOORS);
                }

                // Message display
                if let Some((ref text, _)) = state.message {
                    let msg_color = if state.is_alive {
                        Color::rgb(255, 255, 200)
                    } else {
                        Color::rgb(255, 80, 80)
                    };
                    ui::render_message(&mut fb, text, msg_color);
                }

                // FadeOut effects
                if let EndingPhase::FadeOut(timer) = state.ending_phase {
                    let fade_total = 3.0;
                    let factor = (timer / fade_total).clamp(0.0, 1.0);
                    fb.darken_all(factor);

                    // Starvation: shift frame down for collapse effect
                    if !state.is_alive {
                        let progress = ((fade_total - timer) / fade_total).clamp(0.0, 1.0);
                        let shift = (progress * fb.height() as f64 * 0.3) as usize;
                        fb.shift_down(shift);
                    }
                }
            }
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
