use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::ui::KeyboardHandlerRef;

use super::Playlists;

impl<'a> KeyboardHandlerRef<'a> for Playlists<'a> {

    fn on_key(&self, key: KeyEvent) {
        let mut focused_element_guard = self.focused_element.lock().unwrap();

        match key.code {
            KeyCode::Tab => {
                *focused_element_guard = match *focused_element_guard {
                    crate::components::playlists::playlists::PlaylistScreenElement::PlaylistList => crate::components::playlists::playlists::PlaylistScreenElement::SongList,
                    crate::components::playlists::playlists::PlaylistScreenElement::SongList => crate::components::playlists::playlists::PlaylistScreenElement::PlaylistList,
                };
            }
            _ if *focused_element_guard == crate::components::playlists::playlists::PlaylistScreenElement::PlaylistList  => {
                if key.code == KeyCode::Char('n') && key.modifiers == KeyModifiers::CONTROL {
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

// fn on_key_event_playlist_list(s: &Playlists, key: KeyEvent) {
//     let is_renaming = s.renaming.load(Ordering::Relaxed);
//
//     if !is_renaming {
//         match key.code {
//             KeyCode::Char('r') if key.modifiers == KeyModifiers::CONTROL => {
//                 s.renaming.store(true, Ordering::Relaxed);
//             }
//             _ => {},
//         }
//     } else {
//         match key.code {
//             KeyCode::Char(char) => {
//                 s.selected_playlist_mut(move |pl| {
//                     if pl.name.len() < 60 {
//                         pl.name.push(char);
//                     }
//                 });
//             }
//             KeyCode::Backspace => {
//                 s.selected_playlist_mut(move |pl| {
//                     if key.modifiers == KeyModifiers::ALT {
//                         pl.name.clear();
//                     } else {
//                         pl.name.pop();
//                     }
//                 });
//             }
//             KeyCode::Esc => {
//                 s.renaming.store(false, Ordering::Relaxed);
//             }
//             KeyCode::Enter => {
//                 s.renaming.store(false, Ordering::Relaxed);
//             }
//             _ => {},
//         }
//     }
// }
