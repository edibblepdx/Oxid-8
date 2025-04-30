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
    area: ratatui::layout::Rect,
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

    while !emu.state.should_exit {
        let time = Instant::now();

        let key = if event::poll(Duration::from_millis(1))? {
            handle_events(&mut emu.state)?
        } else {
            None
        };

        // WARN: testing things
        /*
        if let Some(k) = key {
            eprintln!("{}", k);
        }
        */

        if let Err(err) = emu.core.run_cycle(key) {
            eprintln!("{err}");
        }

        let _ = terminal.draw(|frame| {
            let area = frame.area();
            emu.state.area = area;

            frame.render_widget(
                Canvas::default()
                    .x_bounds([0.0, SCREEN_WIDTH as f64])
                    .y_bounds([0.0, SCREEN_HEIGHT as f64])
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
        /*
         * 1 2 3 C
         * 4 5 6 D
         * 7 8 9 E
         * A 0 B f
         */
        KeyCode::Char('1') => Some(0x1),
        KeyCode::Char('2') => Some(0x2),
        KeyCode::Char('3') => Some(0x3),
        KeyCode::Char('4') => Some(0xC),
        KeyCode::Char('q') => Some(0x4),
        KeyCode::Char('w') => Some(0x5),
        KeyCode::Char('e') => Some(0x6),
        KeyCode::Char('r') => Some(0xD),
        KeyCode::Char('a') => Some(0x7),
        KeyCode::Char('s') => Some(0x8),
        KeyCode::Char('d') => Some(0x9),
        KeyCode::Char('f') => Some(0xE),
        KeyCode::Char('z') => Some(0xA),
        KeyCode::Char('x') => Some(0x0),
        KeyCode::Char('c') => Some(0xB),
        KeyCode::Char('v') => Some(0xF),
        _ => None,
    }
}

impl Shape for Emu {
    fn draw(&self, painter: &mut Painter) {
        let screen_ref = self.core.screen_ref();
        for y in 0..SCREEN_HEIGHT {
            for x in 0..SCREEN_WIDTH {
                if screen_ref[x + y * SCREEN_WIDTH]
                    && x < self.state.area.width as usize
                    && y < (self.state.area.height * 2) as usize
                // WARN: ONLY for rendering half-blocks
                {
                    painter.paint(x, y, Color::White);
                }
            }
        }
    }

    /*********************************************************************
     * This scales to terminal size but it looks pretty bad in my opinion
     *********************************************************************
    fn draw(&self, painter: &mut Painter) {
        let screen_ref = self.core.screen_ref();
        for y in 0..SCREEN_HEIGHT {
            for x in 0..SCREEN_WIDTH {
                if screen_ref[x + y * SCREEN_WIDTH] {
                    let y = SCREEN_HEIGHT - 1 - y;
                    if let Some((x, y)) = painter.get_point(x as f64, y as f64) {
                        painter.paint(x, y, Color::White);
                    }
                }
            }
        }
    }
    */
}
