use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

pub enum GameInput {
    MoveForward,
    MoveBackward,
    TurnLeft,
    TurnRight,
    ToggleMinimap,
    Retry,   // Y key — only meaningful in Result phase
    Decline, // N key — only meaningful in Result phase
    Quit,
}

pub fn poll_input(timeout: Duration) -> Option<GameInput> {
    if event::poll(timeout).ok()? {
        if let Event::Key(key) = event::read().ok()? {
            return match key {
                KeyEvent {
                    code: KeyCode::Char('w') | KeyCode::Up,
                    ..
                } => Some(GameInput::MoveForward),
                KeyEvent {
                    code: KeyCode::Char('s') | KeyCode::Down,
                    ..
                } => Some(GameInput::MoveBackward),
                KeyEvent {
                    code: KeyCode::Char('a') | KeyCode::Left,
                    ..
                } => Some(GameInput::TurnLeft),
                KeyEvent {
                    code: KeyCode::Char('d') | KeyCode::Right,
                    ..
                } => Some(GameInput::TurnRight),
                KeyEvent {
                    code: KeyCode::Char('m'),
                    ..
                } => Some(GameInput::ToggleMinimap),
                KeyEvent {
                    code: KeyCode::Char('q'),
                    ..
                } => Some(GameInput::Quit),
                KeyEvent {
                    code: KeyCode::Char('y'),
                    ..
                } => Some(GameInput::Retry),
                KeyEvent {
                    code: KeyCode::Char('n'),
                    ..
                } => Some(GameInput::Decline),
                KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers,
                    ..
                } if modifiers.contains(KeyModifiers::CONTROL) => Some(GameInput::Quit),
                KeyEvent {
                    code: KeyCode::Esc, ..
                } => Some(GameInput::Quit),
                _ => None,
            };
        }
    }
    None
}
