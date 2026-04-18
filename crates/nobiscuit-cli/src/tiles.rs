//! Nobiscuit-specific tile IDs layered on top of termray's reserved trio.
//!
//! termray reserves 0..=2 for EMPTY / WALL / VOID. Nobiscuit uses 3..=11 for
//! its Japanese-house props.

pub use termray::{TileType, TILE_EMPTY, TILE_VOID, TILE_WALL};

pub const TILE_GOAL: TileType = 3;
pub const TILE_WINDOW: TileType = 4;
pub const TILE_STAIRS_UP: TileType = 5;
pub const TILE_STAIRS_DOWN: TileType = 6;
pub const TILE_DOOR_FUSUMA: TileType = 7;
pub const TILE_DOOR_KITCHEN: TileType = 8;
pub const TILE_DOOR_TOILET: TileType = 9;
pub const TILE_DOOR_GENKAN: TileType = 10;
pub const TILE_SHOJI: TileType = 11;
