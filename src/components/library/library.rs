use std::{
    cmp::Ordering,
    rc::Rc,
    sync::{
        atomic::{AtomicU8, Ordering as AtomicOrdering},
        Mutex,
        MutexGuard,
    },
    path::PathBuf,
};

use crossterm::event::KeyEvent;

use crate::{
    structs::{Song},
    config::Theme,
    cue::CueSheet,
};
use crate::components::List;
use super::{album_tree::{AlbumTree, AlbumTreeItem}};

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

    pub(super) songs_by_artist: Rc<Mutex<crate::files::Library>>,

    pub(super) song_list: Rc<List<'a, Song>>,
    pub(super) album_tree: Rc<AlbumTree<'a>>,

    focused_element: AtomicLibraryScreenElement,

    pub(super) on_select_fn: Rc<Mutex<Box<dyn FnMut(Song, KeyEvent) + 'a>>>,
    pub(super) on_select_songs_fn: Rc<Mutex<Box<dyn FnMut(Vec<Song>) + 'a>>>,
}

impl Ord for Song {
    fn cmp(&self, other: &Self) -> Ordering {
        match (&self.album, &other.album) {
            (Some(album_a), Some(album_b)) if album_a == album_b => {
                match self.disc_number.cmp(&other.disc_number) {
                    Ordering::Equal => {
                        match (&self.track, &other.track) {
                            (Some(a), Some(b)) => a.cmp(b),
                            (Some(_), None) => Ordering::Greater,
                            (None, Some(_)) => Ordering::Less,
                            _ => self.title.cmp(&other.title),
                        }
                    }
                    o => o
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
    pub fn new(theme: Theme) -> Self {
        let on_select_fn: Rc<Mutex<Box<dyn FnMut(Song, KeyEvent) + 'a>>> = Rc::new(Mutex::new(Box::new(|_, _| {}) as _));
        let on_select_songs_fn: Rc<Mutex<Box<dyn FnMut(Vec<Song>) + 'a>>> = Rc::new(Mutex::new(Box::new(|_| {}) as _));

        let songs_by_artist = Rc::new(Mutex::new(crate::files::Library::from_file()));
        let album_tree = Rc::new(AlbumTree::new(theme));
        let song_list = Rc::new(List::new(theme, vec![]));

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

                    match songs.songs_by_artist.get(artist.as_str()) {
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

                song_list.set_items(artist_songs);
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
                    let Some(songs) = songs.songs_by_artist.get(artist.as_str()) else {
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
                        songs_by_artist.remove_artist(artist);
                    }
                    AlbumTreeItem::Album(ref artist, ref album) => {
                        songs_by_artist.remove_album(artist, album);
                    }
                };
            }
        });

        song_list.on_select({
            let on_select_fn = on_select_fn.clone();
            move |song, key| {
                log::trace!(target: "::library.song_list.on_select", "song selected {:#?}", song);

                // song.debug_tags();

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

        lib.refresh_components();
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
        songs_by_artist.add_songs(songs_to_add);
        drop(songs_by_artist);

        self.refresh_components();
    }

    pub fn refresh_components(&self) {
        let mut songs_by_artist = self.songs_by_artist.lock().unwrap();
        self.refresh_artist_tree(&mut songs_by_artist);
        self.refresh_song_list(&mut songs_by_artist);
    }

    fn refresh_artist_tree(&self, songs_by_artist: &MutexGuard<crate::files::Library>) {
        for (artist, songs) in &songs_by_artist.songs_by_artist {
            for song in songs {
                let album = song.album.clone().unwrap_or("(no album)".to_string());
                self.album_tree.add_album(artist.clone(), album);
            }
        }
    }

    fn refresh_song_list(&self, songs_by_artist: &MutexGuard<crate::files::Library>) {
        let Some(selected_item) = self.album_tree.selected_item() else {
            return;
        };

        let (selected_artist, selected_album) = match selected_item {
            AlbumTreeItem::Artist(selected_artist) => (selected_artist, None),
            AlbumTreeItem::Album(selected_artist, selected_album) => (selected_artist, Some(selected_album)),
        };

        let Some(songs) = songs_by_artist.songs_by_artist.get(&selected_artist) else {
            log::error!(target: "::library.add_songs", "This should never happen! There's an error with songs_by_artist/songs_by_artist.");
            return;
        };

        let songs = if let Some(selected_album) = selected_album {
            songs.iter().filter(|s| s.album.as_ref().is_some_and(|sa| *sa == selected_album)).cloned().collect()
        } else {
            songs.clone()
        };
        self.song_list.set_items(songs);
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

fn next_index_by_album(songs: &Vec<Song>, i: i32, key: crossterm::event::KeyCode) -> Option<usize> {
    let Some(song) = (*songs).get(i as usize) else {
        log::error!("no selected song");
        return None;
    };

    let Some(ref selected_album) = song.album else {
        log::warn!("no selected song album");
        return None;
    };

    let next_song_index = if key == crossterm::event::KeyCode::Down {
        songs
            .iter()
            .skip(i as usize)
            .position(|s| s.album.as_ref().is_some_and(|a| a != selected_album))
            .map(|ns| ns.saturating_add(i as usize))
    } else {
        songs
            .iter()
            .take(i as usize)
            .rposition(|s| s.album.as_ref().is_some_and(|a| a != selected_album))
            .and_then(|ns| songs.get(ns))
            .and_then(|ref s| s.album.as_ref())
            .and_then(|next_song_album| {
                songs
                    .iter()
                    .position(|song| {
                        song.album.as_ref().is_some_and(|a| a.as_str() == next_song_album)
                    })
            })
    };

    next_song_index
}
