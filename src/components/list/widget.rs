use std::sync::atomic::Ordering;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::{WidgetRef},
};

use super::component::List;

fn line_style(theme: &crate::config::Theme, list_has_focus: bool, is_selected: bool, is_search_match: bool) -> Style {
    if is_selected {
        if list_has_focus {
            Style::default().fg(theme.foreground_selected).bg(theme.background_selected)
        } else {
            Style::default().fg(theme.foreground_selected).bg(theme.background_selected_blur)
        }
    } else {
        let c = if is_search_match {
            theme.search
        } else {
            theme.foreground_secondary
        };
        Style::default().fg(c).bg(theme.background)
    }
}

impl<'a, T> WidgetRef for List<'a, T>
where T: std::fmt::Display,
{
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        self.height.store(area.height as usize, Ordering::Relaxed);

        let items = &self.items.lock().unwrap();

        if items.len() < 1 {
            return;
        }

        let selected_item_index = self.selected_item_index.load(Ordering::Relaxed);
        let offset = self.offset.load(Ordering::Relaxed);

        for i in 0..items.len().min(area.height as usize) {
            let item_index = i + offset;

            if item_index >= items.len() {
                log::error!("item index {item_index} > items.len() {} offset={offset}", items.len());
                break;
            }

            let item = &items[item_index];
            let area = Rect {
                y: area.y + i as u16,
                height: 1,
                ..area
            };

            let style = line_style(&self.theme, true, item_index == selected_item_index, item.is_match);
            let line = ratatui::text::Line::from(item.inner.to_string()).style(style);

            line.render_ref(area, buf);
        }
    }
}
