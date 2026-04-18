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
        Self {
            width,
            height,
            tiles: vec![TILE_WALL; width * height],
        }
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
        // Doors are kept as an explicit arm (instead of falling through to `_ => true`)
        // so the intent — "doors are solid while closed" — stays visible in the source.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tiles::{TILE_SHOJI, TILE_VOID, TILE_WINDOW};

    #[test]
    fn defaults_to_walls_and_walls_are_solid() {
        let m = NobiscuitMap::new(3, 3);
        assert_eq!(m.get(0, 0), Some(TILE_WALL));
        assert!(m.is_solid(0, 0));
    }

    #[test]
    fn walkable_tiles() {
        let mut m = NobiscuitMap::new(4, 4);
        m.set(0, 0, TILE_EMPTY);
        m.set(1, 0, TILE_GOAL);
        m.set(2, 0, TILE_STAIRS_UP);
        m.set(3, 0, TILE_STAIRS_DOWN);
        for x in 0..4 {
            assert!(!m.is_solid(x, 0), "tile at x={x} should be walkable");
        }
    }

    #[test]
    fn solid_tiles() {
        let mut m = NobiscuitMap::new(7, 1);
        m.set(0, 0, TILE_WALL);
        m.set(1, 0, TILE_VOID);
        m.set(2, 0, TILE_WINDOW);
        m.set(3, 0, TILE_SHOJI);
        m.set(4, 0, TILE_DOOR_FUSUMA);
        m.set(5, 0, TILE_DOOR_KITCHEN);
        m.set(6, 0, TILE_DOOR_GENKAN);
        for x in 0..7 {
            assert!(m.is_solid(x, 0), "tile at x={x} should be solid");
        }
    }

    #[test]
    fn out_of_bounds_is_none_and_solid() {
        let m = NobiscuitMap::new(2, 2);
        assert_eq!(m.get(-1, 0), None);
        assert_eq!(m.get(5, 5), None);
        assert!(m.is_solid(-1, 0));
        assert!(m.is_solid(5, 5));
    }
}
