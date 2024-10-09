use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    rc::Rc,
    sync::{
        atomic::{AtomicU8, AtomicUsize, Ordering as AtomicOrdering},
        Mutex,
        Arc,
    },
    path::PathBuf,
};

use crossterm::event::KeyEvent;

use crate::{
    structs::{Song},
    config::Theme,
    cue::CueSheet,
    ui::KeyboardHandlerRef,
};

use super::{song_list::SongList, artist_list::ArtistList};

#[derive(Eq, PartialEq)]
#[repr(u8)]
pub enum LibraryScreenElement {
    ArtistList,
    SongList,
}

impl From<u8> for LibraryScreenElement {
    fn from(value: u8) -> Self {
        if value == 0 {
            LibraryScreenElement::ArtistList
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
    pub(super) theme: Theme,

    pub(super) songs: Rc<Mutex<HashMap<String, Vec<Song>>>>,
    pub(super) song_list: Rc<SongList<'a>>,
    pub(super) artist_list: Rc<ArtistList<'a>>,

    focused_element: AtomicLibraryScreenElement,

    pub(super) selected_artist_index: AtomicUsize,
    pub(super) selected_song_index: AtomicUsize,

    pub(super) on_select_fn: Rc<Mutex<Box<dyn FnMut((Song, KeyEvent)) + 'a>>>,
    pub(super) on_select_songs_fn: Rc<Mutex<Box<dyn FnMut(Vec<Song>) + 'a>>>,

    pub(super) offset: AtomicUsize,
    pub(super) height: AtomicUsize,
}

impl<'a> Library<'a> {
    pub fn new(theme: Theme, songs: Vec<Song>) -> Self {
        let on_select_fn: Rc<Mutex<Box<dyn FnMut((Song, KeyEvent)) + 'a>>> = Rc::new(Mutex::new(Box::new(|_| {}) as _));
        let on_select_songs_fn: Rc<Mutex<Box<dyn FnMut(Vec<Song>) + 'a>>> = Rc::new(Mutex::new(Box::new(|_| {}) as _));

        let song_map = Rc::new(Mutex::new(HashMap::<String, Vec<Song>>::new()));
        let artist_list = Rc::new(ArtistList::new(theme));
        let song_list = Rc::new(SongList::new(theme));

        artist_list.on_select({
            let songs = song_map.clone();
            let song_list = song_list.clone();

            move |artist| {
                log::trace!(target: "::library.artist_list.on_select", "artist selected {:?}", artist);

                let songs = songs.lock().unwrap();
                let artist_songs = songs.get(artist.as_str()).unwrap();
                let artist_songs = artist_songs.iter().map(|s| s.clone()).collect();
                song_list.set_songs(artist_songs);
            }
        });

        artist_list.on_confirm({
            let songs = song_map.clone();
            let on_select_songs_fn = on_select_songs_fn.clone();

            move |artist| {
                log::trace!(target: "::library.artist_list.on_confirm", "artist confirmed {:?}", artist);

                let songs = {
                    let songs = songs.lock().unwrap();
                    let Some(songs) = songs.get(artist.as_str()) else {
                        log::warn!(target: "::library.artist_list.on_confirm", "no songs for artist {:?}", artist);
                        return;
                    };

                    songs.iter().map(|s| s.clone()).collect()
                };

                on_select_songs_fn.lock().unwrap()(songs);

            }
        });

        artist_list.on_delete({
            let songs = song_map.clone();

            move |artist| {
                log::trace!(target: "::library.artist_list.on_delete", "artist deleted {:?}", artist);

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
            artist_list,

            selected_artist_index: AtomicUsize::new(0),
            selected_song_index: AtomicUsize::new(0),

            offset: AtomicUsize::new(0),
            height: AtomicUsize::new(0),
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
        for song in songs {
            self.add_song(song);
        }
    }

    pub fn add_song(&self, song: Song) {
        // log::debug!(target: "::library.add_song", "{:?}", song);

        let mut songs = self.songs.lock().unwrap();

        let Some(artist) = song.artist.clone() else {
            log::error!("Library.add_song() -> no artist! {:?}", song);
            return;
        };

        if let Some(mut x) = songs.get_mut(&artist) {
            if !x.iter().any(|s| s.path == song.path && s.title == song.title) {
                x.push(song);
                x.sort_by(|a, b| {
                    match (&a.album, &b.album) {
                        (Some(album_a), Some(album_b)) if album_a == album_b => {
                            match (&a.track, &b.track) {
                                (Some(a), Some(b)) => a.cmp(b),
                                (Some(_), None) => Ordering::Greater,
                                (None, Some(_)) => Ordering::Less,
                                _ => a.title.cmp(&b.title),
                            }
                        },
                        (Some(album_a), Some(album_b)) if album_a != album_b => {
                            match (a.year, b.year) {
                                (Some(ref year_a), Some(ref year_b)) => {
                                    year_a.cmp(year_b)
                                }
                                _ => album_a.cmp(album_b)
                            }
                        },
                        (Some(_), None) => Ordering::Greater,
                        (None, Some(_)) => Ordering::Less,
                        _ => a.title.cmp(&b.title)
                    }
                })
            }
        } else {
            songs.insert(artist.clone(), vec![song]);
        }

        let mut artists = self.artist_list.artists.lock().unwrap();

        if !artists.contains(&artist) {
            (*artists).push(artist.clone());
        }

        artists.sort_unstable();

        let Some(selected_artist) = artists.get(self.selected_song_index.load(AtomicOrdering::SeqCst)) else {
            return;
        };

        if artist != *selected_artist {
            return;
        }

        let Some(artist_songs) = songs.get(artist.as_str()) else {
            log::error!("No song list for {artist}! This is an error: we should always have a Vec<Song>, even if empty.");
            return;
        };

        // Cloning the song list once per key press is probably more performant than dealing with Rc's, WeakRef's and whatnot.
        let artist_songs = artist_songs.iter().map(|s| s.clone()).collect();
        self.song_list.set_songs(artist_songs);
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
