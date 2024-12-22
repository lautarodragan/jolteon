use chrono::prelude::*;
use ratatui::{
    prelude::*,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Tabs},
};

// static TIME_FORMAT: &str = "%A %-l:%M%P, %B %-e | %F";
// static TIME_FORMAT: &str = "%A %-l:%M%P, %B %-e";
static TIME_FORMAT: &str = "%A %-l:%M%P";

fn time_format() -> String {
    // let st = match Local::now().day() {
    //     1 | 21 | 31 => "st",
    //     2 | 22 => "nd",
    //     3 | 23 => "rd",
    //     _ => "th",
    // };

    // Local::now().format(format!("%A %-l:%M%P, %B %-e{st}").as_str()).to_string()
    // Local::now().format(format!("%A %-l:%M%P").as_str()).to_string()
    Local::now().format(TIME_FORMAT).to_string()
}

pub struct TopBar<'a> {
    theme: crate::config::Theme,
    tab_titles: &'a [&'a str],
    active_tab: usize,
    frame_count: u64,
}

impl<'a> TopBar<'a> {
    pub fn new(theme: crate::config::Theme, tab_titles: &'a [&'a str], active_tab: usize, frame_count: u64) -> Self {
        Self {
            theme,
            tab_titles,
            active_tab,
            frame_count,
        }
    }
}

impl Widget for TopBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let tab_titles: Vec<Line> = self
            .tab_titles
            .iter()
            .map(|t| {
                Line::from(Span::styled(
                    (**t).to_string(),
                    Style::default().fg(self.theme.foreground),
                ))
            })
            .collect();

        let tabs = Tabs::new(tab_titles)
            .block(Block::default())
            .select(self.active_tab)
            .style(
                Style::default()
                    .fg(self.theme.foreground)
                    .bg(self.theme.top_bar_background),
            )
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(self.theme.top_bar_foreground_selected),
            );
        tabs.render(area, buf);

        let clock = Line::from(time_format()).alignment(Alignment::Center);
        clock.render(area, buf);

        Line::from(format!("FRAME {}", self.frame_count))
            .style(Style::default())
            .right_aligned()
            .render(area, buf);
    }
}
