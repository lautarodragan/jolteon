use std::{
    fmt::{Display, Formatter},
    sync::{
        atomic::{AtomicUsize, Ordering as AtomicOrdering},
        Mutex,
    },
    collections::HashMap,
};
use std::cmp::Ordering;
use crate::{
    config::Theme,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AlbumTreeItem {
    Artist(String),
    Album(String, String),
}

impl Ord for AlbumTreeItem {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (AlbumTreeItem::Artist(a), AlbumTreeItem::Artist(b)) => {
                a.cmp(b)
            }
            (AlbumTreeItem::Artist(_), AlbumTreeItem::Album(_, _)) => {
                Ordering::Greater
            }
            (AlbumTreeItem::Album(_, _), AlbumTreeItem::Artist(_)) => {
                Ordering::Less
            }
            (AlbumTreeItem::Album(_, ref a), AlbumTreeItem::Album(_, ref b)) => {
                a.cmp(b)
            }
        }
    }
}

impl PartialOrd for AlbumTreeItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AlbumTreeEntryArtist {
    pub data: String,
    pub is_open: bool,
}

impl AlbumTreeItem {
    pub fn contains(&self, needle: &str) -> bool {
        let haystack = self.to_string();
        haystack.contains(needle) || haystack.to_lowercase().contains(needle.to_lowercase().as_str())
    }
}

impl Display for AlbumTreeItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let x = match self {
            AlbumTreeItem::Artist(s) => s,
            AlbumTreeItem::Album(_, s) => s,
        };
        write!(f, "{}", x)
    }
}

pub struct AlbumTree<'a> {
    pub(super) theme: Theme,

    pub(super) artist_list: Mutex<Vec<AlbumTreeEntryArtist>>, // TODO: just vec<vec<string>> :P
    pub(super) item_tree: Mutex<HashMap<String, Vec<String>>>,
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

            item_tree: Mutex::new(HashMap::new()),
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

    pub fn selected_item(&self) -> AlbumTreeItem {
        let i = self.selected_artist.load(AtomicOrdering::SeqCst);
        let artist_list = self.artist_list.lock().unwrap();
        if let Some(artist) = artist_list.get(i) {
            if !artist.is_open {
                AlbumTreeItem::Artist(artist.data.clone())
            } else {
                let selected_album = self.selected_album.load(AtomicOrdering::SeqCst);
                let tree = self.item_tree.lock().unwrap();

                if let Some(albums) = tree.get(&artist.data) {
                    let album = albums.get(selected_album).unwrap();
                    AlbumTreeItem::Album(artist.data.clone(), album.clone())
                } else {
                    AlbumTreeItem::Album(artist.data.clone(), "(no album name)".to_string())
                }
            }
        } else {
            panic!("say what now?");
        }
    }

    pub fn add_artist(&self, artist: String) {
        let mut artists = self.artist_list.lock().unwrap();

        if let Err(i) = artists.binary_search_by(|a| a.data.cmp(&artist)) {
            artists.insert(i, AlbumTreeEntryArtist {
                data: artist.clone(),
                is_open: false,
            });
        }
    }

    pub fn add_album(&self, artist: String, album: String) {
        let mut items = self.item_tree.lock().unwrap();

        items
            .entry(artist)
            .and_modify(|x| {
                if let Err(i) = x.binary_search(&album) {
                    x.insert(i, album.clone());
                }
            })
            .or_insert(vec![album]);
    }
}

impl Drop for AlbumTree<'_> {
    fn drop(&mut self) {
        log::trace!("Artists.drop()");
    }
}
