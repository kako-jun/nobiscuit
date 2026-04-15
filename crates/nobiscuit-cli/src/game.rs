use nobiscuit_engine::map::{
    GridMap, TileMap, TILE_DOOR_FUSUMA, TILE_DOOR_GENKAN, TILE_DOOR_KITCHEN, TILE_DOOR_TOILET,
    TILE_EMPTY, TILE_GOAL, TILE_STAIRS_DOWN, TILE_STAIRS_UP,
};
use nobiscuit_engine::sprite::Sprite;
use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::HashMap;

use crate::maze;

pub const SPRITE_BISCUIT: u8 = 1;
pub const SPRITE_GOAL: u8 = 2;
pub const SPRITE_STAIRS_UP: u8 = 3;
pub const SPRITE_STAIRS_DOWN: u8 = 4;

/// A floor in the world: its map and its sprites
pub struct Floor {
    pub map: GridMap,
    pub sprites: Vec<Sprite>,
}

/// Multiple floors connected by stairs
pub struct World {
    pub floors: Vec<Floor>,
    pub current_floor: usize,
}

impl World {
    /// Generate a world with multiple floors
    pub fn new(num_floors: usize, maze_w: usize, maze_h: usize, rng: &mut impl Rng) -> Self {
        let mut floors = Vec::new();

        for i in 0..num_floors {
            let map = maze::generate_floor(maze_w, maze_h, i, num_floors, rng);
            let sprites = place_floor_items(&map, i == 0, rng);
            floors.push(Floor { map, sprites });
        }

        World {
            floors,
            current_floor: 0,
        }
    }

    pub fn current_map(&self) -> &GridMap {
        &self.floors[self.current_floor].map
    }

    pub fn current_map_mut(&mut self) -> &mut GridMap {
        &mut self.floors[self.current_floor].map
    }

    pub fn current_sprites(&self) -> &[Sprite] {
        &self.floors[self.current_floor].sprites
    }

    pub fn current_sprites_mut(&mut self) -> &mut Vec<Sprite> {
        &mut self.floors[self.current_floor].sprites
    }

    /// Check if player is on stairs; return new floor index if so
    pub fn check_stairs(&self, player_x: f64, player_y: f64) -> Option<StairTransition> {
        let px = player_x as i32;
        let py = player_y as i32;
        let tile = self.current_map().get(px, py)?;

        match tile {
            TILE_STAIRS_UP if self.current_floor < self.floors.len() - 1 => Some(StairTransition {
                target_floor: self.current_floor + 1,
                direction: StairDirection::Up,
            }),
            TILE_STAIRS_DOWN if self.current_floor > 0 => Some(StairTransition {
                target_floor: self.current_floor - 1,
                direction: StairDirection::Down,
            }),
            _ => None,
        }
    }

    /// Move player to a different floor. Returns the spawn position on the new floor.
    ///
    /// Scans the full map for the first matching stair tile. With multiple islands
    /// per floor, the player may land on a different island than expected — this is
    /// intentional, creating the "wandering between islands" exploration effect.
    pub fn change_floor(&mut self, target_floor: usize, direction: StairDirection) -> (f64, f64) {
        self.current_floor = target_floor;

        // Find the matching stairs on the new floor (first match wins)
        let target_tile = match direction {
            StairDirection::Up => TILE_STAIRS_DOWN, // came up, so land on down-stairs
            StairDirection::Down => TILE_STAIRS_UP, // went down, land on up-stairs
        };

        let map = &self.floors[target_floor].map;
        for y in 0..map.height() {
            for x in 0..map.width() {
                if map.get(x as i32, y as i32) == Some(target_tile) {
                    return (x as f64 + 0.5, y as f64 + 0.5);
                }
            }
        }

        // Fallback: find any walkable cell
        for y in 1..map.height() - 1 {
            for x in 1..map.width() - 1 {
                if map.get(x as i32, y as i32) == Some(TILE_EMPTY) {
                    return (x as f64 + 0.5, y as f64 + 0.5);
                }
            }
        }
        (1.5, 1.5)
    }
}

pub struct StairTransition {
    pub target_floor: usize,
    pub direction: StairDirection,
}

#[derive(Clone, Copy)]
pub enum StairDirection {
    Up,
    Down,
}

/// Place biscuits, goal sprite, and stair sprites on a floor
fn place_floor_items(map: &dyn TileMap, is_ground_floor: bool, rng: &mut impl Rng) -> Vec<Sprite> {
    let mut sprites = Vec::new();
    let mut empties: Vec<(usize, usize)> = Vec::new();
    let mut goal_pos = None;

    let start_pos: Option<(usize, usize)> = if is_ground_floor { Some((1, 1)) } else { None };

    for y in 0..map.height() {
        for x in 0..map.width() {
            let tile = map.get(x as i32, y as i32).unwrap_or(1);
            match tile {
                TILE_EMPTY if start_pos.is_none_or(|(sx, sy)| !(x == sx && y == sy)) => {
                    empties.push((x, y));
                }
                TILE_GOAL => {
                    goal_pos = Some((x, y));
                }
                TILE_STAIRS_UP => {
                    sprites.push(Sprite {
                        x: x as f64 + 0.5,
                        y: y as f64 + 0.5,
                        sprite_type: SPRITE_STAIRS_UP,
                    });
                }
                TILE_STAIRS_DOWN => {
                    sprites.push(Sprite {
                        x: x as f64 + 0.5,
                        y: y as f64 + 0.5,
                        sprite_type: SPRITE_STAIRS_DOWN,
                    });
                }
                _ => {}
            }
        }
    }

    // Place biscuits on ~30% of empty cells
    empties.shuffle(rng);
    let biscuit_count = (empties.len() / 3).max(5);
    for &(x, y) in empties.iter().take(biscuit_count) {
        sprites.push(Sprite {
            x: x as f64 + 0.5,
            y: y as f64 + 0.5,
            sprite_type: SPRITE_BISCUIT,
        });
    }

    // Goal sprite
    if let Some((gx, gy)) = goal_pos {
        sprites.push(Sprite {
            x: gx as f64 + 0.5,
            y: gy as f64 + 0.5,
            sprite_type: SPRITE_GOAL,
        });
    }

    sprites
}

pub struct GameState {
    pub show_minimap: bool,
    pub hunger: f64,       // 1.0 = full, 0.0 = dead
    pub hunger_drain: f64, // per second
    pub biscuits_eaten: u32,
    pub is_alive: bool,
    pub escaped: bool,
    pub message: Option<(String, f64)>, // (text, remaining seconds)
    pub floor_transition: Option<StairTransition>,
    /// Prevent stair re-trigger: true while player is still on the stair tile they arrived on
    on_stair_tile: bool,
    /// Doors that are currently open: (x, y) -> original door tile type
    open_doors: HashMap<(usize, usize), u8>,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            show_minimap: true,
            hunger: 1.0,
            hunger_drain: 0.02, // ~50 seconds to starve
            biscuits_eaten: 0,
            is_alive: true,
            escaped: false,
            message: None,
            floor_transition: None,
            on_stair_tile: false,
            open_doors: HashMap::new(),
        }
    }

    /// Called after a floor transition completes to arm the bounce guard
    pub fn mark_on_stair(&mut self) {
        self.on_stair_tile = true;
    }

    /// Update hunger, check biscuit pickup, check goal, check stairs
    pub fn update(&mut self, world: &mut World, player_x: f64, player_y: f64, dt: f64) {
        if !self.is_alive || self.escaped {
            return;
        }

        // Drain hunger
        self.hunger -= self.hunger_drain * dt;
        if self.hunger <= 0.0 {
            self.hunger = 0.0;
            self.is_alive = false;
            self.message = Some(("You starved...".to_string(), 5.0));
            return;
        }

        // Auto-open doors adjacent to player
        let px_i = player_x as usize;
        let py_i = player_y as usize;
        let door_tiles = [
            TILE_DOOR_FUSUMA,
            TILE_DOOR_KITCHEN,
            TILE_DOOR_TOILET,
            TILE_DOOR_GENKAN,
        ];
        for &(dx, dy) in &[(1i32, 0i32), (-1, 0), (0, 1), (0, -1)] {
            let nx = px_i as i32 + dx;
            let ny = py_i as i32 + dy;
            if let Some(tile) = world.current_map().get(nx, ny) {
                if door_tiles.contains(&tile) {
                    let pos = (nx as usize, ny as usize);
                    self.open_doors.insert(pos, tile);
                    world.current_map_mut().set(pos.0, pos.1, TILE_EMPTY);
                }
            }
        }

        // Auto-close doors that are far from player (manhattan distance >= 3)
        let close_list: Vec<(usize, usize)> = self
            .open_doors
            .keys()
            .filter(|&&(dx, dy)| {
                let dist = (dx as i32 - px_i as i32).unsigned_abs()
                    + (dy as i32 - py_i as i32).unsigned_abs();
                dist >= 3
            })
            .copied()
            .collect();
        for pos in close_list {
            if let Some(original_tile) = self.open_doors.remove(&pos) {
                // Only restore if the cell is still empty (no sprite/item placed there)
                if world.current_map().get(pos.0 as i32, pos.1 as i32) == Some(TILE_EMPTY) {
                    world.current_map_mut().set(pos.0, pos.1, original_tile);
                }
            }
        }

        // Check stairs (with bounce guard)
        let currently_on_stair = world.check_stairs(player_x, player_y).is_some();
        if self.on_stair_tile && !currently_on_stair {
            // Player stepped off the arrival stair — re-enable stair detection
            self.on_stair_tile = false;
        }

        if self.floor_transition.is_none() && !self.on_stair_tile {
            if let Some(transition) = world.check_stairs(player_x, player_y) {
                let floor_name = match transition.direction {
                    StairDirection::Up => {
                        format!("Going up to {}F...", transition.target_floor + 1)
                    }
                    StairDirection::Down => {
                        format!("Going down to {}F...", transition.target_floor + 1)
                    }
                };
                self.message = Some((floor_name, 2.0));
                self.floor_transition = Some(transition);
                return;
            }
        }

        // Check biscuit pickup (within 0.5 units)
        let pickup_dist = 0.5;
        let sprites = world.current_sprites_mut();
        let mut i = 0;
        while i < sprites.len() {
            let s = &sprites[i];
            let dx = s.x - player_x;
            let dy = s.y - player_y;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist < pickup_dist && s.sprite_type == SPRITE_BISCUIT {
                sprites.swap_remove(i);
                self.biscuits_eaten += 1;
                self.hunger = (self.hunger + 0.15).min(1.0);
                self.message = Some(("*crunch*".to_string(), 1.0));
            } else if dist < pickup_dist && s.sprite_type == SPRITE_GOAL {
                self.escaped = true;
                self.message = Some(("Escaped! ...no biscuit.".to_string(), 10.0));
                return;
            } else {
                i += 1;
            }
        }

        // Decay message timer
        if let Some((_, ref mut timer)) = self.message {
            *timer -= dt;
            if *timer <= 0.0 {
                self.message = None;
            }
        }
    }
}
