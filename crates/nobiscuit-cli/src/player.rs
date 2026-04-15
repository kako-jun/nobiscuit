use nobiscuit_engine::camera::Camera;
use nobiscuit_engine::map::TileMap;

use crate::input::GameInput;

/// Cardinal direction (90-degree increments)
#[derive(Debug, Clone, Copy, PartialEq)]
enum Dir {
    East,  // angle = 0
    South, // angle = π/2
    West,  // angle = π
    North, // angle = 3π/2
}

impl Dir {
    fn angle(self) -> f64 {
        match self {
            Dir::East => 0.0,
            Dir::South => std::f64::consts::FRAC_PI_2,
            Dir::West => std::f64::consts::PI,
            Dir::North => std::f64::consts::FRAC_PI_2 * 3.0,
        }
    }

    fn dx(self) -> i32 {
        match self {
            Dir::East => 1,
            Dir::West => -1,
            _ => 0,
        }
    }

    fn dy(self) -> i32 {
        match self {
            Dir::South => 1,
            Dir::North => -1,
            _ => 0,
        }
    }

    fn turn_left(self) -> Dir {
        match self {
            Dir::East => Dir::North,
            Dir::North => Dir::West,
            Dir::West => Dir::South,
            Dir::South => Dir::East,
        }
    }

    fn turn_right(self) -> Dir {
        match self {
            Dir::East => Dir::South,
            Dir::South => Dir::West,
            Dir::West => Dir::North,
            Dir::North => Dir::East,
        }
    }
}

/// Interpolation state for smooth grid movement / turning
#[derive(Debug, Clone)]
enum Motion {
    Idle,
    Moving {
        from_x: f64,
        from_y: f64,
        to_x: f64,
        to_y: f64,
        progress: f64,
    },
    Turning {
        from_angle: f64,
        to_angle: f64,
        progress: f64,
    },
}

/// Animation speed (fraction of move completed per second).
/// 1.0 / MOVE_SPEED = seconds per tile.
const MOVE_SPEED: f64 = 4.0;
const TURN_SPEED: f64 = 6.0;

pub struct Player {
    pub camera: Camera,
    /// Grid position (tile coordinates)
    grid_x: i32,
    grid_y: i32,
    facing: Dir,
    motion: Motion,
}

impl Player {
    pub fn new(x: f64, y: f64, _angle: f64) -> Self {
        let gx = x as i32;
        let gy = y as i32;
        let facing = Dir::East;
        Self {
            camera: Camera::new(
                gx as f64 + 0.5,
                gy as f64 + 0.5,
                facing.angle(),
                std::f64::consts::FRAC_PI_3,
            ),
            grid_x: gx,
            grid_y: gy,
            facing,
            motion: Motion::Idle,
        }
    }

    pub fn update(&mut self, input: Option<&GameInput>, map: &dyn TileMap, dt: f64) {
        // Advance any in-progress animation
        match &mut self.motion {
            Motion::Moving {
                from_x,
                from_y,
                to_x,
                to_y,
                progress,
            } => {
                *progress += MOVE_SPEED * dt;
                if *progress >= 1.0 {
                    self.camera.x = *to_x;
                    self.camera.y = *to_y;
                    self.motion = Motion::Idle;
                } else {
                    let t = ease_in_out(*progress);
                    self.camera.x = *from_x + (*to_x - *from_x) * t;
                    self.camera.y = *from_y + (*to_y - *from_y) * t;
                }
            }
            Motion::Turning {
                from_angle,
                to_angle,
                progress,
            } => {
                *progress += TURN_SPEED * dt;
                if *progress >= 1.0 {
                    self.camera.angle = *to_angle;
                    self.motion = Motion::Idle;
                } else {
                    let t = ease_in_out(*progress);
                    self.camera.angle = lerp_angle(*from_angle, *to_angle, t);
                }
            }
            Motion::Idle => {}
        }

        // Accept new input only when idle
        if !matches!(self.motion, Motion::Idle) {
            return;
        }

        let Some(input) = input else { return };

        match input {
            GameInput::TurnLeft => {
                let old_angle = self.facing.angle();
                self.facing = self.facing.turn_left();
                self.motion = Motion::Turning {
                    from_angle: old_angle,
                    to_angle: self.facing.angle(),
                    progress: 0.0,
                };
            }
            GameInput::TurnRight => {
                let old_angle = self.facing.angle();
                self.facing = self.facing.turn_right();
                self.motion = Motion::Turning {
                    from_angle: old_angle,
                    to_angle: self.facing.angle(),
                    progress: 0.0,
                };
            }
            GameInput::MoveForward => {
                self.try_grid_move(self.facing, map);
            }
            GameInput::MoveBackward => {
                // Move in reverse direction (no turning)
                let reverse = self.facing.turn_left().turn_left();
                self.try_grid_move(reverse, map);
            }
            _ => {}
        }
    }

    fn try_grid_move(&mut self, dir: Dir, map: &dyn TileMap) {
        let nx = self.grid_x + dir.dx();
        let ny = self.grid_y + dir.dy();

        if !map.is_solid(nx, ny) {
            let from_x = self.camera.x;
            let from_y = self.camera.y;
            self.grid_x = nx;
            self.grid_y = ny;
            self.motion = Motion::Moving {
                from_x,
                from_y,
                to_x: nx as f64 + 0.5,
                to_y: ny as f64 + 0.5,
                progress: 0.0,
            };
        }
    }
}

/// Smooth ease-in-out curve (cubic)
fn ease_in_out(t: f64) -> f64 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}

/// Linearly interpolate between two angles, taking the shortest arc.
fn lerp_angle(from: f64, to: f64, t: f64) -> f64 {
    let mut diff = to - from;
    // Normalize to [-π, π]
    while diff > std::f64::consts::PI {
        diff -= std::f64::consts::TAU;
    }
    while diff < -std::f64::consts::PI {
        diff += std::f64::consts::TAU;
    }
    from + diff * t
}
