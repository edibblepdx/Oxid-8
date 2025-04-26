use oxid8::app::App;
use oxid8::core::Oxid8;
use std::{
    io::{self, Write, stdout},
    time::Instant,
};

const TICK_RATE: u64 = 1 / 700; // 700 instructions per second

// NOTE: use bell character for a beep \X07
// NOTE: use the left four columns of 1234 for the keypad

// NOTE: maybe draw two rows per character because terminal characters are tall ▄ ▀ █
// or draw two columns per pixel ██ 128 is pretty wide though (probably easier to do)

fn main() -> io::Result<()> {
    print!("\x07");
    stdout().flush()?;

    let mut emu = Oxid8::new();
    if let Err(err) = emu.load_rom("abc") {
        eprintln!("{err}");
    }
    emu.load_font();

    let mut terminal = ratatui::init();
    let app_result = App::default().run(&mut terminal);
    ratatui::restore();

    loop {
        let time = Instant::now();

        // poll events (key press mainly)
        // run cycle
        if let Err(err) = emu.run_cycle(None) {
            eprintln!("{err}");
        }
        // check draw flag
        // check timers

        while time.elapsed().as_secs() < TICK_RATE {} // spin
        break; // WARN: temporary
    }

    app_result
}
