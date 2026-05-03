use super::Playlists;
use crate::actions::{Action, NavigationAction, OnAction, OnActionMut, PlaylistsAction};

impl OnActionMut for Playlists<'_> {
    fn on_action(&mut self, actions: Vec<Action>) {
        match actions[..] {
            [Action::Navigation(NavigationAction::Right)] => {
                // TODO: either prioritize self.focus_group.on_action(actions) or respect focus_stolen
                self.focus_group.focus_nth(1);
            }
            [Action::Navigation(NavigationAction::Left)] => {
                // TODO: either prioritize self.focus_group.on_action(actions) or respect focus_stolen,
                self.focus_group.focus_nth(0);
            }
            [Action::Playlists(playlist_action)] => match playlist_action {
                PlaylistsAction::ShowHideGraveyard => {
                    log::debug!("PlaylistsAction::ShowHideGraveyard");
                    self.show_deleted_playlists = !self.show_deleted_playlists;
                }
                PlaylistsAction::ViewToggleArtist
                | PlaylistsAction::ViewToggleAlbum
                | PlaylistsAction::ViewToggleYear
                | PlaylistsAction::ViewToggleTrackNumber => {
                    self.selected_playlist_mut(|pl| {
                        let value = match playlist_action {
                            PlaylistsAction::ViewToggleArtist => &mut pl.view_options.artist,
                            PlaylistsAction::ViewToggleAlbum => &mut pl.view_options.album,
                            PlaylistsAction::ViewToggleYear => &mut pl.view_options.year,
                            PlaylistsAction::ViewToggleTrackNumber => &mut pl.view_options.track_number,
                            _ => {
                                unreachable!();
                            }
                        };
                        *value = !*value;
                        self.song_list.set_view_options(pl.view_options);
                    });
                }
            },

            _ => {
                self.focus_group.on_action(actions);
            }
        }
    }
}
