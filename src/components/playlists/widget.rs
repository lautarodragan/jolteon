use std::fmt::{Display, Formatter};

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    prelude::Alignment,
    style::Style,
    widgets::WidgetRef,
};

use super::Playlists;

impl Display for crate::structs::Song {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} - {} - {} - {}",
            self.year
                .as_ref()
                .map(|y| y.to_string())
                .unwrap_or("(no year)".to_string()),
            self.album.clone().unwrap_or("(no album)".to_string()),
            self.track.unwrap_or(0),
            self.title.clone()
        )
    }
}

impl Display for crate::structs::Playlist {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl WidgetRef for Playlists<'_> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let [area_left, _, area_right] = Layout::horizontal([
            Constraint::Percentage(50),
            Constraint::Length(5),
            Constraint::Percentage(50),
        ])
        .horizontal_margin(2)
        .areas(area);

        let show_deleted_playlists = self.show_deleted_playlists.get();

        if show_deleted_playlists {
            let [left_top, _, left_bottom_header, _, left_bottom_list] = Layout::vertical([
                Constraint::Percentage(50),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Percentage(50),
            ])
            .areas(area_left);

            self.playlist_list.render_ref(left_top, buf);

            let block = ratatui::widgets::Block::new()
                .borders(ratatui::widgets::Borders::TOP)
                .border_style(Style::new().fg(self.theme.foreground))
                .title(" Playlist Graveyard ")
                .title_style(Style::new().fg(self.theme.foreground))
                .title_alignment(Alignment::Center);
            block.render_ref(left_bottom_header, buf);

            self.deleted_playlist_list.render_ref(left_bottom_list, buf);
        } else {
            self.playlist_list.render_ref(area_left, buf);
        }

        self.song_list.render_ref(area_right, buf);
    }
}
