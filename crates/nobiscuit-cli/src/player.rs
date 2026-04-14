use nobiscuit_engine::camera::Camera;
use nobiscuit_engine::map::TileMap;
use nobiscuit_engine::math::normalize_angle;

use crate::input::GameInput;

pub struct Player {
    pub camera: Camera,
    pub move_speed: f64,
    pub turn_speed: f64,
}

impl Player {
    pub fn new(x: f64, y: f64, angle: f64) -> Self {
        Self {
            camera: Camera::new(x, y, angle, std::f64::consts::FRAC_PI_3), // 60 deg FOV
            move_speed: 3.0,
            turn_speed: 2.5,
        }
    }

    pub fn update(&mut self, input: Option<&GameInput>, map: &dyn TileMap, dt: f64) {
        let Some(input) = input else { return };

        match input {
            GameInput::TurnLeft => {
                self.camera.angle -= self.turn_speed * dt;
            }
            GameInput::TurnRight => {
                self.camera.angle += self.turn_speed * dt;
            }
            GameInput::MoveForward => {
                let dx = self.camera.angle.cos() * self.move_speed * dt;
                let dy = self.camera.angle.sin() * self.move_speed * dt;
                self.try_move(dx, dy, map);
            }
            GameInput::MoveBackward => {
                let dx = -self.camera.angle.cos() * self.move_speed * dt;
                let dy = -self.camera.angle.sin() * self.move_speed * dt;
                self.try_move(dx, dy, map);
            }
            _ => {}
        }

        self.camera.angle = normalize_angle(self.camera.angle);
    }

    fn try_move(&mut self, dx: f64, dy: f64, map: &dyn TileMap) {
        let margin = 0.2;

        // Try X movement independently
        let new_x = self.camera.x + dx;
        if !map.is_solid(
            (new_x + margin * dx.signum()) as i32,
            self.camera.y as i32,
        ) {
            self.camera.x = new_x;
        }

        // Try Y movement independently
        let new_y = self.camera.y + dy;
        if !map.is_solid(
            self.camera.x as i32,
            (new_y + margin * dy.signum()) as i32,
        ) {
            self.camera.y = new_y;
        }
    }
}
