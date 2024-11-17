use std::{
    sync::Mutex,
    rc::Rc,
    cell::RefCell,
};
use chrono::Local;
use crossterm::event::{KeyEvent};

use crate::{
    structs::{Song, Playlist},
    config::Theme,
    cue::CueSheet,
    components::List,
};

#[derive(Eq, PartialEq)]
pub(super) enum PlaylistScreenElement {
    PlaylistList,
    SongList,
}

pub struct Playlists<'a> {
    pub(super) playlist_list: Rc<RefCell<List<'a, Playlist>>>,
    pub(super) song_list: Rc<RefCell<List<'a, Song>>>,
    pub(super) focused_element: Mutex<PlaylistScreenElement>,
}

impl<'a> Playlists<'a> {
    pub fn new(theme: Theme, playlists: Vec<Playlist>) -> Self {
        let song_list = Rc::new(RefCell::new(List::new(theme, playlists.get(0).map(|pl| pl.songs.clone()).unwrap_or(vec![]))));
        let playlist_list = Rc::new(RefCell::new(List::new(theme, playlists)));

        playlist_list.borrow_mut().on_select({
            let song_list = song_list.clone();
            move |pl, _| {
                let Ok(song_list) = song_list.try_borrow_mut() else {
                    return;
                };

                song_list.set_items(pl.songs.clone());
            }
        });

        playlist_list.borrow_mut().on_rename({
            |pl, new_name| pl.name = new_name.to_string()
        });

        song_list.borrow_mut().on_reorder({
            let playlists = playlist_list.clone();

            move |a, b| {
                log::debug!(target: "::playlists", "on_reorder {a} {b}");

                let Ok(pls) = playlists.try_borrow_mut() else {
                    return;
                };

                pls.with_selected_item_mut(move |pl| {
                    pl.songs.swap(a, b);
                });
            }
        });
        song_list.borrow_mut().on_delete({
            let playlists = playlist_list.clone();

            move |song, index| {
                log::trace!(target: "::playlists", "on_delete {index} {}", song.title);

                let Ok(pls) = playlists.try_borrow_mut() else {
                    return;
                };

                pls.with_selected_item_mut(move |pl| {
                    pl.songs.remove(index);
                });
            }
        });

        Self {
            // playlists: Mutex::new(vec![
            //     Playlist::new("My first Jolteon playlist".to_string()),
            //     Playlist::new("Ctrl+N to create new ones".to_string()),
            //     Playlist::new("Alt+N to rename".to_string()),
            // ]),
            playlist_list,
            song_list,
            focused_element: Mutex::new(PlaylistScreenElement::PlaylistList),
        }
    }

    pub fn on_enter_song(&self, cb: impl FnMut(Song, KeyEvent) + 'a) {
        let Ok(song_list) = self.song_list.try_borrow_mut() else {
            log::error!("try_borrow_mut failed");
            return;
        };
        song_list.on_select(cb);
    }

    pub fn on_enter_playlist(&self, cb: impl Fn(Playlist) + 'a) {
        let Ok(playlists) = self.playlist_list.try_borrow_mut() else {
            log::error!("playlists.try_borrow_mut() failed...");
            return;
        };
        playlists.on_enter(cb);
    }

    pub fn playlists(&self) -> Vec<Playlist> {
        let Ok(playlists) = self.playlist_list.try_borrow() else {
            log::error!("Could not borrow playlists");
            return vec![];
        };

        playlists.with_items(|pl| {
            pl.clone().iter().map(|x| (*x).clone()).map(|x| x).collect()
        })
    }

    pub fn create_playlist(&self) {
        let playlist = Playlist {
            name: format!("New playlist created at {}", Local::now().format("%A %-l:%M:%S%P").to_string()),
            songs: vec![],
        };
        let Ok(playlists) = self.playlist_list.try_borrow_mut() else {
            return;
        };
        playlists.push_item(playlist);

    }

    pub fn selected_playlist_mut(&self, f: impl FnOnce(&mut Playlist)) {
        let Ok(playlists) = self.playlist_list.try_borrow_mut() else {
            return;
        };

        playlists.with_selected_item_mut(f);
    }

    pub fn add_song(&self, song: Song) {
        self.selected_playlist_mut(move |pl| {
            pl.songs.push(song.clone());
        });
    }
    pub fn add_songs(&self, songs: &mut Vec<Song>) {
        self.selected_playlist_mut(move |pl| {
            pl.songs.append(songs);
        });
    }

    pub fn add_cue(&self, cue_sheet: CueSheet) {
        self.selected_playlist_mut(move |pl| {
            let mut songs = Song::from_cue_sheet(cue_sheet);
            pl.songs.append(&mut songs);
        });
    }
}

impl Drop for Playlists<'_> {
    fn drop(&mut self) {
        log::trace!("Playlists.drop()");
    }
}
