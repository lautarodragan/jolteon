use std::sync::atomic::Ordering;

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
                on_key_event_playlist_list(&self, key);

                // overkill but ok for now: we clone the song list whenever any key is pressed, no matter what happened.
                let selected_playlist_index = self.selected_playlist_index.load(Ordering::Relaxed);
                let playlists = self.playlists.lock().unwrap();
                let Some(selected_playlist) = playlists.get(selected_playlist_index) else {
                    return;
                };

                self.song_list.set_items(selected_playlist.songs.clone());
            },
            _ if *focused_element_guard == crate::components::playlists::playlists::PlaylistScreenElement::SongList  => {
                self.song_list.on_key(key);
            },
            _ => {},
        }
    }

}

fn on_key_event_playlist_list(s: &Playlists, key: KeyEvent) {
    let len = s.playlists.lock().unwrap().len();
    let is_renaming = s.renaming.load(Ordering::Relaxed);

    if !is_renaming {
        match key.code {
            KeyCode::Up => {
                let _ = s.selected_playlist_index.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |a| { Some(a.saturating_sub(1)) });
            },
            KeyCode::Down => {
                let _ = s.selected_playlist_index.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |a| { Some(a.saturating_add(1).min(len.saturating_sub(1))) });
            },
            KeyCode::Home => {
                s.selected_playlist_index.store(0, Ordering::Relaxed);
            },
            KeyCode::End => {
                s.selected_playlist_index.store(len.saturating_sub(1), Ordering::Relaxed);
            },
            KeyCode::Char('n') if key.modifiers == KeyModifiers::CONTROL => {
                s.create_playlist();
                let _ = s.selected_playlist_index.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |a| { Some(a.saturating_add(1).min(len)) });
            }
            KeyCode::Char('r') if key.modifiers == KeyModifiers::CONTROL => {
                s.renaming.store(true, Ordering::Relaxed);
            }
            KeyCode::Delete => {
                let selected_playlist_index = s.selected_playlist_index.load(Ordering::Relaxed);
                let mut playlists = s.playlists.lock().unwrap();

                if playlists.len() > 0 {
                    playlists.remove(selected_playlist_index);
                    if selected_playlist_index > playlists.len().saturating_sub(1) {
                        s.selected_playlist_index.store(playlists.len().saturating_sub(1), Ordering::Relaxed);
                    }
                }
            }
            KeyCode::Enter => {
                let selected_playlist_index = s.selected_playlist_index.load(Ordering::Relaxed);
                let playlists = s.playlists.lock().unwrap();
                let Some(selected_playlist) = playlists.get(selected_playlist_index) else {
                    log::error!(target: "::playlist", "on_key_event_playlist_list(Enter) error. No selected playlist at selected playlist index!");
                    return;
                };

                let songs = selected_playlist.songs.clone();

                let mut cb = s.on_select_playlist_fn.lock().unwrap();
                cb(songs, key);
            }
            _ => {},
        }
    } else {
        match key.code {
            KeyCode::Char(char) => {
                s.selected_playlist_mut(move |pl| {
                    if pl.name.len() < 60 {
                        pl.name.push(char);
                    }
                });
            }
            KeyCode::Backspace => {
                s.selected_playlist_mut(move |pl| {
                    if key.modifiers == KeyModifiers::ALT {
                        pl.name.clear();
                    } else {
                        pl.name.pop();
                    }
                });
            }
            KeyCode::Esc => {
                s.renaming.store(false, Ordering::Relaxed);
            }
            KeyCode::Enter => {
                s.renaming.store(false, Ordering::Relaxed);
            }
            _ => {},
        }
    }
}
