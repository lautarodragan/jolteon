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

        let selected_index = self.selected_artist.load(Ordering::Relaxed);
        let selected_album_index = self.selected_album.load(Ordering::Relaxed);
        let offset = self.offset.load(Ordering::Relaxed);

        let mut i_artist = 0;
        let area_height = area.height;
        let area_bottom = area.height.saturating_sub(area.y);

        let mut rect = Rect {
            y: area.y,
            height: 1,
            ..area
        };

        while i_artist < item_list.len().min(area_height as usize) && rect.y < area_bottom {
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
            Line::from(artist.data.to_string()).style(style).render_ref(rect, buf);

            if artist.is_open {
                rect.x += 2;
                rect.width -= 2;

                let mut i_album = 0;
                while i_album < artist.albums.len() && rect.y < area_bottom {
                    rect.y += 1;

                    let style = line_style(&self.theme, true, item_index == selected_index && selected_album_index == i_album, is_filter_match);
                    Line::from(artist.albums[i_album].as_str()).style(style).render_ref(rect, buf);

                    i_album += 1;

                }
                rect.x -= 2;
                rect.width += 2;
            }

            i_artist += 1;
            rect.y += 1;
        };
    }
}
