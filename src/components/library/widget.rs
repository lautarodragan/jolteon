use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    prelude::Widget,
    widgets::WidgetRef,
};

use super::Library;

impl Widget for Library<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        WidgetRef::render_ref(&self, area, buf);
    }
}

impl WidgetRef for Library<'_> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let [area_left, _, area_right] = Layout::horizontal([
            Constraint::Percentage(50),
            Constraint::Length(5),
            Constraint::Percentage(50),
        ])
        .horizontal_margin(2)
        .areas(area);

        self.album_tree.borrow().render_ref(area_left, buf);
        self.song_list.render_ref(area_right, buf);
    }
}
