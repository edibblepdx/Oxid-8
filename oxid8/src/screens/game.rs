use crate::app::AppState;
use oxid8_core::{Oxid8, SCREEN_HEIGHT, SCREEN_WIDTH};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    Frame,
    buffer::Buffer,
    layout::Rect,
    symbols::Marker,
    widgets::{Widget, canvas::Canvas},
};
use std::io;

const TICK_RATE: u64 = 1 / 700;

#[derive(Default)]
pub struct Game {
    emu: Oxid8,
    state: GameState,
}

#[derive(Default)]
struct GameState {
    redraw: bool,
}

impl Game {
    pub fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    pub fn handle_events(&mut self, app_state: &mut AppState) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event, app_state)
            }
            _ => (),
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent, app_state: &mut AppState) {
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Esc => app_state.should_exit = true,
            _ => (),
        }
    }

    //fn canvas(&self) -> impl Widget + '_ {}
}

impl Widget for &mut Game {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let width = SCREEN_WIDTH as f64;
        let height = SCREEN_HEIGHT as f64;

        Widget::render(
            Canvas::default()
                .x_bounds([-width / 2.0, width / 2.0])
                .y_bounds([-height / 2.0, height / 2.0])
                .marker(Marker::HalfBlock)
                .paint(|ctx| {
                    let screen_ref = self.emu.screen_ref();
                    for y in 0..SCREEN_HEIGHT {
                        for x in 0..SCREEN_WIDTH {
                            if screen_ref[x + y * SCREEN_WIDTH] {
                                ctx.print(x as f64, y as f64, "█");
                            }
                        }
                    }
                }),
            area,
            buf,
        );
    }
}
