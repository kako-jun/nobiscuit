use nobiscuit_engine::map::{TileMap, TILE_EMPTY, TILE_GOAL};
use nobiscuit_engine::sprite::Sprite;
use rand::seq::SliceRandom;
use rand::Rng;

pub const SPRITE_BISCUIT: u8 = 1;
pub const SPRITE_GOAL: u8 = 2;

pub struct GameState {
    pub show_minimap: bool,
    pub hunger: f64,        // 1.0 = full, 0.0 = dead
    pub hunger_drain: f64,  // per second
    pub biscuits_eaten: u32,
    pub is_alive: bool,
    pub escaped: bool,
    pub sprites: Vec<Sprite>,
    pub message: Option<(String, f64)>, // (text, remaining seconds)
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
            sprites: Vec::new(),
            message: None,
        }
    }

    /// Place biscuits on random empty tiles and a goal marker
    pub fn place_items(&mut self, map: &dyn TileMap, rng: &mut impl Rng) {
        // Collect all empty cells (not (1,1) where player starts)
        let mut empties: Vec<(usize, usize)> = Vec::new();
        let mut goal_pos = None;

        for y in 0..map.height() {
            for x in 0..map.width() {
                let tile = map.get(x as i32, y as i32).unwrap_or(1);
                if tile == TILE_EMPTY && !(x == 1 && y == 1) {
                    empties.push((x, y));
                } else if tile == TILE_GOAL {
                    goal_pos = Some((x, y));
                }
            }
        }

        // Place biscuits on ~30% of empty cells
        empties.shuffle(rng);
        let biscuit_count = (empties.len() / 3).max(5);
        for &(x, y) in empties.iter().take(biscuit_count) {
            self.sprites.push(Sprite {
                x: x as f64 + 0.5,
                y: y as f64 + 0.5,
                sprite_type: SPRITE_BISCUIT,
            });
        }

        // Goal sprite
        if let Some((gx, gy)) = goal_pos {
            self.sprites.push(Sprite {
                x: gx as f64 + 0.5,
                y: gy as f64 + 0.5,
                sprite_type: SPRITE_GOAL,
            });
        }
    }

    /// Update hunger, check biscuit pickup, check goal
    pub fn update(&mut self, player_x: f64, player_y: f64, dt: f64) {
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

        // Check biscuit pickup (within 0.5 units)
        let pickup_dist = 0.5;
        let mut i = 0;
        while i < self.sprites.len() {
            let s = &self.sprites[i];
            let dx = s.x - player_x;
            let dy = s.y - player_y;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist < pickup_dist && s.sprite_type == SPRITE_BISCUIT {
                self.sprites.remove(i);
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
