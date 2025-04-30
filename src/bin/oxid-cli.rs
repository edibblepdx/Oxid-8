use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use oxid8::core::{Oxid8, SCREEN_HEIGHT, SCREEN_WIDTH};
use ratatui::{
    style::Color,
    symbols::Marker,
    widgets::canvas::{Canvas, Painter, Shape},
};
use std::{
    env, io, process,
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

pub struct Config {
    pub rom_path: String,
}

impl Config {
    pub fn build(args: &[String]) -> Result<Config, &'static str> {
        if args.len() >= 2 {
            Ok(Config {
                rom_path: args[1].clone(),
            })
        } else if let Ok(val) = env::var("OXID_ROM") {
            Ok(Config { rom_path: val })
        } else {
            Err("not enough arguments")
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = Config::build(&args).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {err}");
        process::exit(1);
    });

    if let Err(e) = run(config) {
        eprintln!("Application error: {e}");
        process::exit(1);
    }
}

fn run(config: Config) -> io::Result<()> {
    let mut emu = Emu::default();
    emu.core.load_rom(&config.rom_path)?;
    emu.core.load_font();

    let mut terminal = ratatui::init();
    terminal.clear()?;

    let size = terminal.size()?;
    let width = size.width as f64;
    let height = size.width as f64;

    while !emu.state.should_exit {
        let time = Instant::now();

        let key = if event::poll(Duration::from_millis(1))? {
            handle_events(&mut emu.state)?
        } else {
            None
        };

        // WARN: testing things
        if let Some(k) = key {
            eprintln!("{}", k);
        }

        if let Err(err) = emu.core.run_cycle(key) {
            eprintln!("{err}");
        }

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

fn handle_events(emu_state: &mut EmuState) -> io::Result<Option<u8>> {
    match event::read()? {
        Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
            Ok(handle_key_event(key_event, emu_state))
        }
        _ => Ok(None),
    }
}

fn handle_key_event(key_event: KeyEvent, emu_state: &mut EmuState) -> Option<u8> {
    match key_event.code {
        KeyCode::Esc => {
            emu_state.should_exit = true;
            None
        }
        KeyCode::Char('1') => Some(0x0),
        KeyCode::Char('2') => Some(0x1),
        KeyCode::Char('3') => Some(0x2),
        KeyCode::Char('4') => Some(0x3),
        KeyCode::Char('q') => Some(0x4),
        KeyCode::Char('w') => Some(0x5),
        KeyCode::Char('e') => Some(0x6),
        KeyCode::Char('r') => Some(0x7),
        KeyCode::Char('a') => Some(0x8),
        KeyCode::Char('s') => Some(0x9),
        KeyCode::Char('d') => Some(0xA),
        KeyCode::Char('f') => Some(0xB),
        KeyCode::Char('z') => Some(0xC),
        KeyCode::Char('x') => Some(0xD),
        KeyCode::Char('c') => Some(0xE),
        KeyCode::Char('v') => Some(0xF),
        _ => None,
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
