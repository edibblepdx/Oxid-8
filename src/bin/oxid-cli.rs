use crossterm::{
    cursor,
    event::{
        self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, KeyboardEnhancementFlags,
        PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
    },
    queue,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use oxid8::core::{Oxid8, SCREEN_HEIGHT, SCREEN_WIDTH};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    style::Color,
    symbols::Marker,
    widgets::canvas::{Canvas, Painter, Shape},
};
use signal_hook::{
    consts::{SIGCONT, SIGTSTP},
    iterator::Signals,
};
use std::{
    env,
    io::{self, Stdout},
    process, thread,
    time::{Duration, Instant},
};

const CPU_TICK: Duration = Duration::from_micros(1430); // 700Hz
const TIMER_TICK: Duration = Duration::from_micros(16667); // 60Hz

struct Config {
    pub rom_path: String,
}

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

struct Term;

impl Term {
    pub fn init() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
        Term::enter()?;
        let stdout = io::stdout();
        Terminal::new(CrosstermBackend::new(stdout))
    }

    pub fn enter() -> io::Result<()> {
        let mut stdout = io::stdout();

        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(stdout, EnterAlternateScreen, cursor::Hide)?;

        if matches!(
            crossterm::terminal::supports_keyboard_enhancement(),
            Ok(true)
        ) {
            queue!(
                stdout,
                PushKeyboardEnhancementFlags(
                    KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
                        | KeyboardEnhancementFlags::REPORT_EVENT_TYPES
                        | KeyboardEnhancementFlags::REPORT_ALTERNATE_KEYS
                        | KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES
                )
            )?;
        }

        Ok(())
    }

    pub fn exit() -> io::Result<()> {
        let mut stdout = io::stdout();

        crossterm::terminal::disable_raw_mode()?;
        crossterm::execute!(stdout, LeaveAlternateScreen, cursor::Show)?;

        if matches!(
            crossterm::terminal::supports_keyboard_enhancement(),
            Ok(true)
        ) {
            queue!(stdout, PopKeyboardEnhancementFlags)?;
        }

        ratatui::restore();
        Ok(())
    }

    pub fn suspend() -> io::Result<()> {
        Term::exit()?;
        #[cfg(not(windows))]
        signal_hook::low_level::raise(SIGTSTP)?;
        Ok(())
    }

    pub fn resume() -> io::Result<()> {
        Term::enter()
    }
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let config = Config::build(&args).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {err}");
        process::exit(1);
    });

    let mut signals = Signals::new([SIGCONT])?;
    thread::spawn(move || {
        for _ in signals.forever() {
            Term::resume().unwrap_or_else(|_| panic!("ERROR::Failed to resume terminal."));
        }
    });

    if let Err(e) = run(config) {
        eprintln!("Application error: {e}");
        Term::exit()?;
        process::exit(1);
    }

    Ok(())
}

fn run(config: Config) -> io::Result<()> {
    let mut emu = Emu::default();
    emu.core.load_rom(&config.rom_path)?;
    emu.core.load_font();

    let mut terminal = Term::init()?;
    terminal.clear()?;

    let mut last_cpu_tick = Instant::now();
    let mut last_timer_tick = Instant::now();

    while !emu.state.should_exit {
        let time = Instant::now();

        if time.duration_since(last_cpu_tick) >= CPU_TICK {
            if event::poll(Duration::from_secs(0))? {
                handle_events(&mut emu)?;
            }

            if let Err(err) = emu.core.run_cycle() {
                eprintln!("{err}");
            }

            terminal.draw(|frame| {
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
            })?;

            last_cpu_tick += CPU_TICK;
        }

        if time.duration_since(last_timer_tick) >= TIMER_TICK {
            emu.core.dec_timers();
            last_timer_tick += TIMER_TICK;
        }

        if emu.core.sound() {
            print!("\x07");
        }
    }

    Term::exit()
}

fn handle_events(emu: &mut Emu) -> io::Result<()> {
    match event::read()? {
        Event::Key(KeyEvent {
            code: KeyCode::Char('z'),
            modifiers: KeyModifiers::CONTROL,
            ..
        }) => return Term::suspend(),
        Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
            if let Some(k) = handle_key_event(key_event, &mut emu.state) {
                emu.core.set_key(k as usize, true);
            }
        }
        Event::Key(key_event) if key_event.kind == KeyEventKind::Release => {
            if let Some(k) = handle_key_event(key_event, &mut emu.state) {
                emu.core.set_key(k as usize, false);
            }
        }
        _ => (),
    }
    Ok(())
}

fn handle_key_event(key_event: KeyEvent, state: &mut EmuState) -> Option<u8> {
    match key_event.code {
        KeyCode::Esc => {
            state.should_exit = true;
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
