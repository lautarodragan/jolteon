use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    prelude::{Color, Style},
    widgets::WidgetRef,
};

pub struct RenderingError {
    theme: crate::config::Theme,
}

impl WidgetRef for RenderingError {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        ratatui::widgets::Block::new()
            .style(Style::new().bg(Color::Rgb(255, 0, 0)))
            .borders(ratatui::widgets::Borders::ALL)
            .border_style(Style::new().fg(Color::Rgb(255, 255, 255)))
            .render_ref(area.clone(), buf);

        let [_, area_center, _] = Layout::vertical([
            Constraint::Percentage(50),
            Constraint::Length(1),
            Constraint::Percentage(50),
        ])
            .areas(area);

        ratatui::text::Line::from("RENDERING ERROR").style(Style::new().fg(Color::Rgb(255, 255, 255))).centered().render_ref(area_center, buf);
    }
}
