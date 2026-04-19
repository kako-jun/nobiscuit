mod game;
mod input;
mod maze;
mod minimap;
mod nobiscuit_map;
mod player;
mod terminal;
mod textures;
mod tiles;
mod ui;

use std::time::{Duration, Instant};

use termray::{Color, FlatHeightMap, Framebuffer, TileMap, render_floor_ceiling, render_walls};

use crate::game::{EndingPhase, FADE_DURATION, GamePhase, GameState, World};
use crate::input::{GameInput, poll_input};
use crate::player::Player;
use crate::terminal::TerminalRenderer;
use crate::textures::NobiscuitTextures;

const MAX_DEPTH: f64 = 20.0;
const TARGET_FPS: u64 = 30;
const FRAME_DURATION: Duration = Duration::from_millis(1000 / TARGET_FPS);

fn main() {
    let mut term = TerminalRenderer::new();
    let (cols, rows) = term.size();
    let fb_width = cols;
    let fb_height = rows * 2;

    let mut fb = Framebuffer::new(fb_width, fb_height);

    let mut rng = rand::thread_rng();

    // Start with a dummy world; the real one is created after galagara
    let mut world = World::new(1, 3, 3, &mut rng);
    let mut player = Player::new(1.5, 1.5);
    let mut state = GameState::new();
    state.phase = GamePhase::GaragaraStart {
        spins: 0,
        shake_timer: 0.0,
    };

    let mut last_frame = Instant::now();

    loop {
        let now = Instant::now();
        let dt = now.duration_since(last_frame).as_secs_f64().min(0.1); // cap at 100ms
        last_frame = now;

        // Input
        let input = poll_input(Duration::from_millis(5));

        match state.phase {
            GamePhase::GaragaraStart {
                ref mut spins,
                ref mut shake_timer,
            } => {
                // Decrease shake timer
                *shake_timer = (*shake_timer - dt).max(0.0);

                match &input {
                    Some(GameInput::Quit) => break,
                    Some(GameInput::Confirm) if *spins > 0 => {
                        // Confirm: generate the real world from spin count
                        let params = game::maze_params_from_spins(*spins);
                        world =
                            World::new(params.num_floors, params.width, params.height, &mut rng);
                        player = Player::new(1.5, 1.5);
                        state = GameState::new();
                        state.init_visited(&world);
                        state.phase = GamePhase::Playing;
                        continue;
                    }
                    Some(GameInput::Confirm) => {
                        // spins == 0: treat Enter/Space as a spin too
                        *spins += 1;
                        *shake_timer = 0.3;
                    }
                    Some(
                        GameInput::MoveForward
                        | GameInput::MoveBackward
                        | GameInput::TurnLeft
                        | GameInput::TurnRight
                        | GameInput::ToggleMinimap
                        | GameInput::Retry
                        | GameInput::Decline
                        | GameInput::AnyKey,
                    ) => {
                        *spins += 1;
                        *shake_timer = 0.3;
                    }
                    _ => {}
                }
            }
            GamePhase::Playing => {
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
                        let (nx, ny) =
                            world.change_floor(transition.target_floor, transition.direction);
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
                                    // Restart: go back to galagara
                                    world = World::new(1, 3, 3, &mut rng);
                                    player = Player::new(1.5, 1.5);
                                    state = GameState::new();
                                    state.phase = GamePhase::GaragaraStart {
                                        spins: 0,
                                        shake_timer: 0.0,
                                    };
                                    continue;
                                }
                                Some(GameInput::Quit) | Some(GameInput::Decline) => break,
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

        match state.phase {
            GamePhase::GaragaraStart { spins, shake_timer } => {
                ui::render_garagara_screen(&mut fb, spins, shake_timer);
            }
            GamePhase::Playing => {
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

                        let tex = NobiscuitTextures;

                        // Floor and ceiling
                        render_floor_ceiling(
                            &mut fb,
                            &rays,
                            &tex,
                            &FlatHeightMap,
                            &player.camera,
                            MAX_DEPTH,
                        );

                        // Walls
                        render_walls(
                            &mut fb,
                            &rays,
                            &tex,
                            &FlatHeightMap,
                            &player.camera,
                            MAX_DEPTH,
                        );

                        // Sprites (biscuits + goal + stairs)
                        let projected = termray::project_sprites(
                            world.current_sprites(),
                            &player.camera,
                            &FlatHeightMap,
                            fb.width(),
                            fb.height(),
                        );
                        termray::render_sprites(&mut fb, &projected, &rays, &tex, MAX_DEPTH);

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
                            ui::render_floor_indicator(
                                &mut fb,
                                world.current_floor + 1,
                                world.floors.len(),
                            );
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
                            let factor = (timer / FADE_DURATION).clamp(0.0, 1.0);
                            fb.darken_all(factor);

                            // Starvation: shift frame down for collapse effect
                            if !state.is_alive {
                                let progress =
                                    ((FADE_DURATION - timer) / FADE_DURATION).clamp(0.0, 1.0);
                                let shift = (progress * fb.height() as f64 * 0.3) as usize;
                                fb.shift_down(shift);
                            }
                        }
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
