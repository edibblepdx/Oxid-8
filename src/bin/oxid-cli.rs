use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use oxid8::core::{Oxid8, SCREEN_HEIGHT, SCREEN_WIDTH};
use ratatui::{
    style::Color,
    symbols::Marker,
    widgets::canvas::{Canvas, Painter, Shape},
};
use std::{
    env,
    io::{self, Write, stdout},
    time::{Duration, Instant},
};

const TICK_RATE: u64 = 1 / 700;

#[derive(Default)]
struct Emu {
    core: Oxid8,
    state: EmuState,
}

#[derive(Default)]
struct EmuState {
    should_exit: bool,
}

fn main() -> io::Result<()> {
    let mut emu = Emu::default();
    if let Err(err) = emu.core.load_rom("../oxid-8/c8games/IBMLOGO") {
        eprintln!("{err}");
    }
    emu.core.load_font();

    let mut terminal = ratatui::init();
    terminal.clear()?;

    let size = terminal.size()?;
    let width = size.width as f64;
    let height = size.width as f64;

    while !emu.state.should_exit {
        let time = Instant::now();

        // TODO: poll events (key press mainly)
        if event::poll(Duration::from_millis(1))? {
            if let Err(err) = handle_events(&mut emu.state) {
                eprintln!("{err}");
            }
        }

        if let Err(err) = emu.core.run_cycle() {
            eprintln!("{err}");
        }

        // TODO: check draw flag
        let _ = terminal.draw(|frame| {
            let area = frame.area();

            frame.render_widget(
                Canvas::default()
                    .x_bounds([-width / 2.0, width / 2.0])
                    .y_bounds([-height / 2.0, height / 2.0])
                    .marker(Marker::HalfBlock)
                    .paint(|ctx| {
                        ctx.draw(&emu);
                    }),
                area,
            )
        });

        // TODO: check timers

        while time.elapsed().as_secs() < TICK_RATE {} // spin
    }

    ratatui::restore();
    Ok(())
}

fn handle_events(emu_state: &mut EmuState) -> io::Result<()> {
    match event::read()? {
        Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
            handle_key_event(key_event, emu_state)
        }
        _ => (),
    };
    Ok(())
}

fn handle_key_event(key_event: KeyEvent, emu_state: &mut EmuState) {
    match key_event.code {
        KeyCode::Esc => emu_state.should_exit = true,
        _ => (),
    }
}

impl Shape for Emu {
    fn draw(&self, painter: &mut Painter) {
        let screen_ref = self.core.screen_ref();
        for y in 0..SCREEN_HEIGHT {
            for x in 0..SCREEN_WIDTH {
                if screen_ref[x + y * SCREEN_WIDTH] {
                    painter.paint(x, y, Color::White)
                }
            }
        }
    }
}
