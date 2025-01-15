use std::{
    cell::{RefCell, RefMut},
    collections::HashSet,
    rc::Rc,
};

use crate::{
    components::{list::Direction, FocusGroup, List},
    config::Theme,
    structs::Song,
};

use super::album_tree_item::AlbumTreeItem;

pub struct Library<'a> {
    #[allow(dead_code)]
    pub(super) theme: Theme,

    pub(super) songs_by_artist: Rc<RefCell<crate::files::Library>>,

    pub(super) song_list: Rc<List<'a, Song>>,
    pub(super) album_tree: Rc<List<'a, AlbumTreeItem>>,
    pub(super) focus_group: FocusGroup<'a>,

    pub(super) on_select_songs_fn: Rc<RefCell<Box<dyn FnMut(Vec<Song>) + 'a>>>,
}

impl<'a> Library<'a> {
    pub fn new(theme: Theme) -> Self {
        let on_select_songs_fn: Rc<RefCell<Box<dyn FnMut(Vec<Song>) + 'a>>> =
            Rc::new(RefCell::new(Box::new(|_| {}) as _));

        let songs_by_artist = Rc::new(RefCell::new(crate::files::Library::from_file()));
        let mut album_tree = List::new(theme, vec![]);
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
                    let songs = songs.borrow();

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

        album_tree.on_enter({
            let songs = songs_by_artist.clone();
            let on_select_songs_fn = on_select_songs_fn.clone();

            move |item| {
                log::trace!(target: "::library.album_tree.on_confirm", "artist confirmed {:?}", item);

                let (artist, album) = match item {
                    AlbumTreeItem::Artist(artist) => (artist, None),
                    AlbumTreeItem::Album(artist, album) => (artist, Some(album)),
                };

                let songs = {
                    let songs = songs.borrow();
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

                on_select_songs_fn.borrow_mut()(songs);
            }
        });

        album_tree.on_delete({
            let songs_by_artist = songs_by_artist.clone();

            move |item, _index| {
                log::trace!(target: "::library.album_tree.on_delete", "item deleted {:?}", item);

                let mut songs_by_artist = songs_by_artist.borrow_mut();
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

        let album_tree = Rc::new(album_tree);
        let focus_group = FocusGroup::new(vec![album_tree.clone(), song_list.clone()]);

        let lib = Self {
            theme,
            focus_group,

            on_select_songs_fn,

            songs_by_artist,
            song_list,
            album_tree,
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
        *self.on_select_songs_fn.borrow_mut() = Box::new(cb);
    }

    pub fn add_songs(&self, songs_to_add: Vec<Song>) {
        let mut songs_by_artist = self.songs_by_artist.borrow_mut();
        songs_by_artist.add_songs(songs_to_add);
        drop(songs_by_artist);

        self.refresh_components();
    }

    pub fn refresh_components(&self) {
        let songs_by_artist = self.songs_by_artist.borrow_mut();
        self.refresh_artist_tree(&songs_by_artist);
        self.refresh_song_list(&songs_by_artist);
    }

    fn refresh_artist_tree(&self, songs_by_artist: &RefMut<crate::files::Library>) {
        let mut items = vec![];

        let mut artists: Vec<String> = songs_by_artist.songs_by_artist.keys().cloned().collect();
        artists.sort();

        for artist in artists {
            items.push(AlbumTreeItem::Artist(artist.clone()));

            let mut albums = HashSet::new();
            let songs = songs_by_artist.songs_by_artist.get(artist.as_str()).unwrap();
            let mut year = 0;

            for song in songs {
                if let Some(y) = song.year {
                    year = year.min(y); // hopefully all songs will have the same year.
                }
                let album = song.album.clone().unwrap_or("(no album)".to_string());
                albums.insert((year, album.clone()));
            }

            let mut albums: Vec<(u32, String)> = albums.into_iter().collect();
            albums.sort_by_key(|i| i.0);

            for (_year, album) in albums {
                items.push(AlbumTreeItem::Album(artist.clone(), album));
            }
        }

        self.album_tree.set_items(items);
    }

    fn refresh_song_list(&self, library: &RefMut<crate::files::Library>) {
        let songs = self.album_tree.with_selected_item(|selected_item| {
            let (selected_artist, selected_album) = match selected_item {
                AlbumTreeItem::Artist(selected_artist) => (selected_artist.as_str(), None),
                AlbumTreeItem::Album(selected_artist, selected_album) => (selected_artist.as_str(), Some(selected_album.as_str())),
            };

            let Some(songs) = library.songs_by_artist.get(selected_artist) else {
                log::error!(target: "::library.add_songs", "This should never happen! There's an error with songs_by_artist/songs_by_artist.");
                panic!();
            };

            if let Some(selected_album) = selected_album {
                songs
                    .iter()
                    .filter(|s| s.album.as_ref().is_some_and(|sa| *sa == selected_album))
                    .cloned()
                    .collect()
            } else {
                songs.clone()
            }
        });

        self.song_list.set_items(songs);
    }
}

impl Drop for Library<'_> {
    fn drop(&mut self) {
        log::trace!("Library.drop()");
    }
}
