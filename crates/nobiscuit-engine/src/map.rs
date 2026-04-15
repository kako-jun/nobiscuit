pub type TileType = u8;

pub const TILE_EMPTY: u8 = 0;
pub const TILE_WALL: u8 = 1;
pub const TILE_GOAL: u8 = 2;
pub const TILE_WINDOW: u8 = 3;
pub const TILE_STAIRS_UP: u8 = 4;
pub const TILE_STAIRS_DOWN: u8 = 5;

pub trait TileMap {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn get(&self, x: i32, y: i32) -> Option<TileType>;
    fn is_solid(&self, x: i32, y: i32) -> bool;
}

pub struct GridMap {
    width: usize,
    height: usize,
    tiles: Vec<TileType>,
}

impl GridMap {
    /// Create a new grid map filled with walls
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

impl TileMap for GridMap {
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
            _ => true, // walls, windows, and out-of-bounds are solid
        }
    }
}
