use std::{
    cell::{RefCell, RefMut},
    collections::HashMap,
    rc::Rc,
};

use crate::{
    components::{list::Direction, FocusGroup, List},
    config::Theme,
    structs::Song,
    actions::Action,
};

use super::album_tree_item::{Album, AlbumTreeItem};

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
            let songs_by_artist = songs_by_artist.clone();
            let song_list = song_list.clone();

            move |item| {
                log::trace!(target: "::library.album_tree.on_select", "selected {:?}", item);

                let songs = match item {
                    AlbumTreeItem::Artist(artist) => {
                        let library = songs_by_artist.borrow();

                        match library.songs_by_artist.get(artist.as_str()) {
                            Some(artist_songs) => {
                                artist_songs.clone()
                            },
                            None => {
                                log::error!(target: "::library.album_tree.on_select", "artist with no songs {artist}");
                                vec![]
                            }
                        }
                    }
                    AlbumTreeItem::Album(album) => {
                        album.songs.clone()
                    }
                };
                song_list.set_items(songs);
            }
        });

        album_tree.on_enter({
            let songs = songs_by_artist.clone();
            let on_select_songs_fn = on_select_songs_fn.clone();

            move |item| {
                log::trace!(target: "::library.album_tree.on_confirm", "artist confirmed {:?}", item);

                let (artist, album) = match item {
                    AlbumTreeItem::Artist(artist) => (artist, None),
                    AlbumTreeItem::Album(album) => (album.artist, Some(album.name)),
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
                    AlbumTreeItem::Album(album) => {
                        songs_by_artist.remove_album(album.artist.as_str(), album.name.as_str());
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

        lib.refresh_components(); // TODO: album_tree = library_to_tree_view
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

    /// TODO: drop the songs_by_artist and mutate the album tree view component directly?
    pub fn add_songs(&self, songs_to_add: Vec<Song>) {
        let mut songs_by_artist = self.songs_by_artist.borrow_mut();
        songs_by_artist.add_songs(songs_to_add);
        drop(songs_by_artist);

        self.refresh_components();
    }

    pub fn refresh_components(&self) {
        let songs_by_artist = self.songs_by_artist.borrow_mut();
        self.refresh_artist_tree(&songs_by_artist);
        drop(songs_by_artist);

        self.album_tree.set_auto_select_next(false);
        self.album_tree.exec_action(Action::Confirm);
        self.album_tree.set_auto_select_next(true);
    }

    /// TODO: store the song library as Vec<Artist> -> Vec<Album> -> Vec<Song>
    /// and call this "library file to tree view component" or something like that
    fn refresh_artist_tree(&self, songs_by_artist: &RefMut<crate::files::Library>) {
        let mut items = vec![];

        let mut artists: Vec<String> = songs_by_artist.songs_by_artist.keys().cloned().collect();
        artists.sort();

        for artist in artists {
            items.push(AlbumTreeItem::Artist(artist.clone()));

            let mut albums: HashMap<String, Album> = HashMap::new();
            let songs = songs_by_artist.songs_by_artist.get(artist.as_str()).unwrap();

            for song in songs {
                let album_name = song.album.clone().unwrap_or("(album name missing)".to_string());
                albums.entry(album_name.clone())
                    .and_modify(|album| {
                        album.songs.push(song.clone())
                    })
                    .or_insert(Album {
                        artist: artist.clone(),
                        name: album_name,
                        year: song.year.unwrap_or_default(), // TODO: accept Option<year>
                        songs: vec![song.clone()],
                    });
            }

            let mut albums: Vec<Album> = albums.values().cloned().collect();

            albums.sort_by_key(|a| a.year);

            for album in albums {
                items.push(AlbumTreeItem::Album(album));
            }
        }

        self.album_tree.set_items(items);
    }

}

impl Drop for Library<'_> {
    fn drop(&mut self) {
        log::trace!("Library.drop()");
    }
}
