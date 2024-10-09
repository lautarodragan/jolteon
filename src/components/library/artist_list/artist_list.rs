use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    sync::{
        atomic::{AtomicUsize, Ordering as AtomicOrdering},
        Mutex,
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

    pub artists: Mutex<Vec<String>>,
    pub(super) selected_index: AtomicUsize,

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
        let artists = self.artists.lock().unwrap();

        let i = self.selected_index.load(AtomicOrdering::SeqCst);

        if i >= artists.len() {
            log::error!(target: "::ArtistList.selected_artist()", "selected_index > artists.len");
            return "".to_string();
        }

        artists[i].clone()
    }
}

impl Drop for ArtistList<'_> {
    fn drop(&mut self) {
        log::trace!("Artists.drop()");
    }
}
