use std::{
    cmp::Ordering,
    collections::HashMap,
    rc::Rc,
    sync::{
        atomic::{AtomicU8, Ordering as AtomicOrdering},
        Mutex,
    },
    path::PathBuf,
};

use crossterm::event::KeyEvent;

use crate::{
    structs::{Song},
    config::Theme,
    cue::CueSheet,
};

use super::{song_list::SongList, album_tree::{AlbumTree, AlbumTreeItem}};

#[derive(Eq, PartialEq)]
#[repr(u8)]
pub enum LibraryScreenElement {
    AlbumTree,
    SongList,
}

impl From<u8> for LibraryScreenElement {
    fn from(value: u8) -> Self {
        if value == 0 {
            LibraryScreenElement::AlbumTree
        } else {
            LibraryScreenElement::SongList
        }
    }
}

pub(super) struct AtomicLibraryScreenElement(AtomicU8);

impl AtomicLibraryScreenElement {
    fn new() -> Self {
        Self(AtomicU8::new(0))
    }

    fn load(&self) -> LibraryScreenElement {
        self.0.load(AtomicOrdering::Relaxed).into()
    }

    fn store(&self, v: LibraryScreenElement) {
        self.0.store(v as u8, AtomicOrdering::Relaxed);
    }
}

pub struct Library<'a> {
    #[allow(dead_code)]
    pub(super) theme: Theme,

    pub(super) songs: Rc<Mutex<HashMap<String, Vec<Song>>>>,
    pub(super) song_list: Rc<SongList<'a>>,
    pub(super) album_tree: Rc<AlbumTree<'a>>,

    focused_element: AtomicLibraryScreenElement,

    pub(super) on_select_fn: Rc<Mutex<Box<dyn FnMut((Song, KeyEvent)) + 'a>>>,
    pub(super) on_select_songs_fn: Rc<Mutex<Box<dyn FnMut(Vec<Song>) + 'a>>>,
}

impl Ord for Song {
    fn cmp(&self, other: &Self) -> Ordering {
        match (&self.album, &other.album) {
            (Some(album_a), Some(album_b)) if album_a == album_b => {
                match (&self.track, &other.track) {
                    (Some(a), Some(b)) => a.cmp(b),
                    (Some(_), None) => Ordering::Greater,
                    (None, Some(_)) => Ordering::Less,
                    _ => self.title.cmp(&other.title),
                }
            },
            (Some(album_a), Some(album_b)) if album_a != album_b => {
                match (self.year, other.year) {
                    (Some(ref year_a), Some(ref year_b)) => {
                        if year_a != year_b {
                            year_a.cmp(year_b)
                        } else {
                            album_a.cmp(album_b)
                        }
                    },
                    (Some(_), None) => Ordering::Greater,
                    (None, Some(_)) => Ordering::Less,
                    _ => album_a.cmp(album_b)
                }
            },
            (Some(_), None) => Ordering::Greater,
            (None, Some(_)) => Ordering::Less,
            _ => self.title.cmp(&other.title)
        }
    }
}

impl PartialOrd for Song {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Library<'a> {
    pub fn new(theme: Theme, songs: Vec<Song>) -> Self {
        let on_select_fn: Rc<Mutex<Box<dyn FnMut((Song, KeyEvent)) + 'a>>> = Rc::new(Mutex::new(Box::new(|_| {}) as _));
        let on_select_songs_fn: Rc<Mutex<Box<dyn FnMut(Vec<Song>) + 'a>>> = Rc::new(Mutex::new(Box::new(|_| {}) as _));

        let song_map = Rc::new(Mutex::new(HashMap::<String, Vec<Song>>::new()));
        let album_tree = Rc::new(AlbumTree::new(theme));
        let song_list = Rc::new(SongList::new(theme));

        album_tree.on_select({
            let songs = song_map.clone();
            let song_list = song_list.clone();

            move |item| {
                log::trace!(target: "::library.album_tree.on_select", "selected {:?}", item);

                let (artist, album) = match item {
                    AlbumTreeItem::Artist(artist) => (artist, None),
                    AlbumTreeItem::Album(artist, album) => (artist, Some(album))
                };

                let artist_songs = {
                    let songs = songs.lock().unwrap();

                    match songs.get(artist.as_str()) {
                        Some(artist_songs) => {
                            match album {
                                Some(album) => {
                                    artist_songs.iter().filter(|s| s.album.as_ref().is_some_and(|a| *a == album)).cloned().collect()
                                }
                                None => artist_songs.clone(),
                            }
                        }
                        None => {
                            log::error!(target: "::library.album_tree.on_select", "artist with no songs {artist}");
                            vec![]
                        }
                    }
                };

                song_list.set_songs(artist_songs);
            }
        });

        album_tree.on_confirm({
            let songs = song_map.clone();
            let on_select_songs_fn = on_select_songs_fn.clone();

            move |item| {
                log::trace!(target: "::library.album_tree.on_confirm", "artist confirmed {:?}", item);

                let (artist, album) = match item {
                    AlbumTreeItem::Artist(artist) => {
                        (artist, None)
                    }
                    AlbumTreeItem::Album(artist, album) => {
                        (artist, Some(album))
                    }
                };

                let songs = {
                    let songs = songs.lock().unwrap();
                    let Some(songs) = songs.get(artist.as_str()) else {
                        log::warn!(target: "::library.album_tree.on_confirm", "no songs for artist {:?}", artist);
                        return;
                    };

                    if let Some(album) = album {
                        songs.iter().filter(|s| s.album.as_ref().is_some_and(|a| *a == album)).cloned().collect()
                    } else {
                        songs.iter().cloned().collect()
                    }
                };

                on_select_songs_fn.lock().unwrap()(songs);
            }
        });

        album_tree.on_delete({
            let songs = song_map.clone();

            move |item| {
                log::trace!(target: "::library.album_tree.on_delete", "artist deleted {:?}", item);

                let AlbumTreeItem::Artist(artist) = item else {
                    log::warn!(target: "::library.album_tree.on_select", "artist = album. not implemented");
                    return;
                };

                let mut songs = songs.lock().unwrap();
                songs.remove(artist.as_str());
            }
        });

        song_list.on_select({
            let on_select_fn = on_select_fn.clone();
            move |(song, key)| {
                log::trace!(target: "::library.song_list.on_select", "song selected {:?}", song);

                let mut on_select_fn = on_select_fn.lock().unwrap();
                on_select_fn((song, key));
            }
        });

        let lib = Self {
            theme,
            focused_element: AtomicLibraryScreenElement::new(),

            on_select_fn,
            on_select_songs_fn,

            songs: song_map,
            song_list,
            album_tree,
        };

        lib.add_songs(songs);

        lib
    }

    pub fn focused_element(&self) -> LibraryScreenElement {
        self.focused_element.load()
    }

    pub fn set_focused_element(&self, v: LibraryScreenElement) {
        self.focused_element.store(v);
    }

    pub fn on_select(&self, cb: impl FnMut((Song, KeyEvent)) + 'a) {
        *self.on_select_fn.lock().unwrap() = Box::new(cb);
    }

    pub fn on_select_songs_fn(&self, cb: impl FnMut(Vec<Song>) + 'a) {
        *self.on_select_songs_fn.lock().unwrap() = Box::new(cb);
    }

    pub fn songs(&self) -> Vec<Song> {
        let mut songs = vec![];

        for (_artist, artist_songs) in &*self.songs.lock().unwrap() {
            for song in artist_songs {
                songs.push(song.clone());
            }
        }

        songs
    }

    pub fn add_songs(&self, songs: Vec<Song>) {
        // TODO: anything but this
        for song in songs {
            self.add_song(song);
        }
    }

    pub fn add_song(&self, song: Song) {
        let Some(artist) = song.artist.clone() else {
            log::error!("Library.add_song() -> no artist! {:?}", song);
            return;
        };

        let album = song.album.clone().unwrap_or("(no album)".to_string());

        let mut songs = self.songs.lock().unwrap();

        if let Some(x) = songs.get_mut(&artist) {
            if !x.iter().any(|s| s.path == song.path && s.title == song.title) {
                x.push(song);
                x.sort();
            }
        } else {
            songs.insert(artist.clone(), vec![song]);
        }

        self.album_tree.add_album(artist.clone(), album);

        match self.album_tree.selected_item() {
            AlbumTreeItem::Artist(selected_artist) if selected_artist == artist => {
                let artist_songs = songs.get(artist.as_str())
                    .unwrap() // Safe because we just added it, and we still have the lock on songs.
                    .clone();
                self.song_list.set_songs(artist_songs);
            }
            _ => {}
        };

    }

    pub fn add_cue(&self, cue_sheet: CueSheet) {
        let songs = Song::from_cue_sheet(cue_sheet);
        self.add_songs(songs);
    }

    pub fn add_directory(&self, path: &PathBuf) {
        let songs = Song::from_dir(path);
        self.add_songs(songs);
    }

}

impl Drop for Library<'_> {
    fn drop(&mut self) {
        log::trace!("Library.drop()");
    }
}
