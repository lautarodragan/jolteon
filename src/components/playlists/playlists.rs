use std::{
    sync::{
        atomic::{AtomicUsize, AtomicBool, Ordering},
        Mutex,
    },
};

use chrono::Local;
use crossterm::event::{KeyEvent};

use crate::{
    structs::{Song, Playlist},
    config::Theme,
    cue::CueSheet,
};

#[derive(Eq, PartialEq)]
pub(super) enum PlaylistScreenElement {
    PlaylistList,
    SongList,
}

pub struct Playlists<'a> {
    pub(super) playlists: Mutex<Vec<Playlist>>,
    pub(super) theme: Theme,
    pub(super) focused_element: Mutex<PlaylistScreenElement>,
    pub(super) selected_playlist_index: AtomicUsize,
    pub(super) selected_song_index: AtomicUsize,
    pub(super) renaming: AtomicBool,
    pub(super) on_select_fn: Mutex<Box<dyn FnMut((Song, KeyEvent)) + 'a>>,
    pub(super) on_select_playlist_fn: Mutex<Box<dyn FnMut(Vec<Song>, KeyEvent) + 'a>>,
}

impl<'a> Playlists<'a> {
    pub fn new(theme: Theme, playlists: Vec<Playlist>) -> Self {
        Self {
            // playlists: Mutex::new(vec![
            //     Playlist::new("My first Jolteon playlist".to_string()),
            //     Playlist::new("Ctrl+N to create new ones".to_string()),
            //     Playlist::new("Alt+N to rename".to_string()),
            // ]),
            playlists: Mutex::new(playlists),
            selected_playlist_index: AtomicUsize::new(0),
            selected_song_index: AtomicUsize::new(0),
            theme,
            focused_element: Mutex::new(PlaylistScreenElement::PlaylistList),
            renaming: AtomicBool::new(false),
            on_select_fn: Mutex::new(Box::new(|_| {}) as _),
            on_select_playlist_fn: Mutex::new(Box::new(|_, _| {}) as _),
        }
    }

    pub fn on_select(&self, cb: impl FnMut((Song, KeyEvent)) + 'a) {
        *self.on_select_fn.lock().unwrap() = Box::new(cb);
    }

    pub fn on_select_playlist(&self, cb: impl FnMut(Vec<Song>, KeyEvent) + 'a) {
        *self.on_select_playlist_fn.lock().unwrap() = Box::new(cb);
    }

    pub fn playlists(&self) -> Vec<Playlist> {
        let playlists = self.playlists.lock().unwrap();
        playlists.clone()
    }

    pub fn create_playlist(&self) {
        let playlist = Playlist {
            name: format!("New playlist created at {}", Local::now().format("%A %-l:%M:%S%P").to_string()),
            songs: vec![],
        };
        self.playlists.lock().unwrap().push(playlist);
    }

    pub fn selected_playlist<T>(&self, f: impl FnOnce(&Playlist) -> T) -> Option<T> {
        let selected_playlist_index = self.selected_playlist_index.load(Ordering::Relaxed);
        let mut playlists = self.playlists.lock().unwrap();

        if let Some(selected_playlist) = playlists.get_mut(selected_playlist_index) {
            Some(f(selected_playlist))
        } else {
            None
        }
    }

    pub fn selected_playlist_mut(&self, f: impl FnOnce(&mut Playlist)) {
        let selected_playlist_index = self.selected_playlist_index.load(Ordering::Relaxed);
        let mut playlists = self.playlists.lock().unwrap();

        if let Some(selected_playlist) = playlists.get_mut(selected_playlist_index) {
            f(selected_playlist);
        }
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
