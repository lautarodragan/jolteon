use crossterm::event::{KeyCode, KeyEvent};

use crate::{components::playlists::playlists::PlaylistScreenElement, ui::KeyboardHandlerRef};
use crate::structs::{Action, NavigationAction, OnAction};
use super::Playlists;

impl<'a> KeyboardHandlerRef<'a> for Playlists<'a> {
    fn on_key(&self, key: KeyEvent) {
        match key.code {
            KeyCode::F(8) => {
                self.show_deleted_playlists.set(!self.show_deleted_playlists.get());
            }
            _ => {}
        }
    }
}

impl OnAction for Playlists<'_> {
    fn on_action(&self, action: Action) {
        let mut focused_element_guard = self.focused_element.lock().unwrap();

        match action {
            Action::Navigation(NavigationAction::FocusNext) => {
                *focused_element_guard = match *focused_element_guard {
                    PlaylistScreenElement::PlaylistList => PlaylistScreenElement::SongList,
                    PlaylistScreenElement::SongList => PlaylistScreenElement::PlaylistList,
                };
            }
            _ if *focused_element_guard == PlaylistScreenElement::PlaylistList => {
                self.playlist_list.on_action(action);
            }
            _ if *focused_element_guard == PlaylistScreenElement::SongList => {
                self.song_list.on_action(action);
            }
            _ => {}
        }
    }
}
