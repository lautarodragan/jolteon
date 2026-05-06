use ratatui::{
    buffer::Buffer,
    layout::{Offset, Rect},
    prelude::{Style, Widget},
    style::Modifier,
    text::{Line, Span},
};

use super::{CommandLine, Query, QueryAddSongsTarget};

impl Widget for &CommandLine<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let Some(query) = self.query.as_ref() else {
            return;
        };
        let line = match query {
            Query::AddSongs {
                songs,
                step,
                target,
                target_name,
                ..
            } => {
                let style = if *step == 0 {
                    Style::default().bg(self.theme.background_selected)
                } else {
                    Style::default().bg(self.theme.background_selected_blur)
                };
                let mut spans = vec![
                    Span::from(format!("Add {} song(s) to", songs.len())),
                    Span::from(" "),
                    Span::from(target.to_string()).style(style),
                ];
                if *target == QueryAddSongsTarget::Playlist {
                    let tn = target_name.as_ref().map_or("", String::as_str);
                    let tn = if *step == 0 {
                        Span::from(tn).style(Style::default().bg(self.theme.background_selected_blur))
                    } else {
                        Span::from(tn).style(Style::default().bg(self.theme.background_selected))
                    };
                    spans.push(Span::from(" "));
                    spans.push(tn);
                }

                spans.push(Span::from("?"));
                spans.push(Span::from(" "));
                spans.push(
                    Span::from("Enter to confirm, Left/Right Arrows to change selection, Esc to cancel")
                        .style(Style::default().add_modifier(Modifier::DIM)),
                );

                Line::from(spans)
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
