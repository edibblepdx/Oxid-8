use crate::screens::Screen;
use crate::screens::{game::Game, menu::Menu};

use ratatui::{DefaultTerminal, Frame};
use std::io;

#[derive(Default)]
pub struct App {
    menu: Menu,
    game: Game,
    state: AppState,
}

#[derive(Default)]
pub struct AppState {
    pub should_exit: bool,
    pub screen: Screen,
    pub rom_path: Option<std::path::PathBuf>,
}

impl App {
    pub fn run(mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.state.should_exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        match self.state.screen {
            Screen::Debug => (),
            Screen::Menu => self.menu.draw(frame),
            Screen::Game => self.game.draw(frame),
        }
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match self.state.screen {
            Screen::Debug => (),
            Screen::Menu => self.menu.handle_events(&mut self.state)?,
            Screen::Game => self.game.handle_events(&mut self.state)?,
        }
        Ok(())
    }
}
