use crate::{
    actions::{Action, NavigationAction, OnActionMut},
    components::query::{CommandLine, Query, QueryAddSongsTarget},
};

impl OnActionMut for CommandLine<'_> {
    fn on_action(&mut self, actions: Vec<Action>) {
        if self.query_error.is_some() {
            for action in actions {
                match action {
                    Action::Confirm | Action::Cancel => {
                        self.query_error = None;
                        return;
                    }
                    _ => {}
                }
            }
        } else {
            for action in actions {
                match action {
                    Action::Cancel => {
                        self.query = None;
                        return;
                    }
                    Action::Confirm => {
                        let Some(query) = self.query.take() else {
                            continue;
                        };
                        match query {
                            Query::AddSongs { songs, target } => match target {
                                QueryAddSongsTarget::Library => {
                                    self.on_confirm_fn.call(Query::AddSongs { songs, target });
                                }
                                QueryAddSongsTarget::Soundtracks => {
                                    let (songs_soundtracks, songs_etc): (Vec<_>, Vec<_>) =
                                        songs.into_iter().partition(|song| song.soundtrack_subject.is_some());

                                    if !songs_etc.is_empty() {
                                        // TODO: ask for soundtrack_subject
                                        self.query_error = Some(format!(
                                            "{} song(s) could not be added to soundtracks!",
                                            songs_etc.len()
                                        ));
                                        self.query = Some(Query::AddSongs {
                                            songs: songs_etc,
                                            target: QueryAddSongsTarget::Soundtracks,
                                        });
                                    }
                                    if !songs_soundtracks.is_empty() {
                                        self.on_confirm_fn.call(Query::AddSongs {
                                            songs: songs_soundtracks,
                                            target,
                                        });
                                    }
                                }
                                QueryAddSongsTarget::Playlist => {
                                    self.on_confirm_fn.call(Query::AddSongs { songs, target });
                                }
                            },
                        }
                        return;
                    }
                    Action::Navigation(NavigationAction::Right) => {
                        if let Some(Query::AddSongs { target, .. }) = self.query.as_mut() {
                            *target = target.next();
                        }
                        return;
                    }
                    Action::Navigation(NavigationAction::Left) => {
                        if let Some(Query::AddSongs { target, .. }) = self.query.as_mut() {
                            *target = target.prev();
                        }
                        return;
                    }
                    _ => {}
                }
            }
        }
    }
}
