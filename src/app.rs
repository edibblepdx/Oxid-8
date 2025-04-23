use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use indoc::indoc;
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Modifier, Style, palette::tailwind::SLATE},
    text::{Line, Text},
    widgets::{
        Block, HighlightSpacing, List, ListItem, ListState, Paragraph, StatefulWidget, Widget,
    },
};
use std::io;

const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);

#[derive(Default)]
pub struct App {
    exit: bool,
    state: ListState,
}

struct Title;

impl App {
    pub fn run(mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => (),
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Esc => self.exit = true,
            KeyCode::Char('h') | KeyCode::Left => todo!(),
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
            KeyCode::Char('g') | KeyCode::Home => self.select_first(),
            KeyCode::Char('G') | KeyCode::End => self.select_last(),
            KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => todo!(),
            _ => (),
        }
    }

    fn select_next(&mut self) {
        self.state.select_next();
    }

    fn select_previous(&mut self) {
        self.state.select_previous();
    }

    fn select_first(&mut self) {
        self.state.select_first();
    }

    fn select_last(&mut self) {
        self.state.select_last();
    }
}

/// Rendering logic for the app
impl App {
    fn render_title(area: Rect, buf: &mut Buffer) {
        let [title] = Layout::vertical([Constraint::Length(Title::HEIGHT)])
            .flex(Flex::Center)
            .areas(area);
        let [title] = Layout::horizontal([Constraint::Length(Title::WIDTH)])
            .flex(Flex::Center)
            .areas(title);

        Widget::render(Title, title, buf);
    }

    fn render_menu(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::new().title(Line::raw("Menu").centered());

        let list = List::new([
            ListItem::from("Play"),
            ListItem::from("Load Rom"),
            ListItem::from("Debug"),
        ])
        .block(block)
        .highlight_style(SELECTED_STYLE)
        .highlight_symbol(">")
        .highlight_spacing(HighlightSpacing::Always);

        StatefulWidget::render(list, area, buf, &mut self.state);
    }

    fn render_footer(area: Rect, buf: &mut Buffer) {
        Paragraph::new(
            "Use ↓↑ to move, ← to go back, → to select, g/G to go top/bottom, q to quit.",
        )
        .centered()
        .render(area, buf);
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [top, body, bottom] = Layout::vertical([
            Constraint::Length(8),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(area);

        App::render_title(top, buf);
        App::render_footer(bottom, buf);
        self.render_menu(body, buf);
    }
}

impl Title {
    const WIDTH: u16 = 22;
    const HEIGHT: u16 = 3;

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
