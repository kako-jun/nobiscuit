use nobiscuit_engine::map::{
    GridMap, TileMap, TILE_DOOR_FUSUMA, TILE_DOOR_GENKAN, TILE_DOOR_KITCHEN, TILE_DOOR_TOILET,
    TILE_EMPTY, TILE_GOAL, TILE_SHOJI, TILE_STAIRS_DOWN, TILE_STAIRS_UP, TILE_VOID, TILE_WALL,
    TILE_WINDOW,
};
use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::VecDeque;

/// Player spawn position (grid coordinates). Used for stair exclusion and seed guarantee.
const PLAYER_START: (usize, usize) = (1, 1);

/// DFS neighbor directions (step by 2 on the node grid).
const DFS_DIRS: [(i32, i32); 4] = [(2, 0), (-2, 0), (0, 2), (0, -2)];

/// 4-directional neighbors (step by 1).
const DIRS4: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

/// Maximum contiguous empty area allowed when removing a wall for widening.
const MAX_OPEN_AREA: usize = 12;

/// A placed room (interior coordinates in map space).
pub struct Room {
    pub x: usize, // left column of interior
    pub y: usize, // top row of interior
    pub w: usize, // interior width
    pub h: usize, // interior height
}

/// Check if a cell (cx, cy) is on the border of any room (the wall cells
/// immediately surrounding a room's interior).
fn is_room_border(rooms: &[Room], cx: usize, cy: usize) -> bool {
    for r in rooms {
        // Room border: one cell outside the interior in each direction
        let bx0 = r.x.saturating_sub(1);
        let by0 = r.y.saturating_sub(1);
        let bx1 = r.x + r.w; // one past right edge
        let by1 = r.y + r.h; // one past bottom edge
        if cx >= bx0 && cx <= bx1 && cy >= by0 && cy <= by1 {
            // Inside the border rectangle but not inside the interior
            let in_interior = cx >= r.x && cx < r.x + r.w && cy >= r.y && cy < r.y + r.h;
            if !in_interior {
                return true;
            }
        }
    }
    false
}

/// Compute the flat index for a DFS node at odd coordinates (x, y).
/// `node_cols` is `(map_width - 1) / 2`.
fn node_index(x: usize, y: usize, node_cols: usize) -> usize {
    (y / 2) * node_cols + (x / 2)
}

/// Maximum number of nodes for a given grid size (used for visited/selected arrays).
fn max_node_count(width: usize, height: usize) -> usize {
    let node_cols = (width - 1) / 2;
    (height / 2) * node_cols + node_cols
}

/// Generate a mask indicating which cells are part of the playable area.
///
/// The mask is defined over the full grid (width × height). Outer border cells
/// are always `true` (kept as TILE_WALL). Interior cells that are `false` will
/// become TILE_VOID. The algorithm places random seed points on the DFS node
/// grid (odd coordinates) and expands them via BFS to create organic blobs.
///
/// PLAYER_START is always included as a seed to guarantee the spawn tile is valid.
/// Because BFS expands from PLAYER_START, it will always have neighbors in its island,
/// so single-node islands at the spawn position cannot occur.
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
    let node_cols = (width - 1) / 2;
    let mut selected = vec![false; max_node_count(width, height)];
    let mut selected_count = 0;

    let mut queue: VecDeque<(usize, usize)> = VecDeque::new();

    // Always include PLAYER_START as a seed to guarantee player spawn is valid
    {
        let ni = node_index(PLAYER_START.0, PLAYER_START.1, node_cols);
        if !selected[ni] {
            selected[ni] = true;
            selected_count += 1;
            queue.push_back(PLAYER_START);
        }
    }

    for &(sx, sy) in &seeds {
        let ni = node_index(sx, sy, node_cols);
        if !selected[ni] {
            selected[ni] = true;
            selected_count += 1;
            queue.push_back((sx, sy));
        }
    }

    // BFS expansion from seeds
    while selected_count < target_count {
        if queue.is_empty() {
            break;
        }
        let idx = rng.gen_range(0..queue.len());
        let (cx, cy) = queue[idx];

        // Try to expand to a random unselected neighbor
        let mut neighbor_dirs = DFS_DIRS;
        neighbor_dirs.shuffle(rng);

        let mut expanded = false;
        for (dx, dy) in neighbor_dirs {
            let nx = cx as i32 + dx;
            let ny = cy as i32 + dy;
            if nx > 0 && nx < (width - 1) as i32 && ny > 0 && ny < (height - 1) as i32 {
                let nux = nx as usize;
                let nuy = ny as usize;
                let ni = node_index(nux, nuy, node_cols);
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
            let ni = node_index(x, y, node_cols);
            if selected[ni] {
                // Mark the node cell itself
                mask[y * width + x] = true;

                // Mark wall cells between this node and adjacent selected nodes
                for &(dx, dy) in &DFS_DIRS {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx > 0 && nx < (width - 1) as i32 && ny > 0 && ny < (height - 1) as i32 {
                        let nux = nx as usize;
                        let nuy = ny as usize;
                        let nni = node_index(nux, nuy, node_cols);
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
    let mut visited = vec![false; max_node_count(width, height)];
    let mut islands = Vec::new();

    for y in (1..height - 1).step_by(2) {
        for x in (1..width - 1).step_by(2) {
            if !mask[y * width + x] {
                continue;
            }
            let ni = node_index(x, y, node_cols);
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

                for &(dx, dy) in &DFS_DIRS {
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
                        let nni = node_index(nux, nuy, node_cols);
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

/// Place small rooms randomly within the masked area.
///
/// Rooms have interior sizes ranging from 2×2 to 4×3. They are placed so that
/// all interior cells fall within the mask (not VOID) and do not overlap
/// existing room interiors. Rooms may be adjacent with only a wall between them.
///
/// Room coordinates are not restricted to odd positions — both odd and even cells
/// are set to TILE_EMPTY. DFS connection works via odd-coordinate nodes inside
/// the room (pre-carved in `carve_island`), which then bridge to adjacent corridors.
fn place_rooms(
    map: &mut GridMap,
    mask: &[bool],
    width: usize,
    height: usize,
    rng: &mut impl Rng,
) -> Vec<Room> {
    let mut rooms: Vec<Room> = Vec::new();
    let max_attempts = 80;
    // Room interior sizes: (w, h) from 2×2 to 4×3
    let sizes: [(usize, usize); 5] = [(2, 2), (3, 2), (2, 3), (3, 3), (4, 3)];

    for _ in 0..max_attempts {
        let &(rw, rh) = sizes.choose(rng).unwrap();
        // Interior must fit within border (1..width-1, 1..height-1)
        if rw + 2 > width || rh + 2 > height {
            continue;
        }
        let rx = rng.gen_range(1..width - 1 - rw + 1);
        let ry = rng.gen_range(1..height - 1 - rh + 1);

        // Check all interior cells are in mask
        let mut fits = true;
        'check_mask: for dy in 0..rh {
            for dx in 0..rw {
                if !mask[(ry + dy) * width + (rx + dx)] {
                    fits = false;
                    break 'check_mask;
                }
            }
        }
        if !fits {
            continue;
        }

        // Check no overlap with existing room interiors
        let mut overlaps = false;
        'check_overlap: for room in &rooms {
            // Check if the two room interiors overlap
            if rx < room.x + room.w && rx + rw > room.x && ry < room.y + room.h && ry + rh > room.y
            {
                overlaps = true;
                break 'check_overlap;
            }
        }
        if overlaps {
            continue;
        }

        // Place room: set all interior cells to EMPTY
        for dy in 0..rh {
            for dx in 0..rw {
                map.set(rx + dx, ry + dy, TILE_EMPTY);
            }
        }

        rooms.push(Room {
            x: rx,
            y: ry,
            w: rw,
            h: rh,
        });
    }

    rooms
}

/// Measure the contiguous empty area reachable from (sx, sy) via flood fill.
fn flood_fill_count(map: &GridMap, width: usize, height: usize, sx: usize, sy: usize) -> usize {
    let mut visited = vec![false; width * height];
    let mut queue = VecDeque::new();
    visited[sy * width + sx] = true;
    queue.push_back((sx, sy));
    let mut count = 0;

    while let Some((cx, cy)) = queue.pop_front() {
        count += 1;
        for &(dx, dy) in &DIRS4 {
            let nx = cx as i32 + dx;
            let ny = cy as i32 + dy;
            if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                let nux = nx as usize;
                let nuy = ny as usize;
                let idx = nuy * width + nux;
                if !visited[idx] && map.get(nx, ny) == Some(TILE_EMPTY) {
                    visited[idx] = true;
                    queue.push_back((nux, nuy));
                }
            }
        }
    }

    count
}

/// Widen some corridors by selectively removing walls.
///
/// Candidate walls are interior TILE_WALL cells adjacent to 2+ TILE_EMPTY cells.
/// Room border walls are excluded to preserve the corridor-hub structure.
/// A subset of candidates is processed: each wall is tentatively removed and
/// a flood fill checks whether the resulting open area exceeds MAX_OPEN_AREA.
/// If it does, the wall is restored.
fn widen_corridors(
    map: &mut GridMap,
    mask: &[bool],
    rooms: &[Room],
    width: usize,
    height: usize,
    rng: &mut impl Rng,
) {
    let mut candidates: Vec<(usize, usize)> = Vec::new();

    for y in 1..height - 1 {
        for x in 1..width - 1 {
            if !mask[y * width + x] {
                continue;
            }
            if map.get(x as i32, y as i32) != Some(TILE_WALL) {
                continue;
            }
            // Skip room border walls to preserve corridor-hub structure
            if is_room_border(rooms, x, y) {
                continue;
            }
            // Count adjacent empty cells
            let mut empty_neighbors = 0;
            for &(dx, dy) in &DIRS4 {
                if map.get(x as i32 + dx, y as i32 + dy) == Some(TILE_EMPTY) {
                    empty_neighbors += 1;
                }
            }
            if empty_neighbors >= 2 {
                candidates.push((x, y));
            }
        }
    }

    candidates.shuffle(rng);

    // Process 15-25% of candidates
    let process_count = if candidates.is_empty() {
        0
    } else {
        let pct = rng.gen_range(15..=25);
        (candidates.len() * pct / 100).max(1)
    };

    for &(x, y) in candidates.iter().take(process_count) {
        // Tentatively remove the wall
        map.set(x, y, TILE_EMPTY);

        let area = flood_fill_count(map, width, height, x, y);
        if area > MAX_OPEN_AREA {
            // Too large — restore the wall
            map.set(x, y, TILE_WALL);
        }
    }
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
    let mut in_island = vec![false; max_node_count(width, height)];
    for &(x, y) in island_nodes {
        in_island[node_index(x, y, node_cols)] = true;
    }

    // Pick a random starting node
    let start_idx = rng.gen_range(0..island_nodes.len());
    let (sx, sy) = island_nodes[start_idx];

    map.set(sx, sy, TILE_EMPTY);
    let mut stack: Vec<(usize, usize)> = vec![(sx, sy)];
    let mut carved = vec![false; max_node_count(width, height)];
    carved[node_index(sx, sy, node_cols)] = true;

    // Pre-mark nodes that are already EMPTY (inside rooms) as carved
    // but do NOT push to stack — rooms must stay walled off from corridors.
    // Doors will connect them later.
    for &(nx, ny) in island_nodes {
        if map.get(nx as i32, ny as i32) == Some(TILE_EMPTY) {
            let ni = node_index(nx, ny, node_cols);
            if !carved[ni] {
                carved[ni] = true;
            }
        }
    }

    while let Some(&(x, y)) = stack.last() {
        let mut dirs = DFS_DIRS;
        dirs.shuffle(rng);

        let mut found = false;
        for (dx, dy) in dirs {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;

            if nx > 0 && nx < (width - 1) as i32 && ny > 0 && ny < (height - 1) as i32 {
                let nux = nx as usize;
                let nuy = ny as usize;
                let ni = node_index(nux, nuy, node_cols);
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
pub fn generate_maze(width: usize, height: usize, rng: &mut impl Rng) -> (GridMap, Vec<Room>) {
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

    // Place small rooms within the masked area
    let rooms = place_rooms(&mut map, &mask, width, height, rng);

    // Carve maze independently on each island (rooms are pre-carved)
    for island in &islands {
        carve_island(&mut map, island, width, height, rng);
    }

    // Widen some corridors by selectively removing walls
    widen_corridors(&mut map, &mask, &rooms, width, height, rng);

    // Place doors connecting rooms to corridors
    place_doors(&mut map, &rooms, width, height, rng);

    // Place windows and shoji on some interior walls that border a corridor
    place_windows_and_shoji(&mut map, width, height, rng);

    // Place goal on the largest island.
    // Uses reverse BFS discovery order (last-discovered node in BFS), which
    // tends to be far from the BFS start but is not guaranteed to be the
    // spatially farthest cell.
    let largest_island = islands.iter().max_by_key(|i| i.len());
    if let Some(island) = largest_island {
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

    (map, rooms)
}

/// Place doors connecting rooms to corridors.
///
/// Each room gets 1-2 doors. The door type depends on room size:
/// - Large (3x3, 4x3): fusuma
/// - Medium (3x2, 2x3): kitchen door
/// - Small (2x2): toilet door
///
/// One room on the top floor (closest to goal) gets a genkan door.
fn place_doors(map: &mut GridMap, rooms: &[Room], width: usize, height: usize, rng: &mut impl Rng) {
    if rooms.is_empty() {
        return;
    }

    // Find goal position for genkan placement
    let mut goal_pos: Option<(usize, usize)> = None;
    for y in 0..height {
        for x in 0..width {
            if map.get(x as i32, y as i32) == Some(TILE_GOAL) {
                goal_pos = Some((x, y));
            }
        }
    }

    // Find room closest to goal (for genkan door)
    let genkan_room_idx = goal_pos.map(|(gx, gy)| {
        rooms
            .iter()
            .enumerate()
            .min_by_key(|(_, r)| {
                let cx = r.x + r.w / 2;
                let cy = r.y + r.h / 2;
                let dx = cx as i32 - gx as i32;
                let dy = cy as i32 - gy as i32;
                (dx * dx + dy * dy) as usize
            })
            .map(|(i, _)| i)
            .unwrap_or(0)
    });

    for (room_idx, room) in rooms.iter().enumerate() {
        let mut candidates: Vec<(usize, usize)> = Vec::new();

        // Top edge: wall at (x, ry-1), corridor at (x, ry-2)
        if room.y >= 2 {
            for dx in 0..room.w {
                let x = room.x + dx;
                let wy = room.y - 1;
                if map.get(x as i32, wy as i32) == Some(TILE_WALL)
                    && map.get(x as i32, (wy - 1) as i32) == Some(TILE_EMPTY)
                {
                    candidates.push((x, wy));
                }
            }
        }

        // Bottom edge: wall at (x, ry+rh), corridor at (x, ry+rh+1)
        if room.y + room.h + 1 < height {
            for dx in 0..room.w {
                let x = room.x + dx;
                let wy = room.y + room.h;
                if map.get(x as i32, wy as i32) == Some(TILE_WALL)
                    && map.get(x as i32, (wy + 1) as i32) == Some(TILE_EMPTY)
                {
                    candidates.push((x, wy));
                }
            }
        }

        // Left edge: wall at (rx-1, y), corridor at (rx-2, y)
        if room.x >= 2 {
            for dy in 0..room.h {
                let y = room.y + dy;
                let wx = room.x - 1;
                if map.get(wx as i32, y as i32) == Some(TILE_WALL)
                    && map.get((wx - 1) as i32, y as i32) == Some(TILE_EMPTY)
                {
                    candidates.push((wx, y));
                }
            }
        }

        // Right edge: wall at (rx+rw, y), corridor at (rx+rw+1, y)
        if room.x + room.w + 1 < width {
            for dy in 0..room.h {
                let y = room.y + dy;
                let wx = room.x + room.w;
                if map.get(wx as i32, y as i32) == Some(TILE_WALL)
                    && map.get((wx + 1) as i32, y as i32) == Some(TILE_EMPTY)
                {
                    candidates.push((wx, y));
                }
            }
        }

        if candidates.is_empty() {
            // No corridor-facing wall found. Force-open a wall to prevent
            // the room from being completely isolated (unreachable).
            // Find any wall cell on the room border and break it open.
            let mut fallback: Vec<(usize, usize)> = Vec::new();
            let bx0 = room.x.saturating_sub(1);
            let by0 = room.y.saturating_sub(1);
            let bx1 = room.x + room.w;
            let by1 = room.y + room.h;
            for fy in by0..=by1 {
                for fx in bx0..=bx1 {
                    let in_interior = fx >= room.x
                        && fx < room.x + room.w
                        && fy >= room.y
                        && fy < room.y + room.h;
                    if !in_interior && map.get(fx as i32, fy as i32) == Some(TILE_WALL) {
                        fallback.push((fx, fy));
                    }
                }
            }
            if fallback.is_empty() {
                continue;
            }
            fallback.shuffle(rng);
            candidates.push(fallback[0]);
        }

        candidates.shuffle(rng);

        // Determine door type based on room size
        let area = room.w * room.h;
        let is_genkan = genkan_room_idx == Some(room_idx);

        let door_type = if is_genkan {
            TILE_DOOR_GENKAN
        } else if area >= 9 {
            // 3x3 or 4x3
            TILE_DOOR_FUSUMA
        } else if area >= 6 {
            // 3x2, 2x3
            TILE_DOOR_KITCHEN
        } else {
            // 2x2
            TILE_DOOR_TOILET
        };

        // Place 1-2 doors
        let num_doors = if candidates.len() >= 2 && rng.gen_bool(0.3) {
            2
        } else {
            1
        };

        for &(x, y) in candidates.iter().take(num_doors) {
            map.set(x, y, door_type);
        }
    }
}

/// Convert some interior walls into windows or shoji.
/// A wall becomes a candidate if it has at least one empty neighbor
/// (it's visible from a corridor). ~15% of candidates are converted; ~30% become shoji, rest windows.
fn place_windows_and_shoji(map: &mut GridMap, width: usize, height: usize, rng: &mut impl Rng) {
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
        // ~30% of window candidates become shoji, the rest stay as windows
        if rng.gen_bool(0.3) {
            map.set(x, y, TILE_SHOJI);
        } else {
            map.set(x, y, TILE_WINDOW);
        }
    }
}

/// Collect empty cells on a specific island for item placement.
/// Note: `exclude` is scanned linearly but is always small (1-2 entries).
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
    let (mut map, rooms) = generate_maze(width, height, rng);

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

    // Place stairs: each island gets at least one stair (if applicable).
    // change_floor() does a full-map scan for the matching stair type,
    // landing on whichever stair it finds first — potentially on a different
    // island than the departure island. This is intentional: random island
    // landing maximizes the "wandering between islands" exploration effect.
    //
    // Stairs are placed only in corridors (not inside rooms) per 野比家ルール.

    // Helper: check if a cell is inside any room
    let is_in_room = |x: usize, y: usize| -> bool {
        rooms
            .iter()
            .any(|r| x >= r.x && x < r.x + r.w && y >= r.y && y < r.y + r.h)
    };

    // Place stairs down (not on ground floor)
    if floor_index > 0 {
        for island in &islands {
            let mut empties: Vec<(usize, usize)> =
                collect_island_empties(&map, island, &[PLAYER_START])
                    .into_iter()
                    .filter(|&(x, y)| !is_in_room(x, y))
                    .collect();
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
                let mut v = vec![PLAYER_START];
                // Exclude cells already used for stairs down
                for &(nx, ny) in island {
                    if map.get(nx as i32, ny as i32) == Some(TILE_STAIRS_DOWN) {
                        v.push((nx, ny));
                    }
                }
                v
            };
            let mut empties: Vec<(usize, usize)> = collect_island_empties(&map, island, &exclude)
                .into_iter()
                .filter(|&(x, y)| !is_in_room(x, y))
                .collect();
            empties.shuffle(rng);
            if let Some(&(x, y)) = empties.first() {
                map.set(x, y, TILE_STAIRS_UP);
            }
        }
    }

    map
}
