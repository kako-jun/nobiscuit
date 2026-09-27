//! Black-box geometry oracles deliberately do not use the generator's flood fill.
#![allow(dead_code)]
#[path = "../src/game.rs"]
mod game;
#[path = "../src/maze.rs"]
mod maze;
#[path = "../src/nobiscuit_map.rs"]
mod nobiscuit_map;
#[path = "../src/textures.rs"]
mod textures;
#[path = "../src/tiles.rs"]
mod tiles;
#[path = "../src/windows.rs"]
mod windows;
use nobiscuit_map::NobiscuitMap;
use rand::{SeedableRng, rngs::StdRng};
use std::collections::{HashSet, VecDeque};
use termray::{Camera, Color, Framebuffer, TileMap};
use tiles::*;
const SIZES: &[(usize, usize)] = &[
    (15, 13),
    (25, 19),
    (31, 25),
    (51, 41),
    (61, 45),
    (81, 61),
    (121, 91),
    (19, 25),
    (25, 31),
];
fn door(t: u8) -> bool {
    (TILE_DOOR_FUSUMA..=TILE_DOOR_GENKAN).contains(&t)
}
fn opening(t: u8) -> bool {
    door(t) || t == TILE_WINDOW_PASS
}
fn cells(m: &NobiscuitMap) -> Vec<u8> {
    (0..m.height())
        .flat_map(|y| (0..m.width()).map(move |x| m.get(x as i32, y as i32).unwrap()))
        .collect()
}
fn labels(m: &NobiscuitMap, open: bool) -> Vec<usize> {
    let w = m.width();
    let h = m.height();
    let mut labels = vec![usize::MAX; w * h];
    let mut id = 0;
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            let pass = |x: usize, y: usize| {
                let t = m.get(x as i32, y as i32).unwrap();
                t == TILE_EMPTY
                    || t == TILE_GOAL
                    || t == TILE_STAIRS_UP
                    || t == TILE_STAIRS_DOWN
                    || (open && opening(t))
            };
            if labels[i] != usize::MAX || !pass(x, y) {
                continue;
            }
            labels[i] = id;
            let mut q = VecDeque::from([(x, y)]);
            while let Some((x, y)) = q.pop_front() {
                for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                        continue;
                    }
                    let (nx, ny) = (nx as usize, ny as usize);
                    if labels[ny * w + nx] == usize::MAX && pass(nx, ny) {
                        labels[ny * w + nx] = id;
                        q.push_back((nx, ny));
                    }
                }
            }
            id += 1;
        }
    }
    labels
}
fn samples(mut f: impl FnMut(&NobiscuitMap, &[maze::Room], u64, usize, usize)) {
    for seed in 0..40 {
        for &(w, h) in SIZES {
            let (m, r) = maze::generate_maze(w, h, &mut StdRng::seed_from_u64(seed));
            f(&m, &r, seed, w, h);
        }
    }
}
#[test]
fn rooms_remain_separate_after_closing_openings() {
    samples(|m, rooms, seed, w, h| {
        let l = labels(m, false);
        let mut occupied = HashSet::new();
        let mut regions = HashSet::new();
        for r in rooms {
            assert!((3..=9).contains(&r.w) && (3..=9).contains(&r.h));
            let id = l[r.y * w + r.x];
            assert!(regions.insert(id), "merged rooms seed={seed} {w}x{h}");
            for y in r.y..r.y + r.h {
                for x in r.x..r.x + r.w {
                    assert!(occupied.insert((x, y)), "overlapping rooms");
                    assert_eq!(l[y * w + x], id);
                }
            }
        }
    });
}
fn links(m: &NobiscuitMap) -> Vec<(usize, usize, u8, usize, usize)> {
    let l = labels(m, false);
    let w = m.width();
    let mut out = vec![];
    for y in 1..m.height() - 1 {
        for x in 1..w - 1 {
            let t = m.get(x as i32, y as i32).unwrap();
            if !opening(t) {
                continue;
            }
            let mut pairs = vec![];
            for (a, b, j, k) in [
                (y * w + x - 1, y * w + x + 1, (x, y - 1), (x, y + 1)),
                ((y - 1) * w + x, (y + 1) * w + x, (x - 1, y), (x + 1, y)),
            ] {
                if l[a] != usize::MAX && l[b] != usize::MAX && l[a] != l[b] {
                    assert!(
                        m.is_solid(j.0 as i32, j.1 as i32) && m.is_solid(k.0 as i32, k.1 as i32),
                        "opening missing jamb"
                    );
                    pairs.push((l[a].min(l[b]), l[a].max(l[b])));
                }
            }
            assert_eq!(pairs.len(), 1);
            out.push((pairs[0].0, pairs[0].1, t, x, y));
        }
    }
    out
}
#[test]
fn shared_wall_has_at_most_one_opening() {
    samples(|m, _, seed, w, h| {
        let mut seen = HashSet::new();
        for (a, b, _, _, _) in links(m) {
            assert!(seen.insert((a, b)), "duplicate opening seed={seed} {w}x{h}");
        }
    });
}
#[test]
fn no_isolated_pillars_after_doors_open() {
    samples(|m, _, seed, w, h| {
        for y in 1..h - 1 {
            for x in 1..w - 1 {
                let t = m.get(x as i32, y as i32).unwrap();
                if !m.is_solid(x as i32, y as i32) || door(t) {
                    continue;
                }
                assert!(
                    [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|&(dx, dy)| {
                        let t = m.get(x as i32 + dx, y as i32 + dy).unwrap();
                        m.is_solid(x as i32 + dx, y as i32 + dy) && !door(t)
                    }),
                    "pillar {x},{y} seed={seed} {w}x{h}"
                );
            }
        }
    });
}
#[test]
fn room_to_room_routes_include_window() {
    room_routes(TILE_WINDOW_PASS);
}
#[test]
fn room_to_room_routes_include_door() {
    room_routes(TILE_DOOR_FUSUMA);
}
fn room_routes(kind: u8) {
    samples(|m, r, seed, w, h| {
        let l = labels(m, false);
        let ids: HashSet<_> = r.iter().map(|r| l[r.y * w + r.x]).collect();
        assert!(
            links(m).iter().any(|&(a, b, t, _, _)| ids.contains(&a)
                && ids.contains(&b)
                && if kind == TILE_WINDOW_PASS {
                    t == kind
                } else {
                    door(t)
                }),
            "missing room route seed={seed} {w}x{h}"
        );
    });
}
#[test]
fn long_hallway_has_two_distinct_doors_each_side() {
    samples(|m, r, seed, w, h| {
        if w.min(h) < 19 || w.max(h) < 25 {
            return;
        }
        let l = labels(m, false);
        let ids: HashSet<_> = r.iter().map(|r| l[r.y * w + r.x]).collect();
        let hall: Vec<_> = l
            .iter()
            .enumerate()
            .filter(|(_, id)| **id != usize::MAX && !ids.contains(id))
            .map(|(i, _)| (i % w, i / w))
            .collect();
        assert!(!hall.is_empty());
        let x0 = hall.iter().map(|p| p.0).min().unwrap();
        let x1 = hall.iter().map(|p| p.0).max().unwrap();
        let y0 = hall.iter().map(|p| p.1).min().unwrap();
        let y1 = hall.iter().map(|p| p.1).max().unwrap();
        let vertical = x1 - x0 == 2;
        assert!(if vertical {
            y1 - y0 + 1 >= 9
        } else {
            y1 - y0 == 2 && x1 - x0 + 1 >= 9
        });
        let mut sides = [HashSet::new(), HashSet::new()];
        for (a, b, t, x, y) in links(m) {
            if !door(t) {
                continue;
            }
            let side = if vertical {
                if x + 1 == x0 {
                    Some(0)
                } else if x == x1 + 1 {
                    Some(1)
                } else {
                    None
                }
            } else if y + 1 == y0 {
                Some(0)
            } else if y == y1 + 1 {
                Some(1)
            } else {
                None
            };
            if let Some(s) = side {
                if ids.contains(&a) {
                    sides[s].insert(a);
                }
                if ids.contains(&b) {
                    sides[s].insert(b);
                }
            }
        }
        assert!(
            sides.iter().all(|s| s.len() >= 2),
            "hall doors seed={seed} {w}x{h}"
        );
    });
}
#[test]
fn world_items_and_stairs_reachable() {
    for seed in 0..10 {
        for &(w, h) in SIZES {
            let world = game::World::new(3, w, h, &mut StdRng::seed_from_u64(seed));
            for (i, f) in world.floors.iter().enumerate() {
                let l = labels(&f.map, true);
                let all = cells(&f.map);
                let start = if i == 0 {
                    w + 1
                } else {
                    all.iter()
                        .position(|&t| t == TILE_STAIRS_DOWN)
                        .expect("down stair missing")
                };
                assert_ne!(l[start], usize::MAX);
                for s in &f.sprites {
                    assert_eq!(
                        l[s.y as usize * w + s.x as usize],
                        l[start],
                        "item unreachable seed={seed} {w}x{h} floor={i}"
                    );
                }
                for (t, count) in [
                    (TILE_STAIRS_UP, usize::from(i < 2)),
                    (TILE_STAIRS_DOWN, usize::from(i > 0)),
                    (TILE_GOAL, usize::from(i == 2)),
                ] {
                    assert_eq!(all.iter().filter(|&&v| v == t).count(), count);
                }
                assert!(
                    l.iter()
                        .filter(|&&id| id != usize::MAX)
                        .all(|&id| id == l[start])
                );
            }
        }
    }
}
#[test]
fn outer_ring_is_sealed() {
    samples(|m, _, _, w, h| {
        for x in 0..w {
            assert_eq!(m.get(x as i32, 0), Some(TILE_WALL));
            assert_eq!(m.get(x as i32, h as i32 - 1), Some(TILE_WALL));
        }
        for y in 0..h {
            assert_eq!(m.get(0, y as i32), Some(TILE_WALL));
            assert_eq!(m.get(w as i32 - 1, y as i32), Some(TILE_WALL));
        }
    });
}
#[test]
fn same_seed_reproduces_world() {
    for seed in [0, 42, u64::MAX] {
        let a = game::World::new(3, 31, 25, &mut StdRng::seed_from_u64(seed));
        let b = game::World::new(3, 31, 25, &mut StdRng::seed_from_u64(seed));
        for (a, b) in a.floors.iter().zip(&b.floors) {
            assert_eq!(cells(&a.map), cells(&b.map));
            assert_eq!(
                a.sprites
                    .iter()
                    .map(|s| (s.x, s.y, s.sprite_type))
                    .collect::<Vec<_>>(),
                b.sprites
                    .iter()
                    .map(|s| (s.x, s.y, s.sprite_type))
                    .collect::<Vec<_>>()
            );
        }
    }
}
#[test]
fn seeds_change_room_shapes() {
    let shapes: HashSet<_> = (0..20)
        .map(|seed| {
            let (_, r) = maze::generate_maze(31, 25, &mut StdRng::seed_from_u64(seed));
            r.iter().map(|r| (r.x, r.y, r.w, r.h)).collect::<Vec<_>>()
        })
        .collect();
    assert!(shapes.len() > 1);
}
#[test]
fn invalid_dimensions_rejected() {
    for (w, h) in [
        (0, 13),
        (1, 13),
        (4, 13),
        (6, 13),
        (14, 13),
        (15, 12),
        (15, 0),
    ] {
        assert!(
            std::panic::catch_unwind(|| maze::generate_maze(w, h, &mut StdRng::seed_from_u64(1)))
                .is_err()
        );
    }
    for (w, h) in [
        (5, 5),
        (7, 5),
        (11, 13),
        (13, 13),
        (23, 19),
        (25, 17),
        (25, 19),
        (27, 21),
    ] {
        maze::generate_maze(w, h, &mut StdRng::seed_from_u64(1));
    }
}
fn scene(windows_at: &[usize], blocked: bool) -> NobiscuitMap {
    let mut m = NobiscuitMap::new(13, 9);
    for y in 1..8 {
        for x in 1..12 {
            m.set(x, y, TILE_EMPTY);
        }
    }
    for &x in windows_at {
        for y in 1..8 {
            m.set(x, y, TILE_WALL);
        }
        m.set(x, 4, TILE_WINDOW_PASS);
    }
    if blocked {
        for y in 1..8 {
            m.set(2, y, TILE_WALL);
        }
    }
    m
}
const BG: Color = Color::rgb(3, 7, 11);
fn frame(m: &NobiscuitMap, c: &Camera) -> Vec<Color> {
    let mut fb = Framebuffer::new(160, 100);
    fb.clear(BG);
    let rays = c.cast_all_rays(m, 160, 20.0);
    windows::render_windows(&mut fb, m, c, &rays, &[], 20.0);
    pixels(&fb)
}
fn pixels(f: &Framebuffer) -> Vec<Color> {
    (0..f.height())
        .flat_map(|y| (0..f.width()).map(move |x| f.get_pixel(x, y)))
        .collect()
}
#[test]
fn window_frame_visible_both_sides_and_axes() {
    let original = scene(&[4], false);
    for transpose in [false, true] {
        let mut m = NobiscuitMap::new(
            if transpose { 9 } else { 13 },
            if transpose { 13 } else { 9 },
        );
        for y in 0..9 {
            for x in 0..13 {
                let (a, b) = if transpose { (y, x) } else { (x, y) };
                m.set(a, b, original.get(x as i32, y as i32).unwrap());
            }
        }
        for rear in [false, true] {
            let x = if rear { 7.5 } else { 1.5 };
            let (cx, cy, angle) = if transpose {
                (
                    4.5,
                    x,
                    if rear {
                        -std::f64::consts::FRAC_PI_2
                    } else {
                        std::f64::consts::FRAC_PI_2
                    },
                )
            } else {
                (x, 4.5, if rear { std::f64::consts::PI } else { 0.0 })
            };
            let p = frame(&m, &Camera::new(cx, cy, angle, 1.0));
            assert!(p.iter().any(|&v| v != BG));
            assert_eq!(p[50 * 160 + 80], BG, "central opening obscured");
        }
    }
}
#[test]
fn foreground_wall_hides_window() {
    let m = scene(&[4], true);
    let p = frame(&m, &Camera::new(1.5, 4.5, 0.0, 1.0));
    assert!(p.iter().all(|&p| p == BG));
}
#[test]
fn two_window_layers_preserve_nearest_frame() {
    for rear in [false, true] {
        let c = Camera::new(
            if rear { 10.5 } else { 1.5 },
            4.5,
            if rear { std::f64::consts::PI } else { 0.0 },
            1.0,
        );
        let near = if rear { 8 } else { 4 };
        let far = if rear { 4 } else { 8 };
        let a = frame(&scene(&[near], false), &c);
        let b = frame(&scene(&[far], false), &c);
        let both = frame(&scene(&[4, 8], false), &c);
        assert!(
            a.iter().zip(&b).any(|(&a, &b)| a == BG && b != BG),
            "fixture must see far frame through near opening"
        );
        for i in 0..both.len() {
            assert_eq!(
                both[i],
                if a[i] != BG { a[i] } else { b[i] },
                "depth ordering pixel {i}"
            );
        }
    }
}
#[test]
fn sprite_pixels_occlude_frame_but_transparent_holes_do_not() {
    let m = scene(&[4], false);
    let c = Camera::new(1.5, 4.5, 0.0, 1.0);
    let rays = c.cast_all_rays(&m, 160, 20.0);
    let only_frame = frame(&m, &c);
    let mut checked_opaque = 0;
    let mut checked_holes = 0;
    // Sweep the actual art across the frame; use the engine raster output as oracle.
    for sx in [55, 65, 80, 95, 105] {
        let s = termray::SpriteRenderResult {
            screen_x: sx,
            screen_height: 160.0,
            distance: 1.0,
            sprite_type: game::SPRITE_BISCUIT,
            screen_y_feet: 75.0,
        };
        let mut fb = Framebuffer::new(160, 100);
        fb.clear(BG);
        termray::render_sprites(
            &mut fb,
            std::slice::from_ref(&s),
            &rays,
            &textures::NobiscuitTextures,
            20.0,
        );
        let before = pixels(&fb);
        windows::render_windows(&mut fb, &m, &c, &rays, &[s], 20.0);
        let after = pixels(&fb);
        for i in 0..after.len() {
            if before[i] != BG {
                assert_eq!(after[i], before[i]);
                if only_frame[i] != BG {
                    checked_opaque += 1;
                }
            } else if only_frame[i] != BG {
                assert_eq!(after[i], only_frame[i]);
                checked_holes += 1;
            }
        }
    }
    assert!(checked_opaque > 0 && checked_holes > 0);
}
#[test]
fn open_window_is_walkable_and_rays_hit_back_wall() {
    let m = scene(&[4], false);
    assert!(!m.is_solid(4, 4));
    for t in [TILE_WINDOW, TILE_SHOJI] {
        let mut m = scene(&[], false);
        m.set(4, 4, t);
        assert!(m.is_solid(4, 4));
    }
    let c = Camera::new(1.5, 4.5, 0.0, 1.0);
    let rays = c.cast_all_rays(&m, 160, 20.0);
    assert!(rays[80].as_ref().unwrap().distance > 9.0);
}
#[test]
fn spin_tier_boundaries() {
    for (spins, w, h, floors) in [
        (0, 15, 13, 1),
        (1, 15, 13, 1),
        (2, 15, 13, 1),
        (3, 25, 19, 2),
        (4, 25, 19, 2),
        (5, 31, 25, 3),
        (6, 31, 25, 3),
        (7, 31, 25, 3),
        (8, 51, 41, 5),
        (9, 51, 41, 5),
        (10, 51, 41, 5),
        (11, 81, 61, 8),
        (12, 81, 61, 8),
        (15, 81, 61, 8),
        (16, 121, 91, 12),
        (17, 121, 91, 12),
        (u32::MAX, 121, 91, 12),
    ] {
        let p = game::maze_params_from_spins(spins);
        assert_eq!((p.width, p.height, p.num_floors), (w, h, floors));
    }
}
#[test]
fn frames_cover_background_sprites_only_outside_opening() {
    let m = scene(&[4], false);
    let c = Camera::new(1.5, 4.5, 0.0, 1.0);
    let rays = c.cast_all_rays(&m, 160, 20.0);
    let only_frame = frame(&m, &c);
    let sprite = termray::SpriteRenderResult {
        screen_x: 80,
        screen_height: 160.0,
        distance: 4.0,
        sprite_type: game::SPRITE_BISCUIT,
        screen_y_feet: 75.0,
    };
    let mut fb = Framebuffer::new(160, 100);
    fb.clear(BG);
    termray::render_sprites(
        &mut fb,
        std::slice::from_ref(&sprite),
        &rays,
        &textures::NobiscuitTextures,
        20.0,
    );
    let before = pixels(&fb);
    windows::render_windows(&mut fb, &m, &c, &rays, &[sprite], 20.0);
    let after = pixels(&fb);
    let mut covered = 0;
    let mut visible = 0;
    for i in 0..after.len() {
        if only_frame[i] != BG {
            assert_eq!(after[i], only_frame[i]);
            if before[i] != BG {
                covered += 1;
            }
        } else {
            assert_eq!(after[i], before[i]);
            if before[i] != BG {
                visible += 1;
            }
        }
    }
    assert!(covered > 0 && visible > 0);
}
#[test]
fn doors_close_at_distance_three_while_windows_remain_open() {
    let mut m = scene(&[4], false);
    m.set(4, 3, TILE_DOOR_FUSUMA);
    let mut world = game::World {
        floors: vec![game::Floor {
            map: m,
            sprites: vec![],
        }],
        current_floor: 0,
    };
    let mut state = game::GameState::new();
    state.update(&mut world, 3.5, 3.5, 0.0);
    assert_eq!(world.current_map().get(4, 3), Some(TILE_EMPTY));
    for (x, expected) in [
        (2.5, TILE_EMPTY),
        (1.5, TILE_DOOR_FUSUMA),
        (0.5, TILE_DOOR_FUSUMA),
    ] {
        state.update(&mut world, x, 3.5, 0.0);
        assert_eq!(world.current_map().get(4, 3), Some(expected));
        assert_eq!(world.current_map().get(4, 4), Some(TILE_WINDOW_PASS));
    }
}
#[test]
fn stair_transitions_land_on_reachable_matching_stairs() {
    let mut world = game::World::new(3, 25, 19, &mut StdRng::seed_from_u64(42));
    for (target, dir, tile) in [
        (1, game::StairDirection::Up, TILE_STAIRS_DOWN),
        (2, game::StairDirection::Up, TILE_STAIRS_DOWN),
        (1, game::StairDirection::Down, TILE_STAIRS_UP),
        (0, game::StairDirection::Down, TILE_STAIRS_UP),
    ] {
        let (x, y) = world.change_floor(target, dir);
        assert_eq!(world.current_map().get(x as i32, y as i32), Some(tile));
        let l = labels(world.current_map(), true);
        let id = l[y as usize * 25 + x as usize];
        assert_ne!(id, usize::MAX);
        assert!(l.iter().filter(|&&v| v != usize::MAX).all(|&v| v == id));
    }
}

#[test]
fn one_floor_start_reaches_goal_without_crossing_void() {
    let world = game::World::new(1, 15, 13, &mut StdRng::seed_from_u64(42));
    let (x, y) = world.start_position();
    let map = world.current_map();
    assert!(!map.is_solid(x as i32, y as i32));
    let regions = labels(map, true);
    let start = regions[y as usize * 15 + x as usize];
    assert_ne!(start, usize::MAX);
    let goal = cells(map)
        .iter()
        .position(|&tile| tile == TILE_GOAL)
        .unwrap();
    assert_eq!(regions[goal], start);
    for sprite in world.current_sprites() {
        assert_eq!(regions[sprite.y as usize * 15 + sprite.x as usize], start);
    }
}
