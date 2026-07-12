use termray::TileMap;

use crate::nobiscuit_map::NobiscuitMap;
use crate::tiles::{
    TILE_DOOR_FUSUMA, TILE_DOOR_GENKAN, TILE_DOOR_KITCHEN, TILE_DOOR_TOILET, TILE_EMPTY, TILE_GOAL,
    TILE_SHOJI, TILE_STAIRS_DOWN, TILE_STAIRS_UP, TILE_VOID, TILE_WALL, TILE_WINDOW, TileType,
};
use rand::Rng;
use rand::seq::SliceRandom;
use std::collections::VecDeque;

/// Player spawn position (grid coordinates). Used for stair exclusion and seed guarantee.
const PLAYER_START: (usize, usize) = (1, 1);

/// DFS neighbor directions (step by 2 on the node grid).
const DFS_DIRS: [(i32, i32); 4] = [(2, 0), (-2, 0), (0, 2), (0, -2)];

/// 4-directional neighbors (step by 1).
const DIRS4: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

/// A placed room (interior coordinates in map space).
pub struct Room {
    pub x: usize, // left column of interior
    pub y: usize, // top row of interior
    pub w: usize, // interior width
    pub h: usize, // interior height
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

/// A corridor strip carved through the gap of a BSP split.
///
/// `horizontal` = the corridor runs horizontally (a horizontal split line).
/// `pos` = the split line coordinate (always even = a wall column/row). The
/// corridor occupies the three cells `pos+1..=pos+3` (walls kept at `pos` and
/// `pos+4`, so flanking rooms never merge into the corridor). `from..=to` is the
/// span along the corridor's running axis (interior coordinates).
struct CorridorStrip {
    horizontal: bool,
    pos: usize,
    from: usize,
    to: usize,
}

/// Pick a random even value in `[lo, hi]` (both endpoints must be even).
fn pick_even(lo: usize, hi: usize, rng: &mut impl Rng) -> usize {
    if hi <= lo {
        return lo;
    }
    let steps = (hi - lo) / 2;
    lo + 2 * rng.gen_range(0..=steps)
}

/// Recursively partition a wall-inclusive rectangle `(x, y, w, h)` into leaf
/// regions (future rooms) and, occasionally, corridor strips.
///
/// Invariants that keep the "odd = floor, even = wall" grid intact:
/// - Split offsets are always even, so shared walls land on even coordinates and
///   each leaf's inset-by-1 interior starts on an odd coordinate.
/// - A corridor split leaves a 3-cell gap (`pos+1..=pos+3`) with walls at `pos`
///   and `pos+4`, so rooms on either side stay separated by a single wall.
#[allow(clippy::too_many_arguments)]
fn bsp_split(
    regions_out: &mut Vec<(usize, usize, usize, usize)>,
    corridors_out: &mut Vec<CorridorStrip>,
    x: usize,
    y: usize,
    w: usize,
    h: usize,
    depth: usize,
    corridor_budget: &mut usize,
    rng: &mut impl Rng,
) {
    const MIN_LEAF: usize = 5; // wall-inclusive 5 → interior 3
    const MAX_LEAF: usize = 11; // wall-inclusive 11 → interior 9

    if (w <= MAX_LEAF && h <= MAX_LEAF) || depth >= 5 {
        regions_out.push((x, y, w, h));
        return;
    }

    let vertical = w >= h;

    if vertical {
        // Optional corridor split (needs room for a 3-cell gap + two MIN_LEAF children).
        if *corridor_budget > 0 && depth <= 1 && h >= 8 && w >= 13 && rng.gen_bool(0.6) {
            let lo = MIN_LEAF - 1; // 4 (even)
            let hi = w - MIN_LEAF - 4; // even (w odd)
            if hi >= lo {
                let split = pick_even(lo, hi, rng);
                corridors_out.push(CorridorStrip {
                    horizontal: false,
                    pos: x + split,
                    from: y + 1,
                    to: y + h - 2,
                });
                *corridor_budget -= 1;
                bsp_split(
                    regions_out,
                    corridors_out,
                    x,
                    y,
                    split + 1,
                    h,
                    depth + 1,
                    corridor_budget,
                    rng,
                );
                bsp_split(
                    regions_out,
                    corridors_out,
                    x + split + 4,
                    y,
                    w - split - 4,
                    h,
                    depth + 1,
                    corridor_budget,
                    rng,
                );
                return;
            }
        }
        // Normal split: children share the wall at `x + split`.
        let split = pick_even(MIN_LEAF - 1, w - MIN_LEAF, rng);
        bsp_split(
            regions_out,
            corridors_out,
            x,
            y,
            split + 1,
            h,
            depth + 1,
            corridor_budget,
            rng,
        );
        bsp_split(
            regions_out,
            corridors_out,
            x + split,
            y,
            w - split,
            h,
            depth + 1,
            corridor_budget,
            rng,
        );
    } else {
        if *corridor_budget > 0 && depth <= 1 && w >= 8 && h >= 13 && rng.gen_bool(0.6) {
            let lo = MIN_LEAF - 1;
            let hi = h - MIN_LEAF - 4;
            if hi >= lo {
                let split = pick_even(lo, hi, rng);
                corridors_out.push(CorridorStrip {
                    horizontal: true,
                    pos: y + split,
                    from: x + 1,
                    to: x + w - 2,
                });
                *corridor_budget -= 1;
                bsp_split(
                    regions_out,
                    corridors_out,
                    x,
                    y,
                    w,
                    split + 1,
                    depth + 1,
                    corridor_budget,
                    rng,
                );
                bsp_split(
                    regions_out,
                    corridors_out,
                    x,
                    y + split + 4,
                    w,
                    h - split - 4,
                    depth + 1,
                    corridor_budget,
                    rng,
                );
                return;
            }
        }
        let split = pick_even(MIN_LEAF - 1, h - MIN_LEAF, rng);
        bsp_split(
            regions_out,
            corridors_out,
            x,
            y,
            w,
            split + 1,
            depth + 1,
            corridor_budget,
            rng,
        );
        bsp_split(
            regions_out,
            corridors_out,
            x,
            y + split,
            w,
            h - split,
            depth + 1,
            corridor_budget,
            rng,
        );
    }
}

/// BSP layout for a single island: partition its bounding rectangle into rooms
/// and corridor strips, carving TILE_EMPTY only where the mask is playable.
///
/// Returns the placed rooms and the flat list of carved corridor cells.
fn bsp_layout(
    map: &mut NobiscuitMap,
    island_nodes: &[(usize, usize)],
    mask: &[bool],
    width: usize,
    height: usize,
    rng: &mut impl Rng,
) -> (Vec<Room>, Vec<(usize, usize)>) {
    if island_nodes.is_empty() {
        return (Vec::new(), Vec::new());
    }

    // Bounding rectangle of the island's DFS nodes (all odd coordinates).
    let mut min_x = usize::MAX;
    let mut min_y = usize::MAX;
    let mut max_x = 0;
    let mut max_y = 0;
    for &(nx, ny) in island_nodes {
        min_x = min_x.min(nx);
        min_y = min_y.min(ny);
        max_x = max_x.max(nx);
        max_y = max_y.max(ny);
    }
    // Wall-inclusive rectangle: one even-coordinate wall ring around the nodes.
    let rx = min_x - 1;
    let ry = min_y - 1;
    let rw = max_x - min_x + 3;
    let rh = max_y - min_y + 3;

    let mut regions: Vec<(usize, usize, usize, usize)> = Vec::new();
    let mut corridors: Vec<CorridorStrip> = Vec::new();
    let mut budget = 2usize;
    bsp_split(
        &mut regions,
        &mut corridors,
        rx,
        ry,
        rw,
        rh,
        0,
        &mut budget,
        rng,
    );

    // Carve rooms: each leaf's interior is inset by one wall on every side.
    let mut rooms: Vec<Room> = Vec::new();
    for (x, y, w, h) in regions {
        if w < 3 || h < 3 {
            continue;
        }
        let ix0 = x + 1;
        let iy0 = y + 1;
        let iw = w - 2;
        let ih = h - 2;
        let area = iw * ih;
        let mut cnt = 0;
        for yy in iy0..iy0 + ih {
            for xx in ix0..ix0 + iw {
                if mask[yy * width + xx] {
                    cnt += 1;
                }
            }
        }
        // Skip regions that are mostly VOID (keeps the island's amoeba silhouette
        // at region granularity). Regions that survive are carved as solid
        // rectangles so rooms read as rooms, not perforated swiss cheese.
        if cnt * 2 < area {
            continue;
        }
        for yy in iy0..iy0 + ih {
            for xx in ix0..ix0 + iw {
                map.set(xx, yy, TILE_EMPTY);
            }
        }
        rooms.push(Room {
            x: ix0,
            y: iy0,
            w: iw,
            h: ih,
        });
    }

    // Carve corridors (width 3) in the gaps between children.
    let mut corridor_cells: Vec<(usize, usize)> = Vec::new();
    for c in &corridors {
        if c.horizontal {
            for row in c.pos + 1..=c.pos + 3 {
                if row < 1 || row >= height - 1 {
                    continue;
                }
                for col in c.from..=c.to {
                    if col < 1 || col >= width - 1 {
                        continue;
                    }
                    map.set(col, row, TILE_EMPTY);
                    corridor_cells.push((col, row));
                }
            }
        } else {
            for col in c.pos + 1..=c.pos + 3 {
                if col < 1 || col >= width - 1 {
                    continue;
                }
                for row in c.from..=c.to {
                    if row < 1 || row >= height - 1 {
                        continue;
                    }
                    map.set(col, row, TILE_EMPTY);
                    corridor_cells.push((col, row));
                }
            }
        }
    }

    (rooms, corridor_cells)
}

/// Connect rooms (and corridor components) into a single reachable network.
///
/// Builds an adjacency graph where two carved regions are neighbors when their
/// interiors face each other across a single wall. A spanning tree of that graph
/// is opened as fusuma doors, plus ~15% extra edges to form loops (maze feel).
/// Corridor cells are grouped into connected components, each treated as a graph
/// node so every corridor gets at least one door to an adjacent room.
///
/// `place_doors` runs afterwards to add typed doors (kitchen/toilet/genkan) and
/// windows on the remaining walls; connectivity is already guaranteed here so it
/// does not depend on `place_doors`.
fn connect_rooms(
    map: &mut NobiscuitMap,
    rooms: &[Room],
    corridor_cells: &[(usize, usize)],
    width: usize,
    height: usize,
    rng: &mut impl Rng,
) {
    if rooms.is_empty() {
        return;
    }
    let n = width * height;

    // Room-id grid (index into `rooms`, or usize::MAX).
    let mut roomid = vec![usize::MAX; n];
    for (i, r) in rooms.iter().enumerate() {
        for yy in r.y..r.y + r.h {
            for xx in r.x..r.x + r.w {
                if map.get(xx as i32, yy as i32) == Some(TILE_EMPTY) {
                    roomid[yy * width + xx] = i;
                }
            }
        }
    }

    // Corridor connected components.
    let mut is_corr = vec![false; n];
    for &(cx, cy) in corridor_cells {
        is_corr[cy * width + cx] = true;
    }
    let mut comp = vec![usize::MAX; n];
    let mut ncomp = 0;
    for &(cx, cy) in corridor_cells {
        let start = cy * width + cx;
        if comp[start] != usize::MAX {
            continue;
        }
        let cid = ncomp;
        ncomp += 1;
        let mut q: VecDeque<(usize, usize)> = VecDeque::new();
        comp[start] = cid;
        q.push_back((cx, cy));
        while let Some((x, y)) = q.pop_front() {
            for (dx, dy) in DIRS4 {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx < 0 || ny < 0 || nx >= width as i32 || ny >= height as i32 {
                    continue;
                }
                let idx = ny as usize * width + nx as usize;
                if is_corr[idx] && comp[idx] == usize::MAX {
                    comp[idx] = cid;
                    q.push_back((nx as usize, ny as usize));
                }
            }
        }
    }

    let node_count = rooms.len() + ncomp;
    let node_of = |x: usize, y: usize| -> Option<usize> {
        let idx = y * width + x;
        if roomid[idx] != usize::MAX {
            Some(roomid[idx])
        } else if comp[idx] != usize::MAX {
            Some(rooms.len() + comp[idx])
        } else {
            None
        }
    };

    // Adjacency edges: (node_a, node_b, wall_cell).
    let mut edges: Vec<(usize, usize, usize, usize)> = Vec::new();
    for (i, r) in rooms.iter().enumerate() {
        let mut pairs: Vec<((usize, usize), (usize, usize))> = Vec::new();
        if r.y >= 2 {
            for xx in r.x..r.x + r.w {
                pairs.push(((xx, r.y - 1), (xx, r.y - 2)));
            }
        }
        if r.y + r.h + 1 < height {
            for xx in r.x..r.x + r.w {
                pairs.push(((xx, r.y + r.h), (xx, r.y + r.h + 1)));
            }
        }
        if r.x >= 2 {
            for yy in r.y..r.y + r.h {
                pairs.push(((r.x - 1, yy), (r.x - 2, yy)));
            }
        }
        if r.x + r.w + 1 < width {
            for yy in r.y..r.y + r.h {
                pairs.push(((r.x + r.w, yy), (r.x + r.w + 1, yy)));
            }
        }
        for ((wx, wy), (tx, ty)) in pairs {
            if map.get(wx as i32, wy as i32) != Some(TILE_WALL) {
                continue;
            }
            if let Some(nb) = node_of(tx, ty) {
                if nb != i {
                    edges.push((i, nb, wx, wy));
                }
            }
        }
    }
    edges.shuffle(rng);

    // Union-find spanning tree; open a door per tree edge.
    fn find(parent: &mut [usize], mut x: usize) -> usize {
        while parent[x] != x {
            parent[x] = parent[parent[x]];
            x = parent[x];
        }
        x
    }
    let mut parent: Vec<usize> = (0..node_count).collect();
    let mut leftover: Vec<(usize, usize)> = Vec::new();
    for &(a, b, wx, wy) in &edges {
        let ra = find(&mut parent, a);
        let rb = find(&mut parent, b);
        if ra != rb {
            parent[ra] = rb;
            if map.get(wx as i32, wy as i32) == Some(TILE_WALL) {
                map.set(wx, wy, TILE_DOOR_FUSUMA);
            }
        } else {
            leftover.push((wx, wy));
        }
    }

    // Extra loops: open ~15% of the remaining candidate walls.
    let extra = leftover.len() * 15 / 100;
    leftover.shuffle(rng);
    for &(wx, wy) in leftover.iter().take(extra) {
        if map.get(wx as i32, wy as i32) == Some(TILE_WALL) {
            map.set(wx, wy, TILE_DOOR_FUSUMA);
        }
    }
}

/// Guarantee the player spawn `(1,1)` is walkable and connected to the layout.
///
/// `generate_mask` only seeds `(1,1)`; it does not guarantee it is carved EMPTY
/// after BSP. If `(1,1)` is not EMPTY, force a small walkable pocket at the corner
/// and carve a width-1 straight path to the nearest existing EMPTY cell (which is
/// already part of the connected room/corridor network).
fn ensure_spawn_walkable(map: &mut NobiscuitMap, width: usize, height: usize) {
    if map.get(1, 1) == Some(TILE_EMPTY) {
        return;
    }
    for &(x, y) in &[(1usize, 1usize), (2, 1), (1, 2)] {
        if x < width - 1 && y < height - 1 && map.get(x as i32, y as i32) != Some(TILE_GOAL) {
            map.set(x, y, TILE_EMPTY);
        }
    }

    // Nearest existing EMPTY cell (by Manhattan distance from the spawn corner).
    let mut target: Option<(usize, usize)> = None;
    'outer: for radius in 2..(width + height) {
        for yy in 1..height - 1 {
            for xx in 1..width - 1 {
                let d = (xx as i32 - 1).unsigned_abs() as usize
                    + (yy as i32 - 1).unsigned_abs() as usize;
                if d != radius {
                    continue;
                }
                if map.get(xx as i32, yy as i32) == Some(TILE_EMPTY)
                    && !((xx, yy) == (1, 1) || (xx, yy) == (2, 1) || (xx, yy) == (1, 2))
                {
                    target = Some((xx, yy));
                    break 'outer;
                }
            }
        }
    }

    if let Some((tx, ty)) = target {
        let mut x = 1usize;
        while x != tx {
            x = if tx > x { x + 1 } else { x - 1 };
            if (1..width - 1).contains(&x) && map.get(x as i32, 1) != Some(TILE_GOAL) {
                map.set(x, 1, TILE_EMPTY);
            }
        }
        let mut y = 1usize;
        while y != ty {
            y = if ty > y { y + 1 } else { y - 1 };
            if (1..height - 1).contains(&y) && map.get(tx as i32, y as i32) != Some(TILE_GOAL) {
                map.set(tx, y, TILE_EMPTY);
            }
        }
    }
}

/// Generate a maze with irregular shape using mask-based BSP layout.
///
/// Both `width` and `height` must be odd numbers (the algorithm steps by 2).
pub fn generate_maze(width: usize, height: usize, rng: &mut impl Rng) -> (NobiscuitMap, Vec<Room>) {
    assert!(
        width % 2 == 1 && height % 2 == 1,
        "maze dimensions must be odd"
    );
    let mut map = NobiscuitMap::new(width, height);

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

    // BSP layout on each island: rooms + corridor strips.
    let mut rooms: Vec<Room> = Vec::new();
    let mut corridor_cells: Vec<(usize, usize)> = Vec::new();
    for island in &islands {
        let (island_rooms, island_corridors) =
            bsp_layout(&mut map, island, &mask, width, height, rng);
        rooms.extend(island_rooms);
        corridor_cells.extend(island_corridors);
    }

    // Open doors so every room/corridor is reachable (spanning tree + loops).
    connect_rooms(&mut map, &rooms, &corridor_cells, width, height, rng);

    // Guarantee the spawn corner is walkable and wired into the network.
    ensure_spawn_walkable(&mut map, width, height);

    // Place typed doors (kitchen/toilet/genkan) and extra room openings.
    place_doors(&mut map, &rooms, width, height, rng);

    // Place windows and shoji on some interior walls that border a corridor
    place_windows_and_shoji(&mut map, width, height, rng);

    // Seal VOID boundaries: convert VOID cells adjacent to walkable cells into WALL.
    // Without this, rays from corridors/rooms can reach VOID directly, causing
    // black lines/areas to appear inside playable spaces.
    seal_void_boundaries(&mut map, width, height);

    // Place goal on the largest island.
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
/// - Large (area >= 9): fusuma
/// - Medium (area >= 6): kitchen door
/// - Small (2x2): toilet door
///
/// One room (closest to goal) gets a genkan door.
fn place_doors(
    map: &mut NobiscuitMap,
    rooms: &[Room],
    width: usize,
    height: usize,
    rng: &mut impl Rng,
) {
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
            // No corridor-facing wall found. Force-open a wall whose far side is
            // EMPTY to prevent isolation (never open toward VOID).
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
                        // Only break walls whose outward neighbor is EMPTY.
                        let outward_empty = DIRS4.iter().any(|&(dx, dy)| {
                            let ox = fx as i32 + dx;
                            let oy = fy as i32 + dy;
                            let inside = ox >= room.x as i32
                                && ox < (room.x + room.w) as i32
                                && oy >= room.y as i32
                                && oy < (room.y + room.h) as i32;
                            !inside && map.get(ox, oy) == Some(TILE_EMPTY)
                        });
                        if outward_empty {
                            fallback.push((fx, fy));
                        }
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
            TILE_DOOR_FUSUMA
        } else if area >= 6 {
            TILE_DOOR_KITCHEN
        } else {
            TILE_DOOR_TOILET
        };

        // Place 1-2 doors (do not overwrite doors already opened by connect_rooms)
        let num_doors = if candidates.len() >= 2 && rng.gen_bool(0.3) {
            2
        } else {
            1
        };

        for &(x, y) in candidates.iter().take(num_doors) {
            if map.get(x as i32, y as i32) == Some(TILE_WALL) {
                map.set(x, y, door_type);
            }
        }
    }
}

/// Convert some interior walls into windows or shoji.
/// A wall becomes a candidate if it has at least one empty neighbor
/// (it's visible from a corridor). ~15% of candidates are converted; ~30% become shoji, rest windows.
fn place_windows_and_shoji(
    map: &mut NobiscuitMap,
    width: usize,
    height: usize,
    rng: &mut impl Rng,
) {
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

/// Seal VOID boundaries by converting VOID cells adjacent to non-solid cells into WALL.
///
/// After maze generation, some VOID cells (mask boundary) may directly border
/// walkable cells (EMPTY, GOAL, stairs). This causes rays from playable areas
/// to reach VOID, rendering black lines/areas inside rooms and corridors.
/// Converting these boundary VOID cells to WALL prevents the visual artifact.
/// Uses 8-directional neighbor checks (including diagonals) because DDA rays
/// can step diagonally through cell corners, reaching a diagonally adjacent VOID.
fn seal_void_boundaries(map: &mut NobiscuitMap, width: usize, height: usize) {
    const DIRS8: [(i32, i32); 8] = [
        (1, 0),
        (-1, 0),
        (0, 1),
        (0, -1),
        (1, 1),
        (1, -1),
        (-1, 1),
        (-1, -1),
    ];

    // Collect cells to convert (avoid mutating while iterating)
    let mut to_wall: Vec<(usize, usize)> = Vec::new();

    // Outer ring is always WALL (never VOID), so skip edges
    for y in 1..height - 1 {
        for x in 1..width - 1 {
            if map.get(x as i32, y as i32) != Some(TILE_VOID) {
                continue;
            }
            let has_walkable_neighbor = DIRS8.iter().any(|&(dx, dy)| {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                !map.is_solid(nx, ny)
            });
            if has_walkable_neighbor {
                to_wall.push((x, y));
            }
        }
    }

    for (x, y) in to_wall {
        map.set(x, y, TILE_WALL);
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

/// Return true if `tile` can be walked through (auto-open doors count as passable,
/// matching `game.rs`, which opens adjacent doors in play).
fn is_passable(tile: Option<TileType>) -> bool {
    matches!(
        tile,
        Some(TILE_EMPTY)
            | Some(TILE_GOAL)
            | Some(TILE_STAIRS_UP)
            | Some(TILE_STAIRS_DOWN)
            | Some(TILE_DOOR_FUSUMA)
            | Some(TILE_DOOR_KITCHEN)
            | Some(TILE_DOOR_TOILET)
            | Some(TILE_DOOR_GENKAN)
    )
}

/// Flood fill from the spawn corner and every stair, marking reachable cells.
fn reachable_cells(map: &NobiscuitMap, width: usize, height: usize) -> Vec<bool> {
    let mut visited = vec![false; width * height];
    let mut q: VecDeque<(usize, usize)> = VecDeque::new();

    // Seeds: spawn (1,1) if passable, plus every stair cell (upper-floor landings).
    let seed = |x: usize, y: usize, q: &mut VecDeque<(usize, usize)>, vis: &mut Vec<bool>| {
        let idx = y * width + x;
        if !vis[idx] {
            vis[idx] = true;
            q.push_back((x, y));
        }
    };
    if is_passable(map.get(1, 1)) {
        seed(1, 1, &mut q, &mut visited);
    }
    for y in 0..height {
        for x in 0..width {
            if matches!(
                map.get(x as i32, y as i32),
                Some(TILE_STAIRS_UP) | Some(TILE_STAIRS_DOWN)
            ) {
                seed(x, y, &mut q, &mut visited);
            }
        }
    }

    while let Some((x, y)) = q.pop_front() {
        for (dx, dy) in DIRS4 {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            if nx < 0 || ny < 0 || nx >= width as i32 || ny >= height as i32 {
                continue;
            }
            let idx = ny as usize * width + nx as usize;
            if !visited[idx] && is_passable(map.get(nx, ny)) {
                visited[idx] = true;
                q.push_back((nx as usize, ny as usize));
            }
        }
    }
    visited
}

/// Verify that every walkable tile (EMPTY / GOAL / stairs) is reachable from the
/// spawn corner or a stair. Doors are treated as passable (auto-opened in play).
fn verify_connectivity(map: &NobiscuitMap, width: usize, height: usize) -> bool {
    let visited = reachable_cells(map, width, height);
    for y in 0..height {
        for x in 0..width {
            let walkable = matches!(
                map.get(x as i32, y as i32),
                Some(TILE_EMPTY) | Some(TILE_GOAL) | Some(TILE_STAIRS_UP) | Some(TILE_STAIRS_DOWN)
            );
            if walkable && !visited[y * width + x] {
                return false;
            }
        }
    }
    true
}

/// Fallback repair: wall off any unreachable EMPTY cells so no items spawn where
/// the player cannot reach them.
fn wall_off_unreachable(map: &mut NobiscuitMap, width: usize, height: usize) {
    let visited = reachable_cells(map, width, height);
    for y in 1..height - 1 {
        for x in 1..width - 1 {
            if map.get(x as i32, y as i32) == Some(TILE_EMPTY) && !visited[y * width + x] {
                map.set(x, y, TILE_WALL);
            }
        }
    }
}

/// Fixed top-floor template: descend-stairs → short vertical corridor → fusuma →
/// Nobita's room with the GOAL in the center. Sized 11×11 so it fits the minimum
/// 15×13 maze; stamped centered, the rest of the floor is VOID.
const GOAL_TEMPLATE: [&str; 11] = [
    "###########",
    "#VVVVVVVVV#",
    "#V#######V#",
    "#V#.....#V#",
    "#V#..G..#V#",
    "#V#.....#V#",
    "#V###F###V#",
    "#V###.###V#",
    "#V###D###V#",
    "#VVVVVVVVV#",
    "###########",
];

/// Build the fixed goal floor for the top level.
///
/// The mask/BSP path is skipped: the whole interior becomes VOID and the template
/// is stamped in the center. STAIRS_UP is never placed (this is the top floor).
fn generate_goal_floor(width: usize, height: usize) -> NobiscuitMap {
    let mut map = NobiscuitMap::new(width, height);

    // Interior VOID (outer ring stays WALL from NobiscuitMap::new).
    for y in 1..height - 1 {
        for x in 1..width - 1 {
            map.set(x, y, TILE_VOID);
        }
    }

    let th = GOAL_TEMPLATE.len().min(height);
    let tw = GOAL_TEMPLATE[0].len().min(width);
    let start_x = (width - tw) / 2;
    let start_y = (height - th) / 2;

    for (cy, row) in GOAL_TEMPLATE.iter().enumerate().take(th) {
        for (cx, ch) in row.bytes().enumerate().take(tw) {
            let tile = match ch {
                b'#' => TILE_WALL,
                b'V' => TILE_VOID,
                b'.' => TILE_EMPTY,
                b'G' => TILE_GOAL,
                b'F' => TILE_DOOR_FUSUMA,
                b'D' => TILE_STAIRS_DOWN,
                _ => TILE_WALL,
            };
            map.set(start_x + cx, start_y + cy, tile);
        }
    }

    map
}

/// Build a single (non-top) floor layout: maze + goal removal + stair placement.
fn build_floor_layout(
    width: usize,
    height: usize,
    floor_index: usize,
    total_floors: usize,
    rng: &mut impl Rng,
) -> NobiscuitMap {
    let (mut map, rooms) = generate_maze(width, height, rng);

    // Remove goal from non-top floors (goal only on top floor)
    for y in 0..height {
        for x in 0..width {
            if map.get(x as i32, y as i32) == Some(TILE_GOAL) {
                map.set(x, y, TILE_EMPTY);
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

    // Stairs are placed in corridors (not inside rooms) per 野比家ルール. If an
    // island has no corridor cell (e.g. the minimum 15×13 map splits into a
    // single room), fall back to placing the stair inside a room so the floor
    // stays traversable.
    let is_in_room = |x: usize, y: usize| -> bool {
        rooms
            .iter()
            .any(|r| x >= r.x && x < r.x + r.w && y >= r.y && y < r.y + r.h)
    };

    // Place stairs down (not on ground floor)
    if floor_index > 0 {
        for island in &islands {
            let all = collect_island_empties(&map, island, &[PLAYER_START]);
            let mut empties: Vec<(usize, usize)> = all
                .iter()
                .copied()
                .filter(|&(x, y)| !is_in_room(x, y))
                .collect();
            if empties.is_empty() {
                empties = all;
            }
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
                for &(nx, ny) in island {
                    if map.get(nx as i32, ny as i32) == Some(TILE_STAIRS_DOWN) {
                        v.push((nx, ny));
                    }
                }
                v
            };
            let all = collect_island_empties(&map, island, &exclude);
            let mut empties: Vec<(usize, usize)> = all
                .iter()
                .copied()
                .filter(|&(x, y)| !is_in_room(x, y))
                .collect();
            if empties.is_empty() {
                empties = all;
            }
            empties.shuffle(rng);
            if let Some(&(x, y)) = empties.first() {
                map.set(x, y, TILE_STAIRS_UP);
            }
        }
    }

    map
}

/// Generate a maze for a specific floor with stairs placed automatically.
///
/// The top floor uses a fixed goal template. Other floors are generated with the
/// BSP layout and verified with a flood fill: if any walkable cell is unreachable
/// the floor is regenerated (up to 10 attempts), and as a last resort the
/// unreachable cells are walled off so items never spawn out of reach.
pub fn generate_floor(
    width: usize,
    height: usize,
    floor_index: usize,
    total_floors: usize,
    rng: &mut impl Rng,
) -> NobiscuitMap {
    let is_top_floor = floor_index == total_floors - 1;
    if is_top_floor {
        return generate_goal_floor(width, height);
    }

    for _ in 0..10 {
        let map = build_floor_layout(width, height, floor_index, total_floors, rng);
        if verify_connectivity(&map, width, height) {
            return map;
        }
    }

    let mut map = build_floor_layout(width, height, floor_index, total_floors, rng);
    wall_off_unreachable(&mut map, width, height);
    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    /// Every walkable cell must be reachable from the spawn/stairs across a wide
    /// range of seeds, sizes, and floors.
    #[test]
    fn all_walkable_cells_reachable_over_many_seeds() {
        for seed in 0..300u64 {
            let mut rng = StdRng::seed_from_u64(seed);
            for &(w, h) in &[(15, 13), (31, 25), (61, 45), (121, 91)] {
                for floor in 0..3 {
                    let map = generate_floor(w, h, floor, 3, &mut rng);
                    assert!(
                        verify_connectivity(&map, w, h),
                        "seed={seed} {w}x{h} floor={floor}"
                    );
                }
            }
        }
    }

    /// The BSP layout is rooms + wide corridors, so 1-cell-wide passages (which
    /// only come from spawn-correction paths and door openings) must stay rare.
    #[test]
    fn no_one_wide_dfs_corridors() {
        for seed in 0..40u64 {
            let mut rng = StdRng::seed_from_u64(seed);
            for &(w, h) in &[(31, 25), (61, 45)] {
                // Ground floor uses the BSP layout (not the template).
                let map = generate_floor(w, h, 0, 3, &mut rng);
                let mut empties = 0usize;
                let mut one_wide = 0usize;
                for y in 1..h - 1 {
                    for x in 1..w - 1 {
                        if map.get(x as i32, y as i32) != Some(TILE_EMPTY) {
                            continue;
                        }
                        empties += 1;
                        let left = map.get(x as i32 - 1, y as i32) == Some(TILE_EMPTY);
                        let right = map.get(x as i32 + 1, y as i32) == Some(TILE_EMPTY);
                        let up = map.get(x as i32, y as i32 - 1) == Some(TILE_EMPTY);
                        let down = map.get(x as i32, y as i32 + 1) == Some(TILE_EMPTY);
                        let count = [left, right, up, down].iter().filter(|&&b| b).count();
                        // A 1-wide corridor cell: a straight run with empties only
                        // on one axis and <= 2 empty orthogonal neighbors.
                        let straight =
                            (left && right && !up && !down) || (up && down && !left && !right);
                        if count <= 2 && straight {
                            one_wide += 1;
                        }
                    }
                }
                if empties > 0 {
                    let ratio = one_wide as f64 / empties as f64;
                    assert!(
                        ratio < 0.05,
                        "seed={seed} {w}x{h}: 1-wide ratio {ratio} ({one_wide}/{empties})"
                    );
                }
            }
        }
    }

    /// The ground-floor spawn `(1,1)` must always be walkable.
    #[test]
    fn spawn_is_walkable() {
        for seed in 0..300u64 {
            let mut rng = StdRng::seed_from_u64(seed);
            for &(w, h) in &[(15, 13), (31, 25), (61, 45)] {
                let map = generate_floor(w, h, 0, 3, &mut rng);
                assert!(
                    !map.is_solid(1, 1),
                    "seed={seed} {w}x{h}: spawn (1,1) is solid"
                );
            }
        }
    }

    /// The top floor uses the fixed template: exactly one GOAL and no STAIRS_UP.
    #[test]
    fn top_floor_has_goal_and_template() {
        for seed in 0..50u64 {
            let mut rng = StdRng::seed_from_u64(seed);
            for &(w, h) in &[(15, 13), (31, 25), (61, 45)] {
                let map = generate_floor(w, h, 2, 3, &mut rng);
                let mut goals = 0;
                let mut ups = 0;
                for y in 0..h {
                    for x in 0..w {
                        match map.get(x as i32, y as i32) {
                            Some(TILE_GOAL) => goals += 1,
                            Some(TILE_STAIRS_UP) => ups += 1,
                            _ => {}
                        }
                    }
                }
                assert_eq!(goals, 1, "seed={seed} {w}x{h}: goal count");
                assert_eq!(ups, 0, "seed={seed} {w}x{h}: stairs-up count");
                assert!(verify_connectivity(&map, w, h));
            }
        }
    }
}
