use std::sync::atomic::Ordering;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::WidgetRef,
};

use super::artist_list::ArtistList;

fn line_style(theme: &crate::config::Theme, index: usize, selected_index: usize, list_has_focus: bool, item_is_filtered: bool) -> Style {
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

impl<'a> WidgetRef for ArtistList<'a> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        self.height.store(area.height as usize, Ordering::Relaxed);

        let artists = &self.artists.lock().unwrap();

        if artists.len() < 1 {
            return;
        }

        let selected_index = self.selected_index.load(Ordering::Relaxed);
        let offset = self.offset.load(Ordering::Relaxed);

        for i in 0..artists.len().min(area.height as usize) {
            let artist_index = i + offset;

            if artist_index >= artists.len() {
                log::error!("artist index {artist_index} > artists.len() {} offset={offset}", artists.len());
                break;
            }

            let artist = artists[artist_index].clone();
            let area = Rect {
                y: area.y + i as u16,
                height: 1,
                ..area
            };

            let is_filtered = {
                // TODO: store this data in kb_handler and just read it here, in a Vec<bool>
                let filter = self.filter.lock().unwrap();

                if filter.is_empty() {
                    false
                } else {
                    let filter_low = filter.to_lowercase().to_string();
                    artist.contains(filter.as_str()) || artist.to_lowercase().contains(filter_low.as_str())
                }
            };

            let style = line_style(&self.theme, artist_index, selected_index, true, is_filtered);
            let line = ratatui::text::Line::from(artist).style(style);

            line.render_ref(area, buf);
        }
    }
}
