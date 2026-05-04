use ratatui::{
    buffer::Buffer,
    layout::{Offset, Rect},
    prelude::{Style, Widget},
    style::Modifier,
    text::{Line, Span},
};

use super::{CommandLine, Query};

impl Widget for &CommandLine<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let Some(query) = self.query.as_ref() else {
            return;
        };
        let line = match query {
            Query::AddSongs { songs, target } => {
                let s1 = Span::from(format!("Add {} song(s) to", songs.len()));
                let s2 = Span::from(target.to_string()).style(Style::default().bg(self.theme.background_selected));
                let s3 = Span::from("Enter to confirm, Left/Right Arrows to change selection, Esc to cancel")
                    .style(Style::default().add_modifier(Modifier::DIM));
                Line::from(vec![s1, Span::from(" "), s2, Span::from("?"), Span::from(" "), s3])
            }
        };
        line.render(area, buf);
        if let Some(error) = self.query_error.as_ref() {
            let area = area.offset(Offset::new(0, 1));
            Line::from(error.as_ref())
                .style(Style::default().fg(self.theme.search))
                .render(area, buf);
        }
    }
}
