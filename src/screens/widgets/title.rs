use indoc::indoc;
use ratatui::{buffer::Buffer, layout::Rect, text::Text, widgets::Widget};

pub struct Title;

impl Title {
    pub const WIDTH: u16 = 22;
    pub const HEIGHT: u16 = 3;

    const fn title() -> &'static str {
        indoc! {"
            ▄▄▄▄              ▄▄▄▄
            █  █ ▜▄▛ █ █▀▄ ▄▄ █▄▄█
            █▄▄█ █ █ █ █▄▀    █▄▄█
        "}
    }
    /*
    const fn title() -> &'static str {
        indoc! {"
                ▄  ▄▄▄▄              ▄▄▄▄  ▄▄▄▄▄
              ▄██  █  █ ▜▄▛ █ █▀▄ ▄▄ █▄▄█  ███▀
            ▄████  █▄▄█ █ █ █ █▄▀    █▄▄█  █▀
        "}
    }
    */
}

impl Widget for Title {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Text::raw(Title::title()).render(area, buf);
    }
}
