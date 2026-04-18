//! Nobiscuit map: dense grid with nobiscuit-aware `is_solid`.
//!
//! termray's bundled `GridMap` treats any non-EMPTY tile as solid, which is
//! the right default but not what nobiscuit needs: goals and stairs are
//! walkable, while window/shoji/doors are solid.

use termray::{TileMap, TileType};

use crate::tiles::{
    TILE_DOOR_FUSUMA, TILE_DOOR_GENKAN, TILE_DOOR_KITCHEN, TILE_DOOR_TOILET, TILE_EMPTY, TILE_GOAL,
    TILE_STAIRS_DOWN, TILE_STAIRS_UP, TILE_WALL,
};

pub struct NobiscuitMap {
    width: usize,
    height: usize,
    tiles: Vec<TileType>,
}

impl NobiscuitMap {
    /// Create a new map filled with walls (matches the old engine default).
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height, tiles: vec![TILE_WALL; width * height] }
    }

    pub fn set(&mut self, x: usize, y: usize, tile: TileType) {
        if x < self.width && y < self.height {
            self.tiles[y * self.width + x] = tile;
        }
    }
}

impl TileMap for NobiscuitMap {
    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn get(&self, x: i32, y: i32) -> Option<TileType> {
        if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
            Some(self.tiles[y as usize * self.width + x as usize])
        } else {
            None
        }
    }

    fn is_solid(&self, x: i32, y: i32) -> bool {
        match self.get(x, y) {
            Some(TILE_EMPTY) | Some(TILE_GOAL) | Some(TILE_STAIRS_UP) | Some(TILE_STAIRS_DOWN) => {
                false
            }
            Some(TILE_DOOR_FUSUMA)
            | Some(TILE_DOOR_KITCHEN)
            | Some(TILE_DOOR_TOILET)
            | Some(TILE_DOOR_GENKAN) => true,
            _ => true,
        }
    }
}
