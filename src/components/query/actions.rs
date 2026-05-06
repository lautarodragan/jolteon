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
                        if let Some(query) = self.query.as_mut() {
                            if let Query::AddSongs { step, .. } = query
                                && *step > 0
                            {
                                *step -= 1;
                                return;
                            }
                        }
                        self.query = None;
                        return;
                    }
                    Action::Confirm => {
                        let Some(query) = self.query.take() else {
                            continue;
                        };
                        match query {
                            Query::AddSongs {
                                songs,
                                step,
                                target,
                                target_name,
                                playlists,
                            } => match target {
                                QueryAddSongsTarget::Library => {
                                    self.on_confirm_fn.call(Query::AddSongs {
                                        songs,
                                        step,
                                        target,
                                        target_name,
                                        playlists,
                                    });
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
                                            step,
                                            target: QueryAddSongsTarget::Soundtracks,
                                            target_name: target_name.clone(),
                                            playlists: playlists.clone(),
                                        });
                                    }
                                    if !songs_soundtracks.is_empty() {
                                        self.on_confirm_fn.call(Query::AddSongs {
                                            songs: songs_soundtracks,
                                            step,
                                            target,
                                            target_name,
                                            playlists,
                                        });
                                    }
                                }
                                QueryAddSongsTarget::Playlist => {
                                    if step == 0 {
                                        self.query = Some(Query::AddSongs {
                                            songs,
                                            step: step + 1,
                                            target,
                                            target_name,
                                            playlists,
                                        });
                                    } else if step == 1 {
                                        self.on_confirm_fn.call(Query::AddSongs {
                                            songs,
                                            step,
                                            target,
                                            target_name,
                                            playlists,
                                        });
                                    }
                                }
                            },
                        }
                        return;
                    }
                    Action::Navigation(NavigationAction::Right) => {
                        if let Some(Query::AddSongs {
                            step,
                            target,
                            target_name,
                            playlists,
                            ..
                        }) = self.query.as_mut()
                        {
                            if *step == 0 {
                                *target = target.next();
                            } else if *step == 1 && *target == QueryAddSongsTarget::Playlist {
                                let tn = target_name.clone().unwrap_or_default();
                                let i = playlists.iter().position(|pl| *pl == tn).unwrap();
                                let i = if i + 1 < playlists.len() {
                                    i + 1
                                } else {
                                    playlists.len() - 1
                                };
                                let tn = playlists.get(i).unwrap().clone();
                                *target_name = Some(tn);
                            }
                        }
                        return;
                    }
                    Action::Navigation(NavigationAction::Left) => {
                        if let Some(Query::AddSongs {
                            step,
                            target,
                            target_name,
                            playlists,
                            ..
                        }) = self.query.as_mut()
                        {
                            if *step == 0 {
                                *target = target.prev();
                            } else if *step == 1 && *target == QueryAddSongsTarget::Playlist {
                                let tn = target_name.clone().unwrap_or_default();
                                let i = playlists.iter().position(|pl| *pl == tn).unwrap();
                                let i = if i > 0 { i - 1 } else { 0 };
                                let tn = playlists.get(i).unwrap().clone();
                                *target_name = Some(tn);
                            }
                        }
                        return;
                    }
                    _ => {}
                }
            }
        }
    }
}
