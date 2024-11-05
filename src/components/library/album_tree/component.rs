use std::{
    fmt::{Display, Formatter},
    sync::{
        atomic::{AtomicUsize, Ordering as AtomicOrdering},
        Mutex,
    },
};

use crate::{
    config::Theme,
};

use super::AlbumTreeItem;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AlbumTreeEntryArtist {
    pub artist: String,
    pub albums: Vec<String>,
    pub is_open: bool,
    pub is_match: bool,
}

pub struct AlbumTree<'a> {
    pub(super) theme: Theme,

    pub(super) artist_list: Mutex<Vec<AlbumTreeEntryArtist>>,
    pub(super) selected_artist: AtomicUsize,
    pub(super) selected_album: AtomicUsize,

    pub(super) filter: Mutex<String>,

    pub(super) on_select_fn: Mutex<Box<dyn FnMut(AlbumTreeItem) + 'a>>,
    pub(super) on_confirm_fn: Mutex<Box<dyn FnMut(AlbumTreeItem) + 'a>>,
    pub(super) on_delete_fn: Mutex<Box<dyn FnMut(AlbumTreeItem) + 'a>>,

    pub(super) offset: AtomicUsize,
    pub(super) height: AtomicUsize,
}

impl<'a> AlbumTree<'a> {
    pub fn new(theme: Theme) -> Self {
        Self {
            theme,

            on_select_fn: Mutex::new(Box::new(|_| {}) as _),
            on_confirm_fn: Mutex::new(Box::new(|_| {}) as _),
            on_delete_fn: Mutex::new(Box::new(|_| {}) as _),

            artist_list: Mutex::new(Vec::new()),
            selected_artist: AtomicUsize::new(0),
            selected_album: AtomicUsize::new(0),

            filter: Mutex::new(String::new()),

            offset: AtomicUsize::new(0),
            height: AtomicUsize::new(0),
        }
    }

    pub fn on_select(&self, cb: impl FnMut(AlbumTreeItem) + 'a) {
        *self.on_select_fn.lock().unwrap() = Box::new(cb);
    }

    pub fn on_confirm(&self, cb: impl FnMut(AlbumTreeItem) + 'a) {
        *self.on_confirm_fn.lock().unwrap() = Box::new(cb);
    }

    pub fn on_delete(&self, cb: impl FnMut(AlbumTreeItem) + 'a) {
        *self.on_delete_fn.lock().unwrap() = Box::new(cb);
    }

    pub fn selected_item(&self) -> Option<AlbumTreeItem> {
        let i = self.selected_artist.load(AtomicOrdering::SeqCst);
        let artist_list = self.artist_list.lock().unwrap();
        let Some(artist) = artist_list.get(i) else {
            return None;
        };

        if !artist.is_open {
            Some(AlbumTreeItem::Artist(artist.artist.clone()))
        } else {
            let selected_album = self.selected_album.load(AtomicOrdering::SeqCst);

            let Some(album) = artist.albums.get(selected_album) else {
                log::error!("artist {} selected_album {selected_album} >= len {}", artist.artist, artist.albums.len());
                panic!("no album at selected index!");
            };

            Some(AlbumTreeItem::Album(artist.artist.clone(), album.clone()))

        }
    }

    pub fn add_album(&self, artist: String, album: String) {
        let mut artists = self.artist_list.lock().unwrap();

        match artists.binary_search_by(|a| a.artist.cmp(&artist)) {
            Ok(i) => {
                if let Err(j) = artists[i].albums.binary_search_by(|a| a.cmp(&album)) {
                    artists[i].albums.insert(j, album.clone());
                }
            }
            Err(i) => {
                artists.insert(i, AlbumTreeEntryArtist {
                    artist: artist.clone(),
                    albums: vec![],
                    is_open: false,
                    is_match: false,
                });
            }
        }
    }
}

impl Drop for AlbumTree<'_> {
    fn drop(&mut self) {
        log::trace!("Artists.drop()");
    }
}
