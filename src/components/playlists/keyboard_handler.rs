use super::Playlists;

use crate::{components::playlists::playlists::PlaylistScreenElement, structs::{Action, NavigationAction, OnAction, PlaylistsAction}};

impl OnAction for Playlists<'_> {
    fn on_action(&self, action: Action) {
        let mut focused_element_guard = self.focused_element.lock().unwrap();

        match action {
            Action::Playlists(PlaylistsAction::ShowHideGraveyard) => {
                log::debug!("PlaylistsAction::ShowHideGraveyard");
                self.show_deleted_playlists.set(!self.show_deleted_playlists.get());
            }
            Action::Navigation(NavigationAction::FocusNext | NavigationAction::FocusPrevious) => {
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
