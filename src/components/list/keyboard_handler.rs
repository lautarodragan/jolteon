use std::sync::atomic::Ordering;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{ui::KeyboardHandlerRef};

use super::component::List;

impl<'a, T: 'a + Clone> KeyboardHandlerRef<'a> for List<'a, T> {

    fn on_key(&self, key: KeyEvent) {
        let target = "::List.on_key";
        log::trace!(target: target, "{:?}", key);

        match key.code {
            KeyCode::Up | KeyCode::Down | KeyCode::Home | KeyCode::End => {
                self.on_directional_key(key);
            },
            KeyCode::Enter | KeyCode::Char(_) => {
                let items = self.items.lock().unwrap();

                let i = self.selected_item_index.load(Ordering::SeqCst);
                if i >= items.len() {
                    log::error!(target: target, "selected_item_index > items.len");
                    return;
                }
                let item = items[i].clone();
                drop(items);
                self.on_select_fn.lock().unwrap()(item, key);
            },
            // KeyCode::Delete => {
            //     let selected_song = self.selected_item_index.load(Ordering::Relaxed);
            //     self.selected_playlist_mut(|pl| {
            //         if pl.len() > 0 {
            //             pl.songself.remove(selected_song);
            //             if selected_song >= pl.songself.len() {
            //                 self.selected_item_index.store(selected_song.saturating_sub(1), Ordering::Relaxed);
            //             }
            //         }
            //     });
            // },
            _ => {},
        }
    }
}


impl<'a, T> List<'a, T> {

    fn on_directional_key(&self, key: KeyEvent) {
        let items = self.items.lock().unwrap();
        let length = items.len() as i32;

        let height = self.height.load(Ordering::Relaxed) as i32;
        let padding = 5;

        let mut offset = self.offset.load(Ordering::SeqCst) as i32;
        let mut i = self.selected_item_index.load(Ordering::SeqCst) as i32;

        match key.code {
            KeyCode::Up | KeyCode::Down => {
                if key.modifiers == KeyModifiers::NONE {
                    if key.code == KeyCode::Up {
                        i -= 1;
                    } else {
                        i += 1;
                    }
                // } else if key.modifiers == KeyModifiers::ALT {
                //     if let Some(next) = next_index_by_album(&*items, i, key.code) {
                //         i = next as i32;
                //     }
                } else {
                    return;
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

            // KeyCode::Up if key.modifiers == KeyModifiers::ALT => {
            //     let selected_song = s.selected_song_index.load(Ordering::Relaxed);
            //     s.selected_playlist_mut(|pl| {
            //         if pl.songs.len() > 1 && selected_song > 0 {
            //             pl.songs.swap(selected_song, selected_song - 1);
            //             s.selected_song_index.store(selected_song - 1, Ordering::Relaxed);
            //         }
            //     });
            // },
            // KeyCode::Down if key.modifiers == KeyModifiers::ALT => {
            //     let selected_song = s.selected_song_index.load(Ordering::Relaxed);
            //     s.selected_playlist_mut(|pl| {
            //         if pl.songs.len() > 1 && selected_song < pl.songs.len() - 1 {
            //             pl.songs.swap(selected_song, selected_song + 1);
            //             s.selected_song_index.store(selected_song + 1, Ordering::Relaxed);
            //         }
            //     });
            // },
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
        self.selected_item_index.store(i as usize, Ordering::SeqCst);
    }

}
