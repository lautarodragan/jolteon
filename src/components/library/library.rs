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

use super::album_tree_item::{Album, AlbumTreeItem, Artist};

pub struct Library<'a> {
    #[allow(dead_code)]
    pub(super) theme: Theme,

    pub(super) song_tree: Rc<RefCell<Vec<Artist>>>,

    pub(super) song_list: Rc<List<'a, Song>>,
    pub(super) album_tree: Rc<List<'a, AlbumTreeItem>>,
    pub(super) focus_group: FocusGroup<'a>,

    pub(super) on_select_songs_fn: Rc<RefCell<Box<dyn FnMut(Vec<Song>) + 'a>>>,
}

fn library_file_to_song_tree(songs_by_artist: crate::files::Library) -> Vec<Artist> {
    let mut artists: Vec<Artist> = songs_by_artist.songs_by_artist.into_iter().map(|(name, songs)| {
        let mut albums: HashMap<String, Album> = HashMap::new();

        for song in songs {
            let album_name = song.album.clone().unwrap_or("(album name missing)".to_string());

            if let Some(album) = albums.get_mut(album_name.as_str()) {
                album.songs.push(song)
            } else {
                albums.insert(album_name.clone(), Album {
                    artist: name.clone(),
                    name: album_name,
                    year: song.year,
                    songs: vec![song],
                });
            }
        }

        let mut albums: Vec<Album> = albums.values().cloned().collect();
        albums.sort_by_key(|a| a.year);

        Artist { name: name.clone(), albums }
    }).collect();

    artists.sort_by_key(|a| a.name.clone());
    artists
}

fn song_tree_to_album_tree_item_vec(song_tree: Vec<Artist>) -> Vec<AlbumTreeItem> {
    let mut album_tree_items = vec![];

    for artist in &*song_tree {
        album_tree_items.push(AlbumTreeItem::Artist(Artist {
            name: artist.name.clone(),
            albums: artist.albums.clone(),
        }));

        for album in &artist.albums {
            album_tree_items.push(AlbumTreeItem::Album(album.clone()))
        }
    }

    album_tree_items
}

impl<'a> Library<'a> {
    pub fn new(theme: Theme) -> Self {
        let on_select_songs_fn: Rc<RefCell<Box<dyn FnMut(Vec<Song>) + 'a>>> =
            Rc::new(RefCell::new(Box::new(|_| {}) as _));

        let song_tree = library_file_to_song_tree(crate::files::Library::from_file());
        let album_tree_items = song_tree_to_album_tree_item_vec(song_tree.clone());
        let mut album_tree = List::new(theme, album_tree_items);

        let song_list = Rc::new(List::new(theme, vec![]));
        let song_tree = Rc::new(RefCell::new(song_tree));

        album_tree.on_select({
            let song_list = song_list.clone();

            move |item| {
                log::trace!(target: "::library.album_tree.on_select", "selected {:?}", item);

                let songs = match item {
                    AlbumTreeItem::Artist(artist) => {
                        artist.albums.iter().flat_map(|album| album.songs.clone()).collect()
                    }
                    AlbumTreeItem::Album(album) => {
                        album.songs.clone()
                    }
                };
                song_list.set_items(songs);
            }
        });

        album_tree.on_enter({
            let on_select_songs_fn = on_select_songs_fn.clone();

            move |item| {
                log::trace!(target: "::library.album_tree.on_confirm", "artist confirmed {:?}", item);

                let songs = match item {
                    AlbumTreeItem::Artist(artist) => artist.albums.iter().flat_map(|album| album.songs.clone()).collect(),
                    AlbumTreeItem::Album(album) => album.songs,
                };
                on_select_songs_fn.borrow_mut()(songs);
            }
        });

        album_tree.on_delete({
            let song_tree = song_tree.clone();

            move |item, _index| {
                log::trace!(target: "::library.album_tree.on_delete", "item deleted {:?}", item);

                let mut song_tree = song_tree.borrow_mut();
                match item {
                    AlbumTreeItem::Artist(ref artist) => {
                        let Some(i) = song_tree.iter().position(|a| a.name == artist.name) else {
                            log::error!("Tried to remove an artist that does not exist. {artist:?}");
                            return;
                        };
                        song_tree.remove(i);
                    }
                    AlbumTreeItem::Album(album) => {
                        let Some(i) = song_tree.iter().position(|a| a.name == album.artist) else {
                            log::error!("Tried to remove an album of an artist that does not exist. {album:?}");
                            return;
                        };
                        song_tree[i].albums.retain(|a| {
                            a.name != album.name
                        });
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

        album_tree.set_auto_select_next(false);
        album_tree.exec_action(Action::Confirm);
        album_tree.set_auto_select_next(true);

        let album_tree = Rc::new(album_tree);
        let focus_group = FocusGroup::new(vec![album_tree.clone(), song_list.clone()]);

        Self {
            theme,
            focus_group,

            on_select_songs_fn,

            song_tree,
            song_list,
            album_tree,
        }
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
        // let mut songs_by_artist = self.songs_by_artist.borrow_mut();
        // songs_by_artist.add_songs(songs_to_add);
        // drop(songs_by_artist);
        //
        // self.refresh_components();
    }
}

impl Drop for Library<'_> {
    fn drop(&mut self) {
        log::trace!("Library.drop()");
    }
}
