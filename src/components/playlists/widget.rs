use std::{
    time::{SystemTime, UNIX_EPOCH},
    sync::{
        atomic::{Ordering},
    },
    fmt::{Display, Formatter},
};

use ratatui::{
    prelude::Widget,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Style,
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

        let playlists = self.playlists.lock().unwrap();

        if playlists.len() < 1 {
            return;
        }

        let selected_playlist_index = self.selected_playlist_index.load(Ordering::Relaxed);
        let selected_song = self.selected_song_index.load(Ordering::Relaxed);
        let focused_element = self.focused_element.lock().unwrap();
        let is_renaming = self.renaming.load(Ordering::Relaxed);

        for i in 0..playlists.len().min(area_right.height as usize) {
            let playlist = &playlists[i];
            let area = Rect {
                y: area_left.y + i as u16,
                height: 1,
                ..area_left
            };

            let style = if i == selected_playlist_index {
                if *focused_element == crate::components::playlists::playlists::PlaylistScreenElement::PlaylistList {
                    if is_renaming {
                        Style::default().fg(self.theme.foreground_selected).bg(self.theme.search)
                    } else {
                        Style::default().fg(self.theme.foreground_selected).bg(self.theme.background_selected)
                    }
                } else {
                    Style::default().fg(self.theme.foreground_selected).bg(self.theme.background_selected_blur)
                }
            } else {
                Style::default().fg(self.theme.foreground_secondary).bg(self.theme.background)
            };

            let line = if is_renaming && i == selected_playlist_index {
                let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
                let caret = if now % 500 < 250 {
                    '⎸'
                } else {
                    ' '
                };
                format!("{}{}", playlist.name, caret)
            } else {
                playlist.name.clone()
            };

            let line = ratatui::text::Line::from(line).style(style);

            line.render_ref(area, buf);
        }

        if selected_playlist_index >= playlists.len() {
            log::error!("selected_playlist_index >= playlists.len()");
            return;
        }

        self.song_list.render_ref(area_right, buf);
    }
}
