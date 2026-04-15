use nobiscuit_engine::map::{GridMap, TileMap, TILE_EMPTY, TILE_GOAL, TILE_WALL};
use rand::seq::SliceRandom;
use rand::Rng;

/// Generate a perfect maze using iterative DFS backtracking.
///
/// Both `width` and `height` must be odd numbers (the algorithm steps by 2).
pub fn generate_maze(width: usize, height: usize, rng: &mut impl Rng) -> GridMap {
    assert!(width % 2 == 1 && height % 2 == 1, "maze dimensions must be odd");
    let mut map = GridMap::new(width, height);
    let mut stack: Vec<(usize, usize)> = Vec::new();

    map.set(1, 1, TILE_EMPTY);
    stack.push((1, 1));

    while let Some(&(x, y)) = stack.last() {
        let mut dirs: [(i32, i32); 4] = [(0, -2), (2, 0), (0, 2), (-2, 0)];
        dirs.shuffle(rng);

        let mut found = false;
        for (dx, dy) in dirs {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;

            if nx > 0
                && nx < (width - 1) as i32
                && ny > 0
                && ny < (height - 1) as i32
                && map.get(nx, ny) == Some(TILE_WALL)
            {
                map.set(
                    (x as i32 + dx / 2) as usize,
                    (y as i32 + dy / 2) as usize,
                    TILE_EMPTY,
                );
                map.set(nx as usize, ny as usize, TILE_EMPTY);
                stack.push((nx as usize, ny as usize));
                found = true;
                break;
            }
        }

        if !found {
            stack.pop();
        }
    }

    // Place goal at bottom-right path cell
    'outer: for y in (1..height - 1).rev() {
        for x in (1..width - 1).rev() {
            if map.get(x as i32, y as i32) == Some(TILE_EMPTY) {
                map.set(x, y, TILE_GOAL);
                break 'outer;
            }
        }
    }

    map
}
