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

    pub(super) songs_by_artist: Rc<Mutex<HashMap<String, Vec<Song>>>>,
    pub(super) song_list: Rc<SongList<'a>>,
    pub(super) album_tree: Rc<AlbumTree<'a>>,

    focused_element: AtomicLibraryScreenElement,

    pub(super) on_select_fn: Rc<Mutex<Box<dyn FnMut(Song, KeyEvent) + 'a>>>,
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
        let on_select_fn: Rc<Mutex<Box<dyn FnMut(Song, KeyEvent) + 'a>>> = Rc::new(Mutex::new(Box::new(|_, _| {}) as _));
        let on_select_songs_fn: Rc<Mutex<Box<dyn FnMut(Vec<Song>) + 'a>>> = Rc::new(Mutex::new(Box::new(|_| {}) as _));

        let songs_by_artist = Rc::new(Mutex::new(HashMap::<String, Vec<Song>>::new()));
        let album_tree = Rc::new(AlbumTree::new(theme));
        let song_list = Rc::new(SongList::new(theme));

        album_tree.on_select({
            let songs = songs_by_artist.clone();
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
            let songs = songs_by_artist.clone();
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
            let songs_by_artist = songs_by_artist.clone();

            move |item| {
                log::trace!(target: "::library.album_tree.on_delete", "item deleted {:?}", item);

                let mut songs_by_artist = songs_by_artist.lock().unwrap();
                match item {
                    AlbumTreeItem::Artist(ref artist) => {
                        songs_by_artist.remove(artist);
                        crate::files::Library::save_hash_map(&*songs_by_artist);
                    }
                    AlbumTreeItem::Album(ref artist, album) => {
                        let Some(artist_songs) = songs_by_artist.get_mut(artist) else {
                            log::error!(target: "::library.album_tree.on_delete", "Tried to delete artist's songs, but the artist has no songs.");
                            return;
                        };
                        artist_songs.retain(|s| s.album.as_ref().is_some_and(|a| *a != album));
                        crate::files::Library::save_hash_map(&*songs_by_artist);
                    }
                };
            }
        });

        song_list.on_select({
            let on_select_fn = on_select_fn.clone();
            move |(song, key)| {
                log::trace!(target: "::library.song_list.on_select", "song selected {:?}", song);

                let mut on_select_fn = on_select_fn.lock().unwrap();
                on_select_fn(song, key);
            }
        });

        let lib = Self {
            theme,
            focused_element: AtomicLibraryScreenElement::new(),

            on_select_fn,
            on_select_songs_fn,

            songs_by_artist,
            song_list,
            album_tree,
        };

        lib.add_songs(songs); // TODO: we're saving when we load the file!

        lib
    }

    pub fn focused_element(&self) -> LibraryScreenElement {
        self.focused_element.load()
    }

    pub fn set_focused_element(&self, v: LibraryScreenElement) {
        self.focused_element.store(v);
    }

    pub fn on_select(&self, cb: impl FnMut(Song, KeyEvent) + 'a) {
        *self.on_select_fn.lock().unwrap() = Box::new(cb);
    }

    pub fn on_select_songs_fn(&self, cb: impl FnMut(Vec<Song>) + 'a) {
        *self.on_select_songs_fn.lock().unwrap() = Box::new(cb);
    }

    pub fn add_songs(&self, songs_to_add: Vec<Song>) {
        let mut songs_by_artist = self.songs_by_artist.lock().unwrap();

        for song in songs_to_add {
            let Some(ref artist) = song.artist else {
                log::error!("Library.add_song() -> no artist! {:?}", song);
                continue;
            };

            let album = song.album.clone().unwrap_or("(no album)".to_string());
            self.album_tree.add_album(artist.clone(), album);

            let artist_songs = songs_by_artist.entry(artist.clone()).or_insert(vec![]);
            if let Err(i) = artist_songs.binary_search(&song) {
                artist_songs.insert(i, song);
            }
        }

        let (selected_artist, selected_album) = match self.album_tree.selected_item() {
            AlbumTreeItem::Artist(selected_artist) => (selected_artist, None),
            AlbumTreeItem::Album(selected_artist, selected_album) => (selected_artist, Some(selected_album)),
        };

        let Some(songs) = songs_by_artist.get(&selected_artist) else {
            log::error!(target: "::library.add_songs", "This should never happen! There's an error with songs_by_artist/songs_by_artist.");
            return;
        };

        let songs = if let Some(selected_album) = selected_album {
            songs.iter().filter(|s| s.album.as_ref().is_some_and(|sa| *sa == selected_album)).cloned().collect()
        } else {
            songs.clone()
        };
        self.song_list.set_songs(songs);

        crate::files::Library::save_hash_map(&*songs_by_artist);
    }

    pub fn add_song(&self, song: Song) {
        self.add_songs(vec![song]);
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
