use super::Playlists;
use crate::actions::{Action, OnAction, OnActionMut, PlaylistsAction};

impl OnActionMut for Playlists<'_> {
    fn on_action(&mut self, actions: Vec<Action>) {
        match actions[..] {
            [Action::Playlists(PlaylistsAction::ShowHideGraveyard)] => {
                log::debug!("PlaylistsAction::ShowHideGraveyard");
                self.show_deleted_playlists = !self.show_deleted_playlists;
            }
            [Action::Playlists(PlaylistsAction::ViewToggleArtist)] => {
                self.selected_playlist_mut(|pl| pl.playlist_view.artist = !pl.playlist_view.artist);
            }
            [Action::Playlists(PlaylistsAction::ViewToggleAlbum)] => {
                self.selected_playlist_mut(|pl| pl.playlist_view.album = !pl.playlist_view.album);
            }
            [Action::Playlists(PlaylistsAction::ViewToggleYear)] => {
                self.selected_playlist_mut(|pl| pl.playlist_view.year = !pl.playlist_view.year);
            }
            [Action::Playlists(PlaylistsAction::ViewToggleTrackNumber)] => {
                self.selected_playlist_mut(|pl| pl.playlist_view.track_number = !pl.playlist_view.track_number);
            }
            _ => {
                self.focus_group.on_action(actions);
            }
        }
    }
}
