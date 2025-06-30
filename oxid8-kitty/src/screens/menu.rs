use crate::app::AppState;
use crate::screens::{Screen, widgets::title::Title};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Modifier, Style, palette::tailwind::SLATE},
    text::Line,
    widgets::{
        Block,            //
        HighlightSpacing, //
        List,             //
        ListItem,         //
        ListState,        //
        Paragraph,        //
        StatefulWidget,   //
        Widget,           //
    },
};
use std::io;

#[derive(Default)]
pub struct Menu {
    state: ListState,
}

const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);

impl Menu {
    pub fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    pub fn handle_events(&mut self, app_state: &mut AppState) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event, app_state)
            }
            _ => (),
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent, app_state: &mut AppState) {
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Esc => app_state.should_exit = true,
            KeyCode::Char('h') | KeyCode::Left => todo!(),
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
            KeyCode::Char('g') | KeyCode::Home => self.select_first(),
            KeyCode::Char('G') | KeyCode::End => self.select_last(),
            KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
                self.change_screen(app_state);
            }
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

    fn change_screen(&mut self, app_state: &mut AppState) {
        match self.state.selected().unwrap() {
            0 => app_state.screen = Screen::Game,
            _ => (),
        }
    }
}

/// Rendering logic for the app
impl Menu {
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

impl Widget for &mut Menu {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [top, body, bottom] = Layout::vertical([
            Constraint::Length(8),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(area);

        Menu::render_title(top, buf);
        Menu::render_footer(bottom, buf);
        self.render_menu(body, buf);
    }
}
