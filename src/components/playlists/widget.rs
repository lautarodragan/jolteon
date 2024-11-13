use std::{
    time::{SystemTime, UNIX_EPOCH},
    sync::{
        atomic::{AtomicUsize, AtomicBool, Ordering},
        Mutex,
    },
};

use ratatui::{
    prelude::Widget,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Style,
    widgets::{WidgetRef},
};

use crate::ui::song_to_string;

use super::Playlists;

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

        for i in 0..playlists.len() {
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

        let selected_playlist = &playlists[selected_playlist_index];

        for i in 0..selected_playlist.songs.len() {
            let song = &selected_playlist.songs[i];
            let area = Rect {
                y: area_right.y + i as u16,
                height: 1,
                ..area_right
            };

            let style = if i == selected_song {
                if *focused_element == crate::components::playlists::playlists::PlaylistScreenElement::SongList {
                    Style::default().fg(self.theme.foreground_selected).bg(self.theme.background_selected)
                } else {
                    Style::default().fg(self.theme.foreground_selected).bg(self.theme.background_selected_blur)
                }
            } else {
                Style::default().fg(self.theme.foreground_secondary).bg(self.theme.background)
            };

            let line = ratatui::text::Line::from(song_to_string(song)).style(style);
            line.render_ref(area, buf);
        }
    }
}
