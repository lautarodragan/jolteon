use std::time::Duration;

use ratatui::{
    layout::{Constraint, Layout},
    prelude::*,
    style::Style,
    text::Line,
    widgets::{Block, Borders, Gauge},
};

use crate::{theme::Theme, duration::duration_to_string, structs::Song};

pub fn song_to_string(song: &Song) -> String {
    let title = song.title.clone();

    if let Some(artist) = &song.artist {
        format!("{artist} - {title}")
    } else {
        title
    }
}

pub struct CurrentlyPlaying {
    theme: Theme,
    current_song: Option<Song>,
    current_song_position: Duration,
    queue_total_time: Duration,
    queue_song_count: usize,
    is_paused: bool,
    is_repeating: bool,
    frame: u64,
}

impl CurrentlyPlaying {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        theme: Theme,
        current_song: Option<Song>,
        current_song_position: Duration,
        queue_total_time: Duration,
        queue_song_count: usize,
        is_paused: bool,
        is_repeating: bool,
        frame: u64,
    ) -> Self {
        Self {
            theme,
            current_song,
            current_song_position,
            queue_total_time,
            queue_song_count,
            is_paused,
            is_repeating,
            frame,
        }
    }
}

fn render_anim(area: Rect, buf: &mut Buffer, frame: u64, theme: Theme) {
    const JOLTEON: &str = "jolteon";
    const STARS: [&str; 2] = ["+", "×"];
    const STAR_LEN: usize = 5;
    const REST_LEN: usize = 3;
    const FRAMES_PER_LETTER: usize = STAR_LEN + REST_LEN;
    const ANIM_LEN: usize = FRAMES_PER_LETTER * JOLTEON.len();

    let frame = (frame % (ANIM_LEN as u64 * 4)) as usize;

    if frame < ANIM_LEN {
        return;
    }

    let area = Rect {
        x: area.width / 2 - JOLTEON.len() as u16 / 2,
        y: area.y + 1,
        ..area
    };

    let frame = if frame < ANIM_LEN * 2 {
        frame - ANIM_LEN
    } else if frame >= ANIM_LEN * 3 {
        ANIM_LEN * 4 - frame - 1
    } else {
        JOLTEON.len() * FRAMES_PER_LETTER
    };

    let letter = frame / FRAMES_PER_LETTER;

    for i in 0..letter {
        buf[(area.x + i as u16, area.y)]
            .set_symbol(&JOLTEON[i..i + 1])
            .set_bg(theme.background)
            .set_fg(theme.foreground);
    }

    if letter < JOLTEON.len() {
        let part_of_letter = frame % FRAMES_PER_LETTER;
        if part_of_letter >= REST_LEN {
            let frame = part_of_letter - REST_LEN;
            buf[(area.x + letter as u16 + STAR_LEN as u16 - frame as u16, area.y)]
                .set_symbol(STARS[frame % 2])
                .set_bg(theme.background)
                .set_fg(theme.foreground);
        }
    };
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
        } else {
            render_anim(area_top, buf, self.frame, self.theme);
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
                        log::error!("Song length is zero! {:?}", song.path);
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
