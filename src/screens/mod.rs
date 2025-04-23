use crate::app::AppState;
use ratatui::Frame;
use std::io;

pub mod debug;
pub mod game;
pub mod menu;
pub mod widgets;

pub enum Screen {
    Debug,
    Menu,
    Game,
}

impl Default for Screen {
    fn default() -> Screen {
        Screen::Menu
    }
}

pub trait ScreenTrait {
    fn draw(&mut self, frame: &mut Frame);
    fn handle_events(&mut self, app_state: &mut AppState) -> io::Result<()>;
}
