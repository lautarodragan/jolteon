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

fn line_style(theme: &Theme, list_has_focus: bool, is_selected: bool, is_search_match: bool) -> Style {
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

impl<'a> WidgetRef for AlbumTree<'a> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        self.height.store(area.height as usize, Ordering::Relaxed);

        let item_list = &self.artist_list.lock().unwrap();
        if item_list.len() < 1 {
            return;
        }

        let item_tree = self.item_tree.lock().unwrap();

        let selected_index = self.selected_artist.load(Ordering::Relaxed);
        let selected_album_index = self.selected_album.load(Ordering::Relaxed);
        let offset = self.offset.load(Ordering::Relaxed);

        let mut i_artist = 0;
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

        while i_artist < item_list.len().min(area_height as usize) {
            let item_index = i_artist + offset;

            if item_index >= item_list.len() {
                log::error!("item index {item_index} > item_list.len() {} offset={offset}", item_list.len());
                break;
            }

            let artist = &item_list[item_index];

            let is_filter_match = {
                // TODO: store this data in kb_handler and just read it here, in a Vec<bool>
                let filter = self.filter.lock().unwrap();
                !filter.is_empty() && artist.data.contains(filter.as_str())
            };


            let style = line_style(&self.theme, true, item_index == selected_index, is_filter_match);
            Line::from(artist.data.to_string()).style(style).render_ref(artist_rect(y), buf);

            if artist.is_open {
                let mut i_album = 0;
                let albums = item_tree.get(&artist.data);

                if let Some(albums) = albums {
                    while i_album < albums.len() && y < area_height {
                        y += 1;

                        let style = line_style(&self.theme, true, item_index == selected_index && selected_album_index == i_album, is_filter_match);
                        Line::from(albums[i_album].as_str()).style(style).render_ref(album_rect(y), buf);

                        i_album += 1;

                    }
                }
            }

            i_artist += 1;
            y += 1;
        };
    }
}
