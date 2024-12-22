use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, BorderType, Borders, Cell, Row, Table, TableState, WidgetRef},
};

use crate::{config::Theme, ui::KeyboardHandlerMut};

pub struct Help<'a> {
    theme: Theme,
    header: Vec<&'a str>,
    items: Vec<Vec<&'a str>>,
    state: TableState,
}

impl Help<'_> {
    pub fn new(theme: Theme) -> Self {
        Self {
            theme,
            header: vec!["Keys", "Commands"],
            items: vec![
                vec!["Q", "Quit"],
                vec!["P", "Play / Pause"],
                vec!["G", "Skip Song"],
                vec!["A", "Add To Queue"],
                vec!["R", "Remove From Queue"],
                vec!["Enter", "Enter Directory"],
                vec!["Backspace", "Previous Directory"],
                vec!["Down", "Next Item"],
                vec!["Up", "Previous Item"],
                vec!["Right / Left", "Enter Queue / Browser"],
                vec!["Tab", "Change Tabs"],
                vec!["+", "Volume Up"],
                vec!["-", "Volume Down"],
            ],
            state: TableState::default(),
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

impl<'a> KeyboardHandlerMut<'a> for Help<'a> {
    fn on_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => self.next(),
            KeyCode::Up | KeyCode::Char('k') => self.previous(),
            _ => {}
        }
    }
}

impl WidgetRef for Help<'_> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let [area_top, area_main] = Layout::vertical([Constraint::Max(5), Constraint::Max(5)])
            .horizontal_margin(2)
            .areas(area);

        ratatui::widgets::Paragraph::new(vec![
            ratatui::text::Line::raw("Hi! Welcome to Jolteon."),
            ratatui::text::Line::raw("Lorem ipsum yada yada etcLorem ipsum yada yada etcLorem ipsum yada yada etcLorem ipsum yada yada etcLorem ipsum yada yada etcLorem ipsum yada yada etcLorem ipsum yada yada etcLorem ipsum yada yada etcLorem ipsum yada yada etcLorem ipsum yada yada etcLorem ipsum yada yada etc"),
        ])
            .wrap(ratatui::widgets::Wrap { trim: true })
            .render_ref(area_top, buf);

        let header = self
            .header
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().fg(self.theme.foreground_selected)));

        let header = Row::new(header)
            .style(Style::default().bg(self.theme.background).fg(self.theme.foreground))
            .height(1)
            .bottom_margin(0);

        let rows = self.items.iter().map(|item| {
            let height = item
                .iter()
                .map(|content| content.chars().filter(|c| *c == '\n').count())
                .max()
                .unwrap_or(0)
                + 1;
            let cells = item.iter().map(|c| Cell::from(*c));
            Row::new(cells).height(height as u16).bottom_margin(0)
        });

        let widths = [Constraint::Length(5), Constraint::Length(10)];

        let table = Table::new(rows, widths)
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .title(" Controls ")
                    .title_alignment(Alignment::Center)
                    .border_type(BorderType::Plain),
            )
            .style(Style::default().fg(self.theme.foreground).bg(self.theme.background))
            .row_highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(self.theme.background_selected)
                    .fg(self.theme.foreground_selected),
            )
            .widths([Constraint::Percentage(50), Constraint::Length(30), Constraint::Min(10)]);

        ratatui::widgets::StatefulWidgetRef::render_ref(&table, area_main, buf, &mut self.state.clone());
        // table.render_ref(layout[0], buf, &mut self.state.clone());
    }
}

impl Drop for Help<'_> {
    fn drop(&mut self) {
        log::trace!("HelpTab.drop()");
    }
}
