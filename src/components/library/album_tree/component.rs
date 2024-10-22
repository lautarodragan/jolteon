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
    Artist(String, bool),
    #[allow(dead_code)]
    Album(String),
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
            AlbumTreeItem::Artist(s, _) => s,
            AlbumTreeItem::Album(s) => s,
        };
        write!(f, "{}", x)
    }
}

pub struct AlbumTree<'a> {
    pub(super) theme: Theme,

    pub(super) item_list: Mutex<Vec<AlbumTreeItem>>,
    pub(super) item_tree: Mutex<HashMap<String, Vec<String>>>,
    pub(super) selected_index: AtomicUsize,
    pub(super) selected_item: Mutex<AlbumTreeItem>,

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
            item_list: Mutex::new(Vec::new()),
            selected_index: AtomicUsize::new(0),
            selected_item: Mutex::new(AlbumTreeItem::Artist("".to_string(), false)),

            filter: Mutex::new(String::new()),

            offset: AtomicUsize::new(0),
            height: AtomicUsize::new(0),
        }
    }

    pub fn on_select(&self, cb: impl FnMut(AlbumTreeItem) + 'a) {
        // log::debug!(target: "::albumtree.on_select", "lol {:#?}", *self.items.lock().unwrap());
        *self.on_select_fn.lock().unwrap() = Box::new(cb);
    }

    pub fn on_confirm(&self, cb: impl FnMut(AlbumTreeItem) + 'a) {
        *self.on_confirm_fn.lock().unwrap() = Box::new(cb);
    }

    pub fn on_delete(&self, cb: impl FnMut(AlbumTreeItem) + 'a) {
        *self.on_delete_fn.lock().unwrap() = Box::new(cb);
    }

    // #[allow(dead_code)]
    // pub fn set_artists(&self, artists: Vec<String>) {
    //     *self.artists.lock().unwrap() = artists.into_iter().map(|a| AlbumTreeItem::Artist(a)).collect();
    // }

    pub fn selected_artist(&self) -> AlbumTreeItem {
        self.selected_item.lock().unwrap().clone()
    }

    #[allow(dead_code)]
    pub fn set_selected_artist(&self, artist: String) {
        *self.selected_item.lock().unwrap() = AlbumTreeItem::Artist(artist, false);
    }

    pub fn add_artist(&self, artist: String) {
        let mut item = self.item_list.lock().unwrap();

        if !item.iter().any(|a| matches!(a, AlbumTreeItem::Artist(a, _) if *a == artist)) {
            item.push(AlbumTreeItem::Artist(artist.clone(), false));
        }

        item.sort_by(|a, b| {
            match (a, b) {
                (AlbumTreeItem::Artist(a, _), AlbumTreeItem::Artist(b, _)) => {
                    a.cmp(b)
                }
                (AlbumTreeItem::Artist(_, _), AlbumTreeItem::Album(_)) => {
                    Ordering::Greater
                }
                (AlbumTreeItem::Album(_), AlbumTreeItem::Artist(_, _)) => {
                    Ordering::Less
                }
                (AlbumTreeItem::Album(ref a), AlbumTreeItem::Album(ref b)) => {
                    a.cmp(b)
                }
            }
        });

        let i = self.selected_index.load(AtomicOrdering::SeqCst);
        let selected_item = item[i].clone();
        *self.selected_item.lock().unwrap() = selected_item;
    }

    pub fn add_album(&self, artist: String, album: String) {
        // log::warn!("add_album {artist} {album}");
        let mut items = self.item_tree.lock().unwrap();

        if let Some(x) = items.get_mut(&artist) {
            if !x.contains(&album) { // Set?
                x.push(album);
                x.sort();
            }
        } else {
            items.insert(artist, vec![]);
        }

        // let i = self.selected_index.load(AtomicOrdering::SeqCst);
        // let selected_artist = artists[i].clone();
        // *self.selected_artist.lock().unwrap() = selected_artist;
    }
}

impl Drop for AlbumTree<'_> {
    fn drop(&mut self) {
        log::trace!("Artists.drop()");
    }
}
