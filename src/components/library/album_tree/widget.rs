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

        let artist_list = self.artist_list.lock().unwrap();

        if artist_list.is_empty() {
            return;
        }

        let selected_index = self.selected_artist.load(Ordering::Relaxed);
        let selected_album_index = self.selected_album.load(Ordering::Relaxed);
        let offset = self.offset.load(Ordering::Relaxed);

        let mut list = vec![];

        for i in 0..artist_list.len() {
            let artist = &artist_list[i];

            list.push((artist.artist.as_str(), false, i == selected_index));

            if artist.is_open {
                for j in 0..artist.albums.len() {
                    let album = artist.albums[j].as_str();
                    list.push((album, true, i == selected_index && j == selected_album_index));
                }
            }
        }

        if offset >= list.len() {
            log::error!("offset >= list.len() offset={offset} item_list.len() {} ", list.len());
            return;
        }

        for i in offset..list.len().min(offset + area.height as usize) {
            let (text, is_album, is_selected) = list[i];

            let style = line_style(&self.theme, true, is_selected, false);
            let rect = Rect {
                x: if is_album{
                    area.x + 2
                } else {
                    area.x
                },
                y: area.y + i as u16 - offset as u16,
                width: area.width,
                height: 1,
            };
            Line::from(text).style(style).render_ref(rect, buf);
        }
    }
}
