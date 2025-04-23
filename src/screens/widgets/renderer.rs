use crate::*;

use ratatui::{
    buffer::{Buffer, Cell},
    layout::{Position, Rect},
    style::{Color, Style},
};

fn draw() {
    let mut buffer = Buffer::empty(Rect::new(
        0,
        0,
        SCREEN_WIDTH as u16 * 2,
        SCREEN_HEIGHT as u16,
    ));

    for y in 0..SCREEN_HEIGHT as u16 {
        for x in (0..SCREEN_WIDTH as u16 * 2).step_by(2) {
            buffer[Position { x, y }].set_symbol("█");
            buffer[Position { x: x + 1, y }].set_symbol("█");
        }
    }
}


/*
pub mod tui;

pub trait Renderer {
    fn draw(&mut self);
}
*/

use ratatui::{
    buffer::{Buffer, Cell},
    layout::{Position, Rect},
    style::{Color, Style},
};

fn main() -> Option<&mut Cell> {
    let mut buf = Buffer::empty(Rect {
        x: 0,
        y: 0,
        width: crate::SCREEN_WIDTH as u16,
        height: crate::SCREEN_HEIGHT as u16,
    });

    // indexing using Position
    buf[Position { x: 0, y: 0 }].set_symbol("A");
    assert_eq!(buf[Position { x: 0, y: 0 }].symbol(), "A");

    // indexing using (x, y) tuple (which is converted to Position)
    buf[(0, 1)].set_symbol("B");
    assert_eq!(buf[(0, 1)].symbol(), "x");

    // getting an Option instead of panicking if the position is outside the buffer
    let cell = buf.cell_mut(Position { x: 0, y: 2 })?;
    cell.set_symbol("C");

    let cell = buf.cell(Position { x: 0, y: 2 })?;
    assert_eq!(cell.symbol(), "C");

    buf.set_string(
        3,
        0,
        "string",
        Style::default().fg(Color::Red).bg(Color::White),
    );
    let cell = &buf[(5, 0)]; // cannot move out of buf, so we borrow it
    assert_eq!(cell.symbol(), "r");
    assert_eq!(cell.fg, Color::Red);
    assert_eq!(cell.bg, Color::White);

    Some(())
}
*/
