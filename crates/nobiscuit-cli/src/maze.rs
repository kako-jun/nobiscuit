use nobiscuit_engine::map::{
    GridMap, TileMap, TILE_EMPTY, TILE_GOAL, TILE_STAIRS_DOWN, TILE_STAIRS_UP, TILE_VOID,
    TILE_WALL, TILE_WINDOW,
};
use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::VecDeque;

/// Generate a mask indicating which cells are part of the playable area.
///
/// The mask is defined over the full grid (width × height). Outer border cells
/// are always `true` (kept as TILE_WALL). Interior cells that are `false` will
/// become TILE_VOID. The algorithm places random seed points on the DFS node
/// grid (odd coordinates) and expands them via BFS to create organic blobs.
fn generate_mask(width: usize, height: usize, rng: &mut impl Rng) -> Vec<bool> {
    let mut mask = vec![false; width * height];

    // Outer border is always true (wall boundary)
    for x in 0..width {
        mask[x] = true; // row 0
        mask[(height - 1) * width + x] = true; // last row
    }
    for y in 0..height {
        mask[y * width] = true; // col 0
        mask[y * width + (width - 1)] = true; // last col
    }

    // Collect all DFS nodes (odd coordinates in the interior)
    let mut all_nodes: Vec<(usize, usize)> = Vec::new();
    for y in (1..height - 1).step_by(2) {
        for x in (1..width - 1).step_by(2) {
            all_nodes.push((x, y));
        }
    }

    if all_nodes.is_empty() {
        return mask;
    }

    let total_nodes = all_nodes.len();
    let target_count = rng
        .gen_range(total_nodes * 40 / 100..=total_nodes * 70 / 100)
        .max(1);

    // Place 2-4 seed points
    let num_seeds = rng.gen_range(2..=4).min(total_nodes);
    all_nodes.shuffle(rng);
    let seeds: Vec<(usize, usize)> = all_nodes.iter().copied().take(num_seeds).collect();

    // Track which DFS nodes are selected
    let node_cols = (width - 1) / 2; // number of odd columns
    let node_index = |x: usize, y: usize| -> usize { (y / 2) * node_cols + (x / 2) };
    let mut selected = vec![false; (height / 2) * node_cols + node_cols];
    let mut selected_count = 0;

    let mut queue: VecDeque<(usize, usize)> = VecDeque::new();

    // Always include (1,1) as a seed to guarantee player spawn is valid
    {
        let ni = node_index(1, 1);
        if !selected[ni] {
            selected[ni] = true;
            selected_count += 1;
            queue.push_back((1, 1));
        }
    }

    for &(sx, sy) in &seeds {
        let ni = node_index(sx, sy);
        if !selected[ni] {
            selected[ni] = true;
            selected_count += 1;
            queue.push_back((sx, sy));
        }
    }

    // BFS expansion from seeds
    let dirs: [(i32, i32); 4] = [(2, 0), (-2, 0), (0, 2), (0, -2)];
    while selected_count < target_count {
        if queue.is_empty() {
            break;
        }
        let idx = rng.gen_range(0..queue.len());
        let (cx, cy) = queue[idx];

        // Try to expand to a random unselected neighbor
        let mut neighbor_dirs = dirs;
        neighbor_dirs.shuffle(rng);

        let mut expanded = false;
        for (dx, dy) in neighbor_dirs {
            let nx = cx as i32 + dx;
            let ny = cy as i32 + dy;
            if nx > 0 && nx < (width - 1) as i32 && ny > 0 && ny < (height - 1) as i32 {
                let nux = nx as usize;
                let nuy = ny as usize;
                let ni = node_index(nux, nuy);
                if !selected[ni] {
                    selected[ni] = true;
                    selected_count += 1;
                    queue.push_back((nux, nuy));
                    expanded = true;
                    break;
                }
            }
        }

        if !expanded {
            // This node has no unselected neighbors, remove from queue
            queue.swap_remove_back(idx);
        }
    }

    // Apply selected nodes to the mask: mark selected odd-coordinate cells
    // and the even-coordinate cells between adjacent selected nodes
    for y in (1..height - 1).step_by(2) {
        for x in (1..width - 1).step_by(2) {
            let ni = node_index(x, y);
            if selected[ni] {
                // Mark the node cell itself
                mask[y * width + x] = true;

                // Mark wall cells between this node and adjacent selected nodes
                for &(dx, dy) in &dirs {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx > 0 && nx < (width - 1) as i32 && ny > 0 && ny < (height - 1) as i32 {
                        let nux = nx as usize;
                        let nuy = ny as usize;
                        let nni = node_index(nux, nuy);
                        if selected[nni] {
                            // Mark the cell between them
                            let bx = (x as i32 + dx / 2) as usize;
                            let by = (y as i32 + dy / 2) as usize;
                            mask[by * width + bx] = true;
                        }
                    }
                }
            }
        }
    }

    mask
}

/// Find connected components (islands) among DFS nodes in the mask.
///
/// Returns a list of islands. Each island is a list of DFS node coordinates
/// (odd x, odd y) that are connected via adjacent mask-true cells.
fn find_islands(mask: &[bool], width: usize, height: usize) -> Vec<Vec<(usize, usize)>> {
    let node_cols = (width - 1) / 2;
    let node_index = |x: usize, y: usize| -> usize { (y / 2) * node_cols + (x / 2) };
    let max_nodes = (height / 2) * node_cols + node_cols;
    let mut visited = vec![false; max_nodes];
    let mut islands = Vec::new();

    let dirs: [(i32, i32); 4] = [(2, 0), (-2, 0), (0, 2), (0, -2)];

    for y in (1..height - 1).step_by(2) {
        for x in (1..width - 1).step_by(2) {
            if !mask[y * width + x] {
                continue;
            }
            let ni = node_index(x, y);
            if visited[ni] {
                continue;
            }

            // BFS to find all nodes in this island
            let mut island = Vec::new();
            let mut queue = VecDeque::new();
            visited[ni] = true;
            queue.push_back((x, y));

            while let Some((cx, cy)) = queue.pop_front() {
                island.push((cx, cy));

                for &(dx, dy) in &dirs {
                    let nx = cx as i32 + dx;
                    let ny = cy as i32 + dy;
                    if nx > 0 && nx < (width - 1) as i32 && ny > 0 && ny < (height - 1) as i32 {
                        let nux = nx as usize;
                        let nuy = ny as usize;
                        if !mask[nuy * width + nux] {
                            continue;
                        }
                        // Check that the wall cell between them is also in mask
                        let bx = (cx as i32 + dx / 2) as usize;
                        let by = (cy as i32 + dy / 2) as usize;
                        if !mask[by * width + bx] {
                            continue;
                        }
                        let nni = node_index(nux, nuy);
                        if !visited[nni] {
                            visited[nni] = true;
                            queue.push_back((nux, nuy));
                        }
                    }
                }
            }

            islands.push(island);
        }
    }

    islands
}

/// Run DFS maze carving on a single island.
///
/// `island_nodes` contains the DFS node coordinates belonging to this island.
/// The maze is carved in-place on `map`. Only nodes within the island's mask
/// are considered valid neighbors.
fn carve_island(
    map: &mut GridMap,
    island_nodes: &[(usize, usize)],
    width: usize,
    height: usize,
    rng: &mut impl Rng,
) {
    if island_nodes.is_empty() {
        return;
    }

    // Build a set of island nodes for quick lookup
    let node_cols = (width - 1) / 2;
    let node_index = |x: usize, y: usize| -> usize { (y / 2) * node_cols + (x / 2) };
    let max_nodes = (height / 2) * node_cols + node_cols;
    let mut in_island = vec![false; max_nodes];
    for &(x, y) in island_nodes {
        in_island[node_index(x, y)] = true;
    }

    // Pick a random starting node
    let start_idx = rng.gen_range(0..island_nodes.len());
    let (sx, sy) = island_nodes[start_idx];

    map.set(sx, sy, TILE_EMPTY);
    let mut stack: Vec<(usize, usize)> = vec![(sx, sy)];
    let mut carved = vec![false; max_nodes];
    carved[node_index(sx, sy)] = true;

    while let Some(&(x, y)) = stack.last() {
        let mut dirs: [(i32, i32); 4] = [(0, -2), (2, 0), (0, 2), (-2, 0)];
        dirs.shuffle(rng);

        let mut found = false;
        for (dx, dy) in dirs {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;

            if nx > 0 && nx < (width - 1) as i32 && ny > 0 && ny < (height - 1) as i32 {
                let nux = nx as usize;
                let nuy = ny as usize;
                let ni = node_index(nux, nuy);
                if in_island[ni] && !carved[ni] {
                    // Carve the wall between current and neighbor
                    let bx = (x as i32 + dx / 2) as usize;
                    let by = (y as i32 + dy / 2) as usize;
                    map.set(bx, by, TILE_EMPTY);
                    map.set(nux, nuy, TILE_EMPTY);
                    carved[ni] = true;
                    stack.push((nux, nuy));
                    found = true;
                    break;
                }
            }
        }

        if !found {
            stack.pop();
        }
    }
}

/// Generate a maze with irregular shape using mask-based generation.
///
/// Both `width` and `height` must be odd numbers (the algorithm steps by 2).
pub fn generate_maze(width: usize, height: usize, rng: &mut impl Rng) -> GridMap {
    assert!(
        width % 2 == 1 && height % 2 == 1,
        "maze dimensions must be odd"
    );
    let mut map = GridMap::new(width, height);

    // Generate mask for this floor's shape
    let mask = generate_mask(width, height, rng);

    // Set all non-masked interior cells to VOID
    for y in 1..height - 1 {
        for x in 1..width - 1 {
            if !mask[y * width + x] {
                map.set(x, y, TILE_VOID);
            }
        }
    }

    // Find islands (connected components of DFS nodes)
    let islands = find_islands(&mask, width, height);

    // Carve maze independently on each island
    for island in &islands {
        carve_island(&mut map, island, width, height, rng);
    }

    // Place windows on some interior walls that border a corridor
    place_windows(&mut map, width, height, rng);

    // Place goal on the largest island's furthest cell from its start
    let largest_island = islands.iter().max_by_key(|i| i.len());
    if let Some(island) = largest_island {
        // Find bottom-right-most empty cell in this island
        let mut goal_placed = false;
        for &(x, y) in island.iter().rev() {
            if map.get(x as i32, y as i32) == Some(TILE_EMPTY) {
                map.set(x, y, TILE_GOAL);
                goal_placed = true;
                break;
            }
        }
        if !goal_placed {
            // Fallback: any empty cell
            'outer: for y in (1..height - 1).rev() {
                for x in (1..width - 1).rev() {
                    if map.get(x as i32, y as i32) == Some(TILE_EMPTY) {
                        map.set(x, y, TILE_GOAL);
                        break 'outer;
                    }
                }
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

/// Collect empty cells on a specific island for item placement.
fn collect_island_empties(
    map: &dyn TileMap,
    island: &[(usize, usize)],
    exclude: &[(usize, usize)],
) -> Vec<(usize, usize)> {
    let mut empties = Vec::new();
    for &(x, y) in island {
        if map.get(x as i32, y as i32) == Some(TILE_EMPTY) && !exclude.contains(&(x, y)) {
            empties.push((x, y));
        }
    }
    empties
}

/// Generate a maze for a specific floor with stairs placed automatically.
/// Each island gets at least one stair to enable cross-island traversal via floors.
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

    // Rebuild the mask from the map (cells that are not VOID)
    let mask: Vec<bool> = (0..width * height)
        .map(|i| {
            let x = (i % width) as i32;
            let y = (i / width) as i32;
            map.get(x, y) != Some(TILE_VOID)
        })
        .collect();

    // Find islands for stair placement
    let islands = find_islands(&mask, width, height);

    // Place stairs: each island gets at least one stair (if applicable)
    let start_pos = (1usize, 1usize);

    // Place stairs down (not on ground floor)
    if floor_index > 0 {
        for island in &islands {
            let mut empties = collect_island_empties(&map, island, &[start_pos]);
            empties.shuffle(rng);
            if let Some(&(x, y)) = empties.first() {
                map.set(x, y, TILE_STAIRS_DOWN);
            }
        }
    }

    // Place stairs up (not on top floor)
    if floor_index < total_floors - 1 {
        for island in &islands {
            let exclude: Vec<(usize, usize)> = {
                let mut v = vec![start_pos];
                // Exclude cells already used for stairs down
                for &(nx, ny) in island {
                    if map.get(nx as i32, ny as i32) == Some(TILE_STAIRS_DOWN) {
                        v.push((nx, ny));
                    }
                }
                v
            };
            let mut empties = collect_island_empties(&map, island, &exclude);
            empties.shuffle(rng);
            if let Some(&(x, y)) = empties.first() {
                map.set(x, y, TILE_STAIRS_UP);
            }
        }
    }

    map
}
