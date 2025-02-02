use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    components::{FocusGroup, List, Tree, TreeNode},
    config::Theme,
    structs::{Direction, Song},
    ui::{Component, Focusable},
};

use super::album_tree_item::{Album, AlbumTreeItem, Artist};

pub struct Library<'a> {
    #[allow(dead_code)]
    pub(super) theme: Theme,

    pub(super) song_tree: Rc<RefCell<Vec<Artist>>>,

    pub(super) song_list: Rc<List<'a, Song>>,
    pub(super) album_tree: Rc<RefCell<Tree<'a, AlbumTreeItem>>>,
    pub(super) focus_group: FocusGroup<'a>,

    pub(super) on_select_songs_fn: Rc<RefCell<Box<dyn FnMut(Vec<&Song>) + 'a>>>,
}

/// TODO: refactor crate::files::Library to store Vec<Artist> and delete this
fn library_file_to_song_tree(songs_by_artist: crate::files::Library) -> Vec<Artist> {
    let mut artists: Vec<Artist> = songs_by_artist
        .songs_by_artist
        .into_iter()
        .map(|(name, songs)| {
            let mut albums: HashMap<String, Album> = HashMap::new();

            for song in songs {
                let album_name = song.album.clone().unwrap_or("(album name missing)".to_string());

                if let Some(album) = albums.get_mut(album_name.as_str()) {
                    album.songs.push(song)
                } else {
                    albums.insert(
                        album_name.clone(),
                        Album {
                            artist: name.clone(),
                            name: album_name,
                            year: song.year,
                            songs: vec![song],
                        },
                    );
                }
            }

            let mut albums: Vec<Album> = albums.into_values().collect();
            albums.sort_unstable_by_key(|a| a.year);

            Artist {
                name: name.clone(),
                albums,
            }
        })
        .collect();

    artists.sort_by_key(|a| a.name.clone());
    artists
}

impl<'a> Library<'a> {
    pub fn new(theme: Theme) -> Self {
        let on_select_songs_fn: Rc<RefCell<Box<dyn FnMut(Vec<&Song>) + 'a>>> = Rc::new(RefCell::new(Box::new(|_| {})));

        let song_tree = library_file_to_song_tree(crate::files::Library::from_file());

        let album_tree_items: Vec<TreeNode<AlbumTreeItem>> = song_tree
            .iter()
            .cloned()
            .map(|mut artist| {
                let albums = std::mem::take(&mut artist.albums);
                let mut tree_node = TreeNode::new(AlbumTreeItem::Artist(artist));

                tree_node.children = albums
                    .into_iter()
                    .map(|album| TreeNode::new(AlbumTreeItem::Album(album)))
                    .collect();

                tree_node
            })
            .collect();
        let mut album_tree = Tree::new(theme, album_tree_items);

        // album_tree.set_is_visible_magic(|item| item.is_artist());
        album_tree.set_is_open_all(false);

        let mut song_list = List::new(
            theme,
            song_tree.first().map(|artist| artist.songs()).unwrap_or_default(),
        );
        let song_tree = Rc::new(RefCell::new(song_tree));

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
        song_list.on_select(|song| log::debug!("library: selected song {song:#?}"));

        let song_list = Rc::new(song_list);

        album_tree.on_select({
            let song_list = song_list.clone();

            move |item| {
                // log::trace!(target: "::library.album_tree.on_select", "selected {:#?}", item);

                let songs = match &item.inner {
                    AlbumTreeItem::Artist(_) => item.children.iter().flat_map(|child| child.inner.songs()).collect(),
                    AlbumTreeItem::Album(album) => album.songs.clone(),
                };
                song_list.set_items(songs.clone());
            }
        });
        album_tree.on_confirm({
            let on_select_songs_fn = on_select_songs_fn.clone();

            move |item| {
                log::trace!(target: "::library.album_tree.on_confirm", "artist confirmed {:?}", item);

                let songs = match item {
                    AlbumTreeItem::Artist(artist) => artist.albums.iter().flat_map(|album| &album.songs).collect(),
                    AlbumTreeItem::Album(album) => album.songs.iter().collect(),
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
                    AlbumTreeItem::Artist(artist) => {
                        let Some(i) = song_tree.iter().position(|a| a.name == artist.name) else {
                            log::error!("Tried to remove an artist that does not exist. {artist:?}");
                            return;
                        };
                        song_tree.remove(i);
                        // TODO: save changes
                        // TODO: delete artist's albums (or make list a proper tree view component?)
                        //   let album_tree_items = song_tree_to_album_tree_item_vec(song_tree.clone());
                        //   self.album_tree.set_items(album_tree_items);
                    }
                    AlbumTreeItem::Album(album) => {
                        let Some(i) = song_tree.iter().position(|a| a.name == album.artist) else {
                            log::error!("Tried to remove an album of an artist that does not exist. {album:?}");
                            return;
                        };
                        song_tree[i].albums.retain(|a| a.name != album.name);
                        // TODO: save changes
                        // TODO: mutate artist entry - remove album (or make list a proper tree view component?)
                        //   let album_tree_items = song_tree_to_album_tree_item_vec(song_tree.clone());
                        //   self.album_tree.set_items(album_tree_items);
                    }
                };
            }
        });
        album_tree.on_reorder({
            move |parent_path, old_index, new_index| {
                log::debug!("album_tree.on_reorder({parent_path}, {old_index}, {new_index})")
            }
        });

        let album_tree = Rc::new(RefCell::new(album_tree));

        let focus_group = FocusGroup::new(vec![
            Component::Mut(album_tree.clone()),
            Component::Ref(song_list.clone()),
        ]);

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
        self.song_list.on_confirm(cb);
    }

    pub fn on_enter_alt(&self, cb: impl Fn(Song) + 'a) {
        self.song_list.on_confirm_alt(cb);
    }

    pub fn on_select_songs_fn(&self, cb: impl FnMut(Vec<&Song>) + 'a) {
        *self.on_select_songs_fn.borrow_mut() = Box::new(cb);
    }

    pub fn add_songs(&self, songs: Vec<Song>) {
        log::debug!(
            "Library.add_songs({:?})",
            songs.iter().map(|s| s.title.as_str()).collect::<Vec<&str>>()
        );
        let mut song_tree = self.song_tree.borrow_mut();

        for song in songs {
            #[rustfmt::skip]
            if let Some(ref artist) = song.artist && let Some(ref album) = song.album {
                if let Some(artist) = song_tree.iter_mut().find(|i| i.name == *artist) {
                    if let Some(album) = artist.albums.iter_mut().find(|i| i.name == *album) {
                        if !album.songs.contains(&song) {
                            album.songs.push(song);
                        } else {
                            log::info!("Library.add_songs: song already in library {song:#?}");
                        }
                    } else {
                        artist.albums.push(Album {
                            artist: artist.name.clone(),
                            name: album.clone(),
                            year: song.year,
                            songs: vec![song],
                        })
                    }
                    artist.albums.sort_unstable_by_key(|a| a.year);
                } else {
                    song_tree.push(Artist {
                        name: artist.to_string(),
                        albums: vec![Album {
                            artist: artist.to_string(),
                            name: album.to_string(),
                            year: song.year,
                            songs: vec![song],
                        }],
                    });
                }
            } else {
                log::warn!("Library.add_songs: ignoring song due to missing artist or album in song {song:#?}");
            }
        }

        // let album_tree_items = song_tree_to_album_tree_item_vec(song_tree.clone()); // TODO: optimize. this is extremely wasteful!
        let album_tree_items: Vec<TreeNode<AlbumTreeItem>> = song_tree
            .iter()
            .cloned()
            .map(|mut artist| {
                let albums = std::mem::take(&mut artist.albums);
                let mut tree_node = TreeNode::new(AlbumTreeItem::Artist(artist));

                tree_node.children = albums
                    .into_iter()
                    .map(|album| TreeNode::new(AlbumTreeItem::Album(album)))
                    .collect();

                tree_node
            })
            .collect();
        self.album_tree.borrow_mut().set_items(album_tree_items);

        // TODO: save changes
    }
}

impl Drop for Library<'_> {
    fn drop(&mut self) {
        log::trace!("Library.drop()");
    }
}

impl Focusable for Library<'_> {}
