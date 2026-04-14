pub struct Sprite {
    pub x: f64,
    pub y: f64,
    pub sprite_type: u8,
}

pub struct SpriteRenderResult {
    pub screen_x: i32,
    pub screen_height: i32,
    pub distance: f64,
    pub sprite_type: u8,
}

// TODO: implement sprite projection
pub fn project_sprites(
    _sprites: &[Sprite],
    _camera_x: f64,
    _camera_y: f64,
    _camera_angle: f64,
    _fov: f64,
    _screen_width: usize,
) -> Vec<SpriteRenderResult> {
    Vec::new()
}
