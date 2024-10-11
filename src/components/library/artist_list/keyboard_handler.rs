use std::sync::atomic::Ordering;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{
    ui::KeyboardHandlerRef,
};

use super::artist_list::ArtistList;

impl<'a> KeyboardHandlerRef<'a> for ArtistList<'a> {

    fn on_key(&self, key: KeyEvent) -> bool {
        let target = "::ArtistList.on_key";
        log::trace!(target: target, "{:?}", key);

        match key.code {
            KeyCode::Up | KeyCode::Down | KeyCode::Home | KeyCode::End => {
                self.on_artist_list_directional_key(key);
                let artist = self.selected_artist();
                self.on_select_fn.lock().unwrap()(artist);
            },
            KeyCode::Enter => {
                let artist = self.selected_artist();
                self.on_confirm_fn.lock().unwrap()(artist);
            },
            KeyCode::Delete => {
                let mut artists = self.artists.lock().unwrap();
                let i = self.selected_index.load(Ordering::SeqCst);

                let removed_artist = artists.remove(i);

                self.on_delete_fn.lock().unwrap()(removed_artist);

                let i = i.saturating_sub(1).min(artists.len().saturating_sub(1));

                self.selected_index.store(i, Ordering::SeqCst);
                self.offset.store(0, Ordering::SeqCst); // TODO: fix me

                let artist = artists[i].clone();
                *self.selected_artist.lock().unwrap() = artist.clone();

                drop(artists);

                self.on_select_fn.lock().unwrap()(artist);
            },
            KeyCode::Char(char) => {
                let artists = self.artists.lock().unwrap();
                let mut filter = self.filter.lock().unwrap();

                filter.push(char);
                let filter_low = filter.to_lowercase().to_string();

                let Some(i) = artists.iter().position(|artist|
                    artist.contains(filter.as_str()) ||
                        artist.to_lowercase().contains(filter_low.as_str())
                ) else {
                    return false;
                };

                self.selected_index.store(i, Ordering::SeqCst);
                let artist = artists[i].clone();

                drop(artists);
                drop(filter);

                self.on_select_fn.lock().unwrap()(artist);
            }
            KeyCode::Esc => {
                let mut filter = self.filter.lock().unwrap();
                filter.clear();
            }
            _ => {},
        }

        true
    }
}


impl<'a> ArtistList<'a> {

    fn on_artist_list_directional_key(&self, key: KeyEvent) {
        let artists = self.artists.lock().unwrap();
        let length = artists.len() as i32;

        let height = self.height.load(Ordering::Relaxed) as i32;
        let padding = 5;

        let mut offset = self.offset.load(Ordering::SeqCst) as i32;
        let mut i = self.selected_index.load(Ordering::SeqCst) as i32;

        match key.code {
            KeyCode::Up | KeyCode::Down => {
                if key.code == KeyCode::Up {
                    i -= 1;
                } else {
                    i += 1;
                }

                let padding = if key.code == KeyCode::Up {
                    padding
                } else {
                    height.saturating_sub(padding).saturating_sub(1)
                };

                if (key.code == KeyCode::Up && i < offset + padding) || (key.code == KeyCode::Down && i > offset + padding) {
                    offset = if i > padding {
                        i - padding
                    } else {
                        0
                    };
                }

            },
            KeyCode::Home => {
                i = 0;
                offset = 0;
            },
            KeyCode::End => {
                i = length - 1;
                offset = i - height + padding;
            },
            _ => {},
        }

        offset = offset.min(length - height).max(0);
        i = i.min(length - 1).max(0);

        self.offset.store(offset as usize, Ordering::SeqCst);
        self.selected_index.store(i as usize, Ordering::SeqCst);

        let selected_artist = artists[i as usize].clone();
        *self.selected_artist.lock().unwrap() = selected_artist;
    }

}
