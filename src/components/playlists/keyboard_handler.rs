use crossterm::event::{KeyCode, KeyEvent};

use crate::ui::KeyboardHandlerRef;

use super::Playlists;

impl<'a> KeyboardHandlerRef<'a> for Playlists<'a> {

    fn on_key(&self, key: KeyEvent) {
        let mut focused_element_guard = self.focused_element.lock().unwrap();

        match key.code {
            KeyCode::F(8) => {
                self.show_deleted_playlists.set(!self.show_deleted_playlists.get());
            }
            KeyCode::Tab => {
                *focused_element_guard = match *focused_element_guard {
                    crate::components::playlists::playlists::PlaylistScreenElement::PlaylistList => crate::components::playlists::playlists::PlaylistScreenElement::SongList,
                    crate::components::playlists::playlists::PlaylistScreenElement::SongList => crate::components::playlists::playlists::PlaylistScreenElement::PlaylistList,
                };
            }
            _ if *focused_element_guard == crate::components::playlists::playlists::PlaylistScreenElement::PlaylistList  => {
                if key.code == KeyCode::Insert {
                    self.create_playlist();
                    return;
                }
                let Ok(playlists) = self.playlist_list.try_borrow() else {
                    return;
                };
                playlists.on_key(key);
            },
            _ if *focused_element_guard == crate::components::playlists::playlists::PlaylistScreenElement::SongList  => {
                let Ok(song_list) = self.song_list.try_borrow() else {
                    return;
                };
                song_list.on_key(key);
            },
            _ => {},
        }
    }

}
