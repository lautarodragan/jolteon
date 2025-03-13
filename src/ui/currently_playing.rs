use std::time::Duration;

use log::error;
use ratatui::{
    layout::{Constraint, Layout},
    prelude::*,
    style::Style,
    text::Line,
    widgets::{Block, Borders, Gauge},
};

use crate::{duration::duration_to_string, structs::Song};

pub fn song_to_string(song: &Song) -> String {
    let title = song.title.clone();

    if let Some(artist) = &song.artist {
        format!("{artist} - {title}")
    } else {
        title
    }
}

pub struct CurrentlyPlaying {
    theme: crate::config::Theme,
    current_song: Option<Song>,
    current_song_position: Duration,
    queue_total_time: Duration,
    queue_song_count: usize,
    is_paused: bool,
    is_repeating: bool,
}

impl CurrentlyPlaying {
    pub fn new(
        theme: crate::config::Theme,
        current_song: Option<Song>,
        current_song_position: Duration,
        queue_total_time: Duration,
        queue_song_count: usize,
        is_paused: bool,
        is_repeating: bool,
    ) -> Self {
        Self {
            theme,
            current_song,
            current_song_position,
            queue_total_time,
            queue_song_count,
            is_paused,
            is_repeating,
        }
    }
}

impl Widget for CurrentlyPlaying {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [area_top, area_bottom] = Layout::vertical([Constraint::Length(2), Constraint::Length(1)]).areas(area);

        if let Some(ref current_song) = self.current_song {
            let playing_file = Block::default()
                .style(Style::default().fg(self.theme.foreground))
                .title(song_to_string(current_song))
                .borders(Borders::NONE)
                .title_alignment(Alignment::Center)
                .title_position(ratatui::widgets::block::Position::Bottom);
            playing_file.render(area_top, buf);
        }

        let playing_song_label = self.current_song.as_ref().map(|song| {
            format!(
                "{time_played} / {current_song_length}",
                time_played = duration_to_string(self.current_song_position),
                current_song_length = duration_to_string(song.length),
            )
        });

        let songs = if self.queue_song_count == 1 { "song" } else { "songs" };

        let queue_label = if self.queue_song_count > 0 {
            Some(format!(
                "{queue_items} {songs} / {total_time} in queue",
                total_time = duration_to_string(self.queue_total_time),
                queue_items = self.queue_song_count,
            ))
        } else {
            None
        };

        let playing_gauge_label = match (playing_song_label, queue_label) {
            (Some(playing_song_label), Some(queue_label)) => {
                format!("{playing_song_label}  |  {queue_label}")
            }
            (None, Some(queue_label)) => queue_label.to_string(),
            (Some(playing_song_label), None) => playing_song_label.to_string(),
            _ => "".to_string(),
        };

        if !playing_gauge_label.is_empty() {
            let song_progress = match self.current_song {
                Some(ref song) => match song.length.as_secs_f64() {
                    0.0 => {
                        error!("Song length is zero! {:?}", song.path);
                        0.0
                    }
                    n => f64::clamp(self.current_song_position.as_secs_f64() / n, 0.0, 1.0),
                },
                _ => 0.0,
            };

            let playing_gauge = Gauge::default()
                .style(Style::default().fg(self.theme.foreground))
                .label(playing_gauge_label)
                .gauge_style(Style::default().fg(self.theme.background_selected))
                .use_unicode(true)
                .ratio(song_progress);
            playing_gauge.render(area_bottom, buf);
        }

        let [_, area_bottom_right] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Length(10)]).areas(area_bottom);

        if self.is_paused {
            Line::from("PAUSED")
                .style(Style::default().fg(self.theme.foreground).bg(self.theme.background))
                .alignment(Alignment::Right)
                .render(area_bottom_right, buf);
        } else if self.is_repeating {
            Line::from("REPEAT ONE")
                .style(Style::default().fg(self.theme.foreground).bg(self.theme.background))
                .alignment(Alignment::Right)
                .render(area_bottom_right, buf);
        }
    }
}
