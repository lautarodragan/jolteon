use std::{
    cell::Cell,
    rc::Rc,
    sync::{Mutex, MutexGuard},
};

use crossterm::event::KeyEvent;

use crate::{components::list::Direction, components::List, config::Theme, structs::Song, ui::Component};

use super::album_tree::{AlbumTree, AlbumTreeItem};

pub struct Library<'a> {
    #[allow(dead_code)]
    pub(super) theme: Theme,

    pub(super) songs_by_artist: Rc<Mutex<crate::files::Library>>,

    pub(super) song_list: Rc<List<'a, Song>>,
    pub(super) album_tree: Rc<AlbumTree<'a>>,
    pub(super) components: Rc<Vec<Component<'a>>>,
    pub(super) focused_component: Rc<Cell<usize>>,

    pub(super) on_select_songs_fn: Rc<Mutex<Box<dyn FnMut(Vec<Song>) + 'a>>>,
}

impl<'a> Library<'a> {
    pub fn new(theme: Theme) -> Self {
        let on_select_fn: Rc<Mutex<Box<dyn FnMut(Song, KeyEvent) + 'a>>> =
            Rc::new(Mutex::new(Box::new(|_, _| {}) as _));
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
                    AlbumTreeItem::Album(artist, album) => (artist, Some(album)),
                };

                let artist_songs = {
                    let songs = songs.lock().unwrap();

                    match songs.songs_by_artist.get(artist.as_str()) {
                        Some(artist_songs) => match album {
                            Some(album) => artist_songs
                                .iter()
                                .filter(|s| s.album.as_ref().is_some_and(|a| *a == album))
                                .cloned()
                                .collect(),
                            None => artist_songs.clone(),
                        },
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
                    AlbumTreeItem::Artist(artist) => (artist, None),
                    AlbumTreeItem::Album(artist, album) => (artist, Some(album)),
                };

                let songs = {
                    let songs = songs.lock().unwrap();
                    let Some(songs) = songs.songs_by_artist.get(artist.as_str()) else {
                        log::warn!(target: "::library.album_tree.on_confirm", "no songs for artist {:?}", artist);
                        return;
                    };

                    if let Some(album) = album {
                        songs
                            .iter()
                            .filter(|s| s.album.as_ref().is_some_and(|a| *a == album))
                            .cloned()
                            .collect()
                    } else {
                        songs.to_vec()
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

        song_list.find_next_item_by_fn({
            |songs, i, direction| {
                let Some(ref selected_album) = songs[i].album else {
                    log::warn!("no selected song album");
                    return None;
                };

                if direction == Direction::Forwards {
                    songs
                        .iter()
                        .skip(i)
                        .position(|s| s.album.as_ref().is_some_and(|a| a != selected_album))
                        .map(|ns| ns.saturating_add(i))
                } else {
                    songs
                        .iter()
                        .take(i)
                        .rposition(|s| s.album.as_ref().is_some_and(|a| a != selected_album))
                        .and_then(|ns| songs.get(ns))
                        .and_then(|s| s.album.as_ref())
                        .and_then(|next_song_album| {
                            songs
                                .iter()
                                .position(|song| song.album.as_ref().is_some_and(|a| a.as_str() == next_song_album))
                        })
                }
            }
        });

        let components: Rc<Vec<Component>> = Rc::new(vec![
            Component::RefRc(album_tree.clone()),
            Component::RefRc(song_list.clone()),
        ]);

        let lib = Self {
            theme,
            focused_component: Rc::new(Cell::new(0)),

            on_select_songs_fn,

            songs_by_artist,
            song_list,
            album_tree,
            components,
        };

        lib.refresh_components();
        lib
    }

    pub fn on_enter(&self, cb: impl Fn(Song) + 'a) {
        self.song_list.on_enter(cb);
    }

    pub fn on_enter_alt(&self, cb: impl Fn(Song) + 'a) {
        self.song_list.on_enter_alt(cb);
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
        let songs_by_artist = self.songs_by_artist.lock().unwrap();
        self.refresh_artist_tree(&songs_by_artist);
        self.refresh_song_list(&songs_by_artist);
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
            songs
                .iter()
                .filter(|s| s.album.as_ref().is_some_and(|sa| *sa == selected_album))
                .cloned()
                .collect()
        } else {
            songs.clone()
        };
        self.song_list.set_items(songs);
    }
}

impl Drop for Library<'_> {
    fn drop(&mut self) {
        log::trace!("Library.drop()");
    }
}
