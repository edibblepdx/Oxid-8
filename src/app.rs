use crate::screens::Screen;
use crate::screens::menu::Menu;

use ratatui::{DefaultTerminal, Frame};
use std::io;

#[derive(Default)]
pub struct App {
    menu: Menu,
    state: AppState,
}

#[derive(Default)]
pub struct AppState {
    pub should_exit: bool,
    pub screen: Screen,
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
            Screen::Game => (),
        }
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match self.state.screen {
            Screen::Debug => (),
            Screen::Menu => self.menu.handle_events(&mut self.state)?,
            Screen::Game => (),
        }
        Ok(())
    }
}
