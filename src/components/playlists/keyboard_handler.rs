use super::Playlists;

use crate::structs::{Action, OnAction, PlaylistsAction};

impl OnAction for Playlists<'_> {
    fn on_action(&self, action: Action) {
        match action {
            Action::Playlists(PlaylistsAction::ShowHideGraveyard) => {
                log::debug!("PlaylistsAction::ShowHideGraveyard");
                self.show_deleted_playlists.set(!self.show_deleted_playlists.get());
            }
            _ => {
                self.focus_group.on_action(action);
            }
        }
    }
}
