use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::prelude::{Modifier, Style};
use ratatui::widgets::{List, ListState, StatefulWidget, WidgetRef};

use crate::ui;

use super::queue::Queue;

impl WidgetRef for Queue {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let [area] = Layout::horizontal([Constraint::Percentage(100)])
            .horizontal_margin(2)
            .areas(area);

        let queue_items: Vec<String> = self.songs().iter().map(ui::song_to_string).collect();

        let queue_list = List::new(queue_items)
            .style(Style::default().fg(self.theme.foreground))
            .highlight_style(
                Style::default()
                    .bg(self.theme.background_selected)
                    .fg(self.theme.foreground_selected)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("");

        StatefulWidget::render(
            queue_list,
            area,
            buf,
            &mut ListState::default().with_selected(Some(self.selected_song_index())),
        );
    }
}
