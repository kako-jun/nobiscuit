pub struct GameState {
    pub show_minimap: bool,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            show_minimap: true,
        }
    }
}
