use crate::app::{App, AppTab};
use crate::config::Config;
use crate::constants::{SECONDS_PER_HOUR, SECONDS_PER_MINUTE};
use crate::helpers::gen_funcs::{Song, song_to_string};
use crate::ui::{instructions_tab, music_tab};
use ratatui::widgets::{BorderType, Gauge};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    prelude::*,
    style::{Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Tabs},
    Frame,
};
use std::time::Duration;

static MAIN_SECTIONS: [&str; 2] = ["Music", "Help"];

fn duration_to_string(total_time: Duration) -> String {
    let hours = total_time.as_secs() / SECONDS_PER_HOUR;
    let minutes = (total_time.as_secs() % SECONDS_PER_HOUR) / SECONDS_PER_MINUTE;
    let seconds = total_time.as_secs() % SECONDS_PER_MINUTE;

    let mut time_parts = vec![];

    if hours > 0 {
        time_parts.push(hours);
    }

    time_parts.push(minutes);
    time_parts.push(seconds);

    let strings: Vec<String> = time_parts.iter().map(|s| format!("{:0>2}", s)).collect();
    strings.join(":")
}

pub fn render_ui(f: &mut Frame, app: &mut App, cfg: &Config, current_song: &Option<Song>) {
    let size = f.size();

    let main_layouts = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(2),
                Constraint::Min(0),
                Constraint::Length(2),
                Constraint::Length(1),
            ]
            .as_ref(),
        )
        .split(size);

    let block = Block::default().style(Style::default().bg(cfg.background()));
    f.render_widget(block, size);

    let tab_titles: Vec<Line> = MAIN_SECTIONS
        .iter()
        .map(|t| {
            Line::from(Span::styled(
                t.to_string(),
                Style::default().fg(cfg.foreground()),
            ))
        })
        .collect();

    let tabs = Tabs::new(tab_titles)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_type(BorderType::Plain)
                .border_style(Style::default().fg(cfg.background()).bg(cfg.background())),
        )
        .select(app.active_tab as usize)
        .style(
            Style::default()
                .fg(cfg.foreground())
                .bg(Color::from_hsl(29.0, 34.0, 20.0)),
        )
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::from_hsl(39.0, 67.0, 69.0)), // .bg(cfg.highlight_background()),
        );
    f.render_widget(tabs, main_layouts[0]);

    match app.active_tab {
        AppTab::Music => music_tab(f, app, main_layouts[1], cfg),
        AppTab::Controls => instructions_tab(f, app, main_layouts[1], cfg),
    };

    render_playing_gauge(f,main_layouts[2], main_layouts[3], current_song, app, cfg);
}

fn render_playing_gauge(f: &mut Frame, main_layouts: Rect, main_layouts2: Rect, current_song: &Option<Song>, app: &mut App, cfg: &Config) {
    if let Some(current_song) = current_song {
        let playing_file = Block::default()
            .style(Style::default().fg(cfg.foreground()))
            .title(song_to_string(&current_song))
            .borders(Borders::NONE)
            .title_alignment(Alignment::Center)
            .title_position(ratatui::widgets::block::Position::Bottom);
        f.render_widget(playing_file, main_layouts);
    }

    let playing_gauge_label_current_song = match current_song {
        Some(song) => format!(
            "{time_played} / {current_song_length}",
            time_played = duration_to_string(app.sink().get_pos()),
            current_song_length = duration_to_string(song.length),
        ),
        _ => "".to_string(),
    };

    let playing_gauge_label = format!(
        "{playing_gauge_label_current_song} — {total_time}, {queue_items} songs",
        total_time = duration_to_string(app.queue_items.total_time()),
        queue_items = app.queue_items.length(),
    );

    let playing_gauge = Gauge::default()
        .style(Style::default().fg(cfg.foreground()))
        .label(playing_gauge_label)
        .gauge_style(Style::default().fg(cfg.highlight_background()))
        .ratio(app.song_progress());
    f.render_widget(playing_gauge, main_layouts2);
}