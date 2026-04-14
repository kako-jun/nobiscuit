use crate::map::TileMap;
use crate::math::{fisheye_correct, Vec2f};
use crate::ray::{cast_ray, RayHit};

pub struct Camera {
    pub x: f64,
    pub y: f64,
    pub angle: f64,
    pub fov: f64,
}

impl Camera {
    pub fn new(x: f64, y: f64, angle: f64, fov: f64) -> Self {
        Self { x, y, angle, fov }
    }

    /// Cast rays for all screen columns, returning fisheye-corrected hits
    pub fn cast_all_rays(
        &self,
        map: &dyn TileMap,
        num_rays: usize,
        max_depth: f64,
    ) -> Vec<Option<RayHit>> {
        let half_fov = self.fov / 2.0;
        let origin = Vec2f::new(self.x, self.y);

        (0..num_rays)
            .map(|i| {
                let ray_angle = self.angle - half_fov
                    + self.fov * (i as f64) / (num_rays as f64);

                cast_ray(map, origin, ray_angle, max_depth).map(|mut hit| {
                    hit.distance = fisheye_correct(hit.distance, ray_angle, self.angle);
                    hit
                })
            })
            .collect()
    }
}
