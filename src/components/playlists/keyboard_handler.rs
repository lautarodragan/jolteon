use super::Playlists;

use crate::structs::{Action, OnAction, OnActionMut, PlaylistsAction};

impl OnActionMut for Playlists<'_> {
    fn on_action(&mut self, action: Action) {
        match action {
            Action::Playlists(PlaylistsAction::ShowHideGraveyard) => {
                log::debug!("PlaylistsAction::ShowHideGraveyard");
                self.show_deleted_playlists = !self.show_deleted_playlists;
            }
            _ => {
                self.focus_group.on_action(action);
            }
        }
    }
}
