use crossterm::{
    cursor,
    event::{
        self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, KeyboardEnhancementFlags,
        PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
    },
    queue,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use oxid8_core::{CPU_TICK, Oxid8, SCREEN_HEIGHT, SCREEN_WIDTH, TIMER_TICK};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Flex, Layout, Rect},
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
    process,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

struct Config {
    pub rom_path: String,
}

#[derive(Default)]
struct Emu {
    core: Oxid8,
    state: EmuState,
}

struct EmuState {
    should_exit: bool,
    area: Rect,
    enhanced: bool,
}

struct Terminal;

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

impl Default for EmuState {
    fn default() -> Self {
        Self {
            should_exit: false,
            area: Rect::default(),
            enhanced: matches!(
                crossterm::terminal::supports_keyboard_enhancement(),
                Ok(true)
            ),
        }
    }
}

impl Terminal {
    /// Create and return a new Terminal (can fail)
    pub fn init() -> io::Result<ratatui::Terminal<CrosstermBackend<Stdout>>> {
        Terminal::enter()?;
        let stdout = io::stdout();
        ratatui::Terminal::new(CrosstermBackend::new(stdout))
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

        if matches!(
            crossterm::terminal::supports_keyboard_enhancement(),
            Ok(true)
        ) {
            queue!(stdout, PopKeyboardEnhancementFlags)?;
        }

        crossterm::terminal::disable_raw_mode()?;
        crossterm::execute!(stdout, LeaveAlternateScreen, cursor::Show)?;

        Ok(())
    }

    pub fn suspend() -> io::Result<()> {
        Terminal::exit()?;
        #[cfg(not(windows))]
        signal_hook::low_level::raise(SIGTSTP)?;
        Ok(())
    }

    pub fn resume() -> io::Result<()> {
        Terminal::enter()
    }
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let config = Config::build(&args).unwrap_or_else(|err| {
        eprintln!("Error parsing arguments: {err}");
        process::exit(1);
    });

    if let Err(e) = run(config) {
        eprintln!("Application error: {e}");
        Terminal::exit()?;
        process::exit(1);
    }

    Ok(())
}

fn run(config: Config) -> io::Result<()> {
    // Install Signal Hooks
    let (tx, rx) = mpsc::channel();
    let mut signals = Signals::new([SIGCONT])?;
    thread::spawn(move || {
        for signal in signals.forever() {
            if tx.send(signal).is_err() {
                break; // Main thread terminated
            }
        }
    });

    // Terminal
    let mut terminal = Terminal::init()?;
    terminal.clear()?;

    // Emulator
    let mut emu = Emu::default();
    emu.core.load_rom(&config.rom_path)?;
    emu.core.load_font();

    let mut last_cpu_tick = Instant::now();
    let mut last_timer_tick = Instant::now();

    while !emu.state.should_exit {
        let time = Instant::now();

        // Poll Signals
        if let Ok(signal) = rx.try_recv() {
            match signal {
                SIGCONT => {
                    Terminal::resume()?;
                    terminal.clear()?;
                }
                _ => (),
            }
        }

        // Emu Cycle
        if time.duration_since(last_cpu_tick) >= CPU_TICK {
            if event::poll(Duration::from_secs(0))? {
                handle_events(&mut emu)?;
            }

            if let Err(err) = emu.core.run_cycle() {
                eprintln!("{err}");
            }

            // To support more terminals
            if !emu.state.enhanced {
                emu.core.clear_keys();
            }

            last_cpu_tick += CPU_TICK;
        }

        // Decrement Timers
        if time.duration_since(last_timer_tick) >= TIMER_TICK {
            emu.core.dec_timers();
            last_timer_tick += TIMER_TICK;

            terminal.draw(|frame| {
                // Clipping area
                emu.state.area = frame.area();

                // Rendering half-blocks
                let width = SCREEN_WIDTH;
                let height = SCREEN_HEIGHT / 2;

                // Drawing area
                let area = center(
                    frame.area(),
                    Constraint::Length(width as u16),
                    Constraint::Length(height as u16),
                );

                frame.render_widget(
                    Canvas::default()
                        .x_bounds([0.0, width as f64])
                        .y_bounds([0.0, height as f64])
                        .marker(Marker::HalfBlock)
                        .paint(|ctx| {
                            ctx.draw(&emu);
                        }),
                    area,
                )
            })?;
        }

        if emu.core.sound() {
            print!("\x07");
        }
    }

    Terminal::exit()
}

fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}

fn handle_events(emu: &mut Emu) -> io::Result<()> {
    match event::read()? {
        Event::Key(KeyEvent {
            code: KeyCode::Char('z'),
            modifiers: KeyModifiers::CONTROL,
            ..
        }) => return Terminal::suspend(),
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
