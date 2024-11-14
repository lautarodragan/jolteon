use std::sync::atomic::Ordering;

use ratatui::{
    prelude::Widget,
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::{WidgetRef},
};

use super::component::List;

fn line_style(theme: &crate::config::Theme, index: usize, selected_index: usize, list_has_focus: bool) -> Style {
    if index == selected_index {
        if list_has_focus {
            Style::default().fg(theme.foreground_selected).bg(theme.background_selected)
        } else {
            Style::default().fg(theme.foreground_selected).bg(theme.background_selected_blur)
        }
    } else {
        Style::default().fg(theme.foreground_secondary).bg(theme.background)
    }
}

impl<'a, T> Widget for List<'a, T> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        WidgetRef::render_ref(&self, area, buf);
    }
}

impl<'a, T> WidgetRef for List<'a, T> {
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

            let style = line_style(&self.theme, item_index, selected_item_index, true);
            let line = ratatui::text::Line::from(
                // format!("{} - {} - {} - {}",
                //         song.year.as_ref().map(|y| y.to_string()).unwrap_or("(no year)".to_string()),
                //         song.album.clone().unwrap_or("(no album)".to_string()),
                //         song.track.unwrap_or(0),
                //         song.title.clone()
                // ),
                "some text"
            ).style(style);

            line.render_ref(area, buf);
        }
    }
}
