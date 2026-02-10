use ratatui::{buffer::Buffer, layout::Rect, widgets::WidgetRef};

use super::SongList;

impl WidgetRef for SongList<'_> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        self.list.render_ref(area, buf);
    }
}
