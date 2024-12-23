use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::widgets::WidgetRef;

use super::Queue;

impl WidgetRef for Queue<'_> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let [area] = Layout::horizontal([Constraint::Percentage(100)])
            .horizontal_margin(2)
            .areas(area);

        self.song_list.render_ref(area, buf);
    }
}
