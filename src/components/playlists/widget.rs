use std::{
    fmt::{Display, Formatter},
};

use ratatui::{
    prelude::Widget,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::{WidgetRef},
};

use super::Playlists;

impl Display for crate::structs::Song {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {} - {} - {}",
                self.year.as_ref().map(|y| y.to_string()).unwrap_or("(no year)".to_string()),
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

impl<'a> Widget for Playlists<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        WidgetRef::render_ref(&self, area, buf);
    }
}

impl<'a> WidgetRef for Playlists<'a> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let [area_left, _, area_right] = Layout::horizontal([
            Constraint::Percentage(50),
            Constraint::Length(5),
            Constraint::Percentage(50),
        ])
            .horizontal_margin(2)
            .areas(area);

        let Ok(playlists) = self.playlist_list.try_borrow() else {
            return;
        };

        playlists.render_ref(area_left, buf);

        let Ok(song_list) = self.song_list.try_borrow() else {
            return;
        };
        song_list.render_ref(area_right, buf);
    }
}
