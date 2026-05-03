use super::Root;
use crate::{
    actions::{Action, NavigationAction, OnActionMut, ScreenAction},
    components::{Query, QueryAddSongsArg1},
};

impl OnActionMut for Root<'_> {
    fn on_action(&mut self, action: Vec<Action>) {
        if self.query.borrow().is_some() {
            let mut query_error = self.query_error.borrow_mut();
            if query_error.is_some() {
                match action[0] {
                    Action::Confirm | Action::Cancel => {
                        *query_error = None;
                    }
                    _ => {}
                }
            } else {
                match action[0] {
                    Action::Cancel => {
                        *self.query.borrow_mut() = None;
                    }
                    Action::Confirm => {
                        let query = self.query.borrow_mut().take();
                        match query {
                            None => {}
                            Some(Query::AddSongs { songs, target }) => match target {
                                QueryAddSongsArg1::Library => {
                                    self.library_screen.borrow_mut().add_songs(songs);
                                }
                                QueryAddSongsArg1::Soundtracks => {
                                    let (songs_soundtracks, songs_etc): (Vec<_>, Vec<_>) =
                                        songs.into_iter().partition(|song| song.soundtrack_subject.is_some());

                                    if !songs_soundtracks.is_empty() {
                                        self.soundtracks_screen.borrow_mut().add_songs(songs_soundtracks);
                                    }
                                    if !songs_etc.is_empty() {
                                        *query_error = Some(format!(
                                            "{} song(s) could not be added to soundtracks!",
                                            songs_etc.len()
                                        ));
                                        *self.query.borrow_mut() = Some(Query::AddSongs {
                                            songs: songs_etc,
                                            target: QueryAddSongsArg1::Soundtracks,
                                        });
                                    }
                                }
                                QueryAddSongsArg1::Playlist => {
                                    self.playlists_screen.borrow_mut().add_songs(songs);
                                }
                            },
                        }
                    }
                    Action::Navigation(NavigationAction::Right) => {
                        if let Some(Query::AddSongs { target, .. }) = self.query.borrow_mut().as_mut() {
                            *target = target.next();
                        }
                    }
                    Action::Navigation(NavigationAction::Left) => {
                        if let Some(Query::AddSongs { target, .. }) = self.query.borrow_mut().as_mut() {
                            *target = target.prev();
                        }
                    }
                    _ => {
                        // TODO: do something with the query
                    }
                }
            }
        } else {
            match action[0] {
                Action::Screen(action) if !self.is_focus_trapped.get() => match action {
                    ScreenAction::Next => {
                        if self.focused_screen < 5 {
                            self.focused_screen += 1;
                        } else {
                            self.focused_screen = 0;
                        }
                    }
                    ScreenAction::Previous => {
                        if self.focused_screen > 0 {
                            self.focused_screen -= 1;
                        } else {
                            self.focused_screen = 5;
                        }
                    }
                    ScreenAction::Library => self.focused_screen = 0,
                    ScreenAction::Soundtracks => self.focused_screen = 1,
                    ScreenAction::Playlists => self.focused_screen = 2,
                    ScreenAction::Queue => self.focused_screen = 3,
                    ScreenAction::FileBrowser => self.focused_screen = 4,
                    ScreenAction::Help => self.focused_screen = 5,
                },
                _ => {
                    let mut c = self.screens[self.focused_screen].1.borrow_mut();
                    c.on_action(action);
                }
            }
        }
    }
}
