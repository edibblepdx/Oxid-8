use oxid_8::app::App;
use std::{io, time::Instant};

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = App::default().run(&mut terminal);
    ratatui::restore();

    let time = Instant::now();
    while time.elapsed().as_secs() < 1 {} // spin

    app_result
}
