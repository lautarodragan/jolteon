use std::sync::atomic::Ordering;

use ratatui::{
    prelude::Widget,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    widgets::{WidgetRef},
};

use super::component::SongList;

fn line_style(theme: &crate::config::Theme, index: usize, selected_index: usize, list_has_focus: bool) -> Style {
    if index == selected_index {
        if list_has_focus {
            Style::default().fg(theme.foreground_selected).bg(theme.background_selected)
        } else {
            Style::default().fg(theme.foreground_selected).bg(theme.background_selected_blur)
        }
    } else {
        Style::default().fg(theme.foreground_secondary).bg(theme.background)
    }
}

impl<'a> Widget for SongList<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        WidgetRef::render_ref(&self, area, buf);
    }
}

impl<'a> WidgetRef for SongList<'a> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        self.height.store(area.height as usize, Ordering::Relaxed);

        let songs = &self.songs.lock().unwrap();

        if songs.len() < 1 {
            return;
        }

        let selected_song_index = self.selected_song_index.load(Ordering::Relaxed);
        let offset = self.offset.load(Ordering::Relaxed);

        for i in 0..songs.len().min(area.height as usize) {
            let song_index = i + offset;

            if song_index >= songs.len() {
                log::error!("song index {song_index} > song_list.len() {} offset={offset}", songs.len());
                break;
            }

            let song = &songs[song_index];
            let area = Rect {
                y: area.y + i as u16,
                height: 1,
                ..area
            };

            let style = line_style(&self.theme, song_index, selected_song_index, true);
            let line = ratatui::text::Line::from(
                format!("{} - {} - {} - {}",
                        song.year.as_ref().map(|y| y.to_string()).unwrap_or("(no year)".to_string()),
                        song.album.clone().unwrap_or("(no album)".to_string()),
                        song.track.unwrap_or(0),
                        song.title.clone()
                ),
            ).style(style);

            line.render_ref(area, buf);
        }
    }
}
