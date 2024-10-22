use std::sync::atomic::Ordering;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::WidgetRef,
    text::Line,
};

use crate::config::Theme;

use super::component::{AlbumTree, AlbumTreeItem};

fn line_style(theme: &Theme, index: usize, selected_index: usize, list_has_focus: bool, item_is_filtered: bool) -> Style {
    if index == selected_index {
        if list_has_focus {
            Style::default().fg(theme.foreground_selected).bg(theme.background_selected)
        } else {
            Style::default().fg(theme.foreground_selected).bg(theme.background_selected_blur)
        }
    } else {
        let c = if item_is_filtered {
            theme.search
        } else {
            theme.foreground_secondary
        };
        Style::default().fg(c).bg(theme.background)
    }
}

impl<'a> WidgetRef for AlbumTree<'a> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        self.height.store(area.height as usize, Ordering::Relaxed);

        let item_list = &self.item_list.lock().unwrap();
        if item_list.len() < 1 {
            return;
        }

        let item_tree = self.item_tree.lock().unwrap();

        let selected_index = self.selected_index.load(Ordering::Relaxed);
        let offset = self.offset.load(Ordering::Relaxed);


        let mut i = 0;
        let mut y = 0;
        let area_height = area.height;

        let artist_rect = {
            let area = area.clone();

            move |y: u16| Rect {
                y: area.y + y,
                height: 1,
                ..area
            }
        };

        let album_rect = |y: u16|
            artist_rect(y).offset(ratatui::layout::Offset { x: 2, y: 0 });


        while i < item_list.len().min(area_height as usize) {
            let item_index = i + offset;

            if item_index >= item_list.len() {
                log::error!("item index {item_index} > item_list.len() {} offset={offset}", item_list.len());
                break;
            }

            let list_item = &item_list[item_index];

            let is_filtered = {
                // TODO: store this data in kb_handler and just read it here, in a Vec<bool>
                let filter = self.filter.lock().unwrap();
                !filter.is_empty() && list_item.contains(filter.as_str())
            };

            let style = line_style(&self.theme, item_index, selected_index, true, is_filtered);
            Line::from(list_item.to_string()).style(style).render_ref(artist_rect(y), buf);

            let AlbumTreeItem::Artist(_, is_open) = list_item else {
                continue;
            };

            if *is_open {
                let mut j = 0;
                let albums = item_tree.get(&list_item.to_string());

                if let Some(albums) = albums {
                    while j < albums.len() && y < area_height {
                        y += 1;

                        Line::from(albums[j].as_str()).render_ref(album_rect(y), buf);

                        j += 1;

                    }
                }
            }

            i += 1;
            y += 1;
        };
    }
}
