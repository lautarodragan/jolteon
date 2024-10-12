use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    sync::{
        atomic::{AtomicUsize, Ordering as AtomicOrdering},
        Mutex,
        MutexGuard,
        Arc,
    },
    path::PathBuf,
};

use crossterm::event::KeyEvent;

use crate::{
    config::Theme,
};

pub struct ArtistList<'a> {
    pub(super) theme: Theme,

    pub(super) artists: Mutex<Vec<String>>,
    pub(super) selected_index: AtomicUsize,
    pub(super) selected_artist: Mutex<String>,

    pub(super) filter: Mutex<String>,

    pub(super) on_select_fn: Mutex<Box<dyn FnMut(String) + 'a>>,
    pub(super) on_confirm_fn: Mutex<Box<dyn FnMut(String) + 'a>>,
    pub(super) on_delete_fn: Mutex<Box<dyn FnMut(String) + 'a>>,

    pub(super) offset: AtomicUsize,
    pub(super) height: AtomicUsize,
}

impl<'a> ArtistList<'a> {
    pub fn new(theme: Theme) -> Self {
        Self {
            theme,

            on_select_fn: Mutex::new(Box::new(|_| {}) as _),
            on_confirm_fn: Mutex::new(Box::new(|_| {}) as _),
            on_delete_fn: Mutex::new(Box::new(|_| {}) as _),

            artists: Mutex::new(Vec::new()),
            selected_index: AtomicUsize::new(0),
            selected_artist: Mutex::new("".to_string()),

            filter: Mutex::new(String::new()),

            offset: AtomicUsize::new(0),
            height: AtomicUsize::new(0),
        }
    }

    pub fn on_select(&self, cb: impl FnMut(String) + 'a) {
        *self.on_select_fn.lock().unwrap() = Box::new(cb);
    }

    pub fn on_confirm(&self, cb: impl FnMut(String) + 'a) {
        *self.on_confirm_fn.lock().unwrap() = Box::new(cb);
    }

    pub fn on_delete(&self, cb: impl FnMut(String) + 'a) {
        *self.on_delete_fn.lock().unwrap() = Box::new(cb);
    }

    pub fn set_artists(&self, artists: Vec<String>) {
        *self.artists.lock().unwrap() = artists;
    }

    pub fn selected_artist(&self) -> String {
        self.selected_artist.lock().unwrap().clone()
    }

    pub fn set_selected_artist(&self, artist: String) {
        *self.selected_artist.lock().unwrap() = artist;
    }

    pub fn add_artist(&self, artist: String) {
        let mut artists = self.artists.lock().unwrap();

        if !artists.contains(&artist) {
            artists.push(artist.clone());
        }

        artists.sort_unstable();

        let i = self.selected_index.load(AtomicOrdering::SeqCst);
        let selected_artist = artists[i].clone();
        *self.selected_artist.lock().unwrap() = selected_artist;
    }
}

impl Drop for ArtistList<'_> {
    fn drop(&mut self) {
        log::trace!("Artists.drop()");
    }
}
