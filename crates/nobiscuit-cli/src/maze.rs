use termray::TileMap;

use crate::nobiscuit_map::NobiscuitMap;
use crate::tiles::{
    TILE_DOOR_FUSUMA, TILE_DOOR_GENKAN, TILE_DOOR_KITCHEN, TILE_DOOR_TOILET, TILE_EMPTY, TILE_GOAL,
    TILE_SHOJI, TILE_STAIRS_DOWN, TILE_STAIRS_UP, TILE_VOID, TILE_WALL, TILE_WINDOW,
    TILE_WINDOW_PASS, TileType,
};
use rand::Rng;
use rand::seq::SliceRandom;
use std::collections::{BTreeMap, VecDeque};

const PLAYER_START: (usize, usize) = (1, 1);
const DIRS4: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

/// Interior bounds of a room (walls are immediately outside the rectangle).
#[derive(Clone, Copy, Debug)]
pub struct Room {
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
}

impl Room {
    fn contains(self, x: usize, y: usize) -> bool {
        x >= self.x && x < self.x + self.w && y >= self.y && y < self.y + self.h
    }
}

/// Split one wall-inclusive rectangle. Children share exactly one wall;
/// no depth cutoff is allowed to leave enormous rooms on large floors.
fn partition(x: usize, y: usize, w: usize, h: usize, rooms: &mut Vec<Room>, rng: &mut impl Rng) {
    if w <= 11 && h <= 11 {
        rooms.push(Room {
            x: x + 1,
            y: y + 1,
            w: w - 2,
            h: h - 2,
        });
        return;
    }
    let vertical = w > 11 && (h <= 11 || w > h || (w == h && rng.gen_bool(0.5)));
    let extent = if vertical { w } else { h };
    // Both children have at least a 3-cell interior. Even split positions
    // preserve the shared wall lattice; choosing near the middle bounds depth.
    let candidates: Vec<_> = (4..=extent - 5)
        .step_by(2)
        .filter(|&s| s >= extent / 3 && s <= extent * 2 / 3)
        .collect();
    let split = *candidates.choose(rng).expect("splittable room");
    if vertical {
        partition(x, y, split + 1, h, rooms, rng);
        partition(x + split, y, w - split, h, rooms, rng);
    } else {
        partition(x, y, w, split + 1, rooms, rng);
        partition(x, y + split, w, h - split, rooms, rng);
    }
}

/// A connection belongs to a pair of regions, not to each wall cell.
struct Connection {
    a: usize,
    b: usize,
    candidates: Vec<(usize, usize)>,
}

fn root(parent: &mut [usize], mut n: usize) -> usize {
    while parent[n] != n {
        parent[n] = parent[parent[n]];
        n = parent[n];
    }
    n
}

fn door_for(rooms: &[Room], a: usize, b: usize) -> TileType {
    if a == 0 || b == 0 {
        TILE_DOOR_GENKAN
    } else {
        let area = [a, b]
            .into_iter()
            .filter_map(|i| rooms.get(i))
            .map(|r| r.w * r.h)
            .min()
            .unwrap_or(usize::MAX);
        match area {
            0..=9 => TILE_DOOR_TOILET,
            10..=21 => TILE_DOOR_KITCHEN,
            _ => TILE_DOOR_FUSUMA,
        }
    }
}

fn connect_regions(
    map: &mut NobiscuitMap,
    rooms: &[Room],
    owner: &[usize],
    corridor_id: Option<usize>,
    rng: &mut impl Rng,
) {
    let width = map.width();
    let height = map.height();
    let mut pairs: BTreeMap<(usize, usize), Vec<(usize, usize)>> = BTreeMap::new();
    for y in 1..height - 1 {
        for x in 1..width - 1 {
            if map.get(x as i32, y as i32) != Some(TILE_WALL) {
                continue;
            }
            for (dx, dy) in [(1isize, 0isize), (0, 1)] {
                let at = |ox: isize, oy: isize| {
                    owner[(y as isize + oy) as usize * width + (x as isize + ox) as usize]
                };
                let a = at(-dx, -dy);
                let b = at(dx, dy);
                if a == usize::MAX || b == usize::MAX || a == b {
                    continue;
                }
                // Leave a jamb on either end; neither room may meet the opening
                // at a corner. This also excludes T-junctions in partition walls.
                if at(-dx + dy, -dy + dx) != a
                    || at(-dx - dy, -dy - dx) != a
                    || at(dx + dy, dy + dx) != b
                    || at(dx - dy, dy - dx) != b
                {
                    continue;
                }
                pairs.entry((a.min(b), a.max(b))).or_default().push((x, y));
            }
        }
    }
    let mut edges: Vec<_> = pairs
        .into_iter()
        .map(|((a, b), candidates)| Connection { a, b, candidates })
        .collect();
    edges.shuffle(rng);
    let mut parent: Vec<_> = (0..rooms.len() + usize::from(corridor_id.is_some())).collect();
    let mut selected = vec![false; edges.len()];
    // Every room facing the long hallway gets one entrance. These edges form a
    // star, so they cannot themselves introduce cycles or merge the room walls.
    for (i, e) in edges.iter().enumerate() {
        if corridor_id.is_some_and(|c| e.a == c || e.b == c) {
            selected[i] = true;
            let a = root(&mut parent, e.a);
            let b = root(&mut parent, e.b);
            parent[a] = b;
        }
    }
    for (i, e) in edges.iter().enumerate() {
        let a = root(&mut parent, e.a);
        let b = root(&mut parent, e.b);
        if a != b {
            selected[i] = true;
            parent[a] = b;
        }
    }
    let remaining: Vec<_> = (0..edges.len()).filter(|&i| !selected[i]).collect();
    for &i in remaining.iter().take((remaining.len() * 15).div_ceil(100)) {
        selected[i] = true;
    }
    // Even a narrow house has direct room-to-room routes in addition to its
    // hallway. Keep at least two such connections where the partition permits.
    let is_room_pair = |e: &Connection| e.a < rooms.len() && e.b < rooms.len();
    let mut room_links = edges
        .iter()
        .enumerate()
        .filter(|(i, e)| selected[*i] && is_room_pair(e))
        .count();
    for (i, e) in edges.iter().enumerate() {
        if room_links >= 2 {
            break;
        }
        if !selected[i] && is_room_pair(e) {
            selected[i] = true;
            room_links += 1;
        }
    }
    let mut room_link_index = 0;
    for (i, e) in edges.iter().enumerate() {
        if !selected[i] {
            continue;
        }
        let &(x, y) = e.candidates.choose(rng).expect("connection has an opening");
        let window = if is_room_pair(e) {
            room_link_index += 1;
            room_link_index == 1 || (room_link_index > 2 && rng.gen_bool(0.25))
        } else {
            false
        };
        map.set(
            x,
            y,
            if window {
                TILE_WINDOW_PASS
            } else {
                door_for(rooms, e.a, e.b)
            },
        );
    }
}

/// One non-overlapping house partition per floor. The main hallway occupies a
/// reserved BSP strip with intact walls; subsequent splits only divide rooms.
pub fn generate_maze(width: usize, height: usize, rng: &mut impl Rng) -> (NobiscuitMap, Vec<Room>) {
    assert!(
        width >= 5 && height >= 5 && width % 2 == 1 && height % 2 == 1,
        "maze dimensions must be odd and at least five"
    );
    let mut map = NobiscuitMap::new(width, height);
    let mut rooms = Vec::new();
    let vertical = width >= height;
    let extent = if vertical { width } else { height };
    let corridor = if extent >= 13 {
        // Gap = wall + three floor cells + wall. Keep both flanks wide enough
        // for rooms and vary the split without creating huge or skinny rooms.
        let positions: Vec<_> = (4..=extent - 9).step_by(2).collect();
        let split = *positions.choose(rng).expect("hallway fits");
        if vertical {
            partition(0, 0, split + 1, height, &mut rooms, rng);
            partition(split + 4, 0, width - split - 4, height, &mut rooms, rng);
            Some(Room {
                x: split + 1,
                y: 1,
                w: 3,
                h: height - 2,
            })
        } else {
            partition(0, 0, width, split + 1, &mut rooms, rng);
            partition(0, split + 4, width, height - split - 4, &mut rooms, rng);
            Some(Room {
                x: 1,
                y: split + 1,
                w: width - 2,
                h: 3,
            })
        }
    } else {
        partition(0, 0, width, height, &mut rooms, rng);
        None
    };
    let mut owner = vec![usize::MAX; width * height];
    for (i, r) in rooms.iter().chain(corridor.iter()).enumerate() {
        for y in r.y..r.y + r.h {
            for x in r.x..r.x + r.w {
                debug_assert_eq!(owner[y * width + x], usize::MAX);
                owner[y * width + x] = i;
                map.set(x, y, TILE_EMPTY);
            }
        }
    }
    connect_regions(&mut map, &rooms, &owner, corridor.map(|_| rooms.len()), rng);
    // Decorative glass/shoji stay solid on remaining partition walls; they
    // never add graph edges or remove jambs beside an opening.
    for y in 1..height - 1 {
        for x in 1..width - 1 {
            if map.get(x as i32, y as i32) != Some(TILE_WALL) {
                continue;
            }
            let near_opening = DIRS4.iter().any(|&(dx, dy)| {
                is_passable(map.get(x as i32 + dx, y as i32 + dy))
                    && map.get(x as i32 + dx, y as i32 + dy) != Some(TILE_EMPTY)
            });
            let room_sides = DIRS4
                .iter()
                .filter(|&&(dx, dy)| {
                    owner[(y as i32 + dy) as usize * width + (x as i32 + dx) as usize] != usize::MAX
                })
                .count();
            if room_sides > 0 && !near_opening && rng.gen_bool(0.12) {
                map.set(
                    x,
                    y,
                    if rng.gen_bool(0.3) {
                        TILE_SHOJI
                    } else {
                        TILE_WINDOW
                    },
                );
            }
        }
    }
    debug_assert!(verify_connectivity(&map, width, height));
    (map, rooms)
}

fn is_passable(tile: Option<TileType>) -> bool {
    matches!(
        tile,
        Some(
            TILE_EMPTY
                | TILE_GOAL
                | TILE_STAIRS_UP
                | TILE_STAIRS_DOWN
                | TILE_DOOR_FUSUMA
                | TILE_DOOR_KITCHEN
                | TILE_DOOR_TOILET
                | TILE_DOOR_GENKAN
                | TILE_WINDOW_PASS
        )
    )
}

/// A single flood seed prevents disconnected stairs from masking an island.
fn reachable_cells(map: &NobiscuitMap, width: usize, height: usize) -> Vec<bool> {
    let mut visited = vec![false; width * height];
    let start = if is_passable(map.get(1, 1)) {
        Some(PLAYER_START)
    } else {
        (0..height)
            .flat_map(|y| (0..width).map(move |x| (x, y)))
            .find(|&(x, y)| is_passable(map.get(x as i32, y as i32)))
    };
    let mut queue = VecDeque::new();
    if let Some((x, y)) = start {
        visited[y * width + x] = true;
        queue.push_back((x, y));
    }
    while let Some((x, y)) = queue.pop_front() {
        for (dx, dy) in DIRS4 {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            if nx < 0 || ny < 0 || nx >= width as i32 || ny >= height as i32 {
                continue;
            }
            let idx = ny as usize * width + nx as usize;
            if !visited[idx] && is_passable(map.get(nx, ny)) {
                visited[idx] = true;
                queue.push_back((nx as usize, ny as usize));
            }
        }
    }
    visited
}

fn verify_connectivity(map: &NobiscuitMap, width: usize, height: usize) -> bool {
    let visited = reachable_cells(map, width, height);
    (0..height).all(|y| {
        (0..width).all(|x| !is_passable(map.get(x as i32, y as i32)) || visited[y * width + x])
    })
}

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
/// The generated BSP path is skipped: the whole interior becomes VOID and the template
/// is stamped in the center. STAIRS_UP is never placed (this is the top floor).
fn generate_goal_floor(width: usize, height: usize) -> NobiscuitMap {
    debug_assert!(
        width % 2 == 1 && height % 2 == 1,
        "maze dimensions must be odd"
    );
    debug_assert!(
        width >= GOAL_TEMPLATE[0].len() && height >= GOAL_TEMPLATE.len(),
        "goal floor must fit the {}x{} template",
        GOAL_TEMPLATE[0].len(),
        GOAL_TEMPLATE.len()
    );
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

    // Seal VOID cells adjacent to walkable cells (same invariant as generate_floor).
    // The template's STAIRS_DOWN sits directly above a VOID row; without this the
    // walkable stair foot would border VOID and render as a black leak in rays.
    seal_void_boundaries(&mut map, width, height);

    map
}

/// Ordinary floors are connected by construction; stair placement cannot erase
/// connections or rely on regeneration and wall-off repairs.
pub fn generate_floor(
    width: usize,
    height: usize,
    floor_index: usize,
    total_floors: usize,
    rng: &mut impl Rng,
) -> NobiscuitMap {
    if floor_index == total_floors - 1 {
        return generate_goal_floor(width, height);
    }
    let (mut map, rooms) = generate_maze(width, height, rng);
    let mut empty = Vec::new();
    let mut hallway = Vec::new();
    for y in 1..height - 1 {
        for x in 1..width - 1 {
            if (x, y) == PLAYER_START || map.get(x as i32, y as i32) != Some(TILE_EMPTY) {
                continue;
            }
            empty.push((x, y));
            if !rooms.iter().any(|r| r.contains(x, y)) {
                hallway.push((x, y));
            }
        }
    }
    let candidates = if hallway.len() >= 2 {
        &mut hallway
    } else {
        &mut empty
    };
    candidates.shuffle(rng);
    if floor_index > 0 {
        if let Some((x, y)) = candidates.pop() {
            map.set(x, y, TILE_STAIRS_DOWN);
        }
    }
    if let Some((x, y)) = candidates.pop() {
        map.set(x, y, TILE_STAIRS_UP);
    }
    debug_assert!(verify_connectivity(&map, width, height));
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
    /// only come from door/window openings) must stay rare.
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
