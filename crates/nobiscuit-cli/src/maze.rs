use nobiscuit_engine::map::{GridMap, TileMap, TILE_EMPTY, TILE_GOAL, TILE_STAIRS_DOWN, TILE_STAIRS_UP, TILE_WALL, TILE_WINDOW};
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

    // Place windows on some interior walls that border a corridor
    place_windows(&mut map, width, height, rng);

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

/// Convert some interior walls into windows.
/// A wall becomes a window candidate if it has at least one empty neighbor
/// (it's visible from a corridor). ~15% of candidates become windows.
fn place_windows(map: &mut GridMap, width: usize, height: usize, rng: &mut impl Rng) {
    let mut candidates: Vec<(usize, usize)> = Vec::new();

    // Skip outer border (row/col 0 and last)
    for y in 2..height - 2 {
        for x in 2..width - 2 {
            if map.get(x as i32, y as i32) != Some(TILE_WALL) {
                continue;
            }
            // Check if adjacent to at least one empty tile
            let neighbors = [
                map.get(x as i32 - 1, y as i32),
                map.get(x as i32 + 1, y as i32),
                map.get(x as i32, y as i32 - 1),
                map.get(x as i32, y as i32 + 1),
            ];
            let has_corridor = neighbors.contains(&Some(TILE_EMPTY));
            if has_corridor {
                candidates.push((x, y));
            }
        }
    }

    candidates.shuffle(rng);
    let window_count = (candidates.len() / 7).max(3);
    for &(x, y) in candidates.iter().take(window_count) {
        map.set(x, y, TILE_WINDOW);
    }
}

/// Generate a maze for a specific floor with stairs placed automatically.
/// Stairs up/down are placed on random empty cells based on floor position.
pub fn generate_floor(
    width: usize,
    height: usize,
    floor_index: usize,
    total_floors: usize,
    rng: &mut impl Rng,
) -> GridMap {
    let is_top_floor = floor_index == total_floors - 1;
    let mut map = generate_maze(width, height, rng);

    // Remove goal from non-top floors (goal only on top floor)
    if !is_top_floor {
        for y in 0..height {
            for x in 0..width {
                if map.get(x as i32, y as i32) == Some(TILE_GOAL) {
                    map.set(x, y, TILE_EMPTY);
                }
            }
        }
    }

    // Collect empty cells for stair placement (not start pos, not goal)
    let mut empties: Vec<(usize, usize)> = Vec::new();
    for y in 1..height - 1 {
        for x in 1..width - 1 {
            if map.get(x as i32, y as i32) == Some(TILE_EMPTY) && !(x == 1 && y == 1) {
                empties.push((x, y));
            }
        }
    }
    empties.shuffle(rng);

    // Place stairs up (not on ground floor, i.e. floor_index > 0)
    if floor_index > 0 {
        if let Some(&(x, y)) = empties.first() {
            map.set(x, y, TILE_STAIRS_DOWN);
            empties.remove(0);
        }
    }

    // Place stairs up to next floor (not on top floor)
    if floor_index < total_floors - 1 {
        if let Some(&(x, y)) = empties.first() {
            map.set(x, y, TILE_STAIRS_UP);
        }
    }

    map
}
