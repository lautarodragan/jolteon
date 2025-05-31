use std::{
    cell::RefCell,
    collections::HashMap,
    fs::{read_to_string, write},
    rc::Rc,
};

use uuid::Uuid;

use super::album_tree_item::{Album, AlbumTreeItem, Artist};
use crate::{
    components::{FocusGroup, List, Tree, TreeNode},
    theme::Theme,
    structs::{Direction, Song},
    ui::{Component, Focusable},
};

pub struct Library<'a> {
    #[allow(dead_code)]
    pub(super) theme: Theme,

    pub(super) song_list: Rc<List<'a, Song>>,
    pub(super) album_tree: Rc<RefCell<Tree<'a, AlbumTreeItem>>>,
    pub(super) focus_group: FocusGroup<'a>,

    pub(super) on_select_songs_fn: Rc<RefCell<Box<dyn FnMut(Vec<&Song>) + 'a>>>,
}

impl<'a> Library<'a> {
    pub fn new(theme: Theme) -> Self {
        let album_tree_items = load_lib();

        let on_select_songs_fn: Rc<RefCell<Box<dyn FnMut(Vec<&Song>) + 'a>>> = Rc::new(RefCell::new(Box::new(|_| {})));

        let mut song_list = List::new(
            theme,
            album_tree_items
                .first()
                .map(|node| {
                    if let AlbumTreeItem::Artist(ref artist) = node.inner {
                        artist.songs()
                    } else {
                        vec![]
                    }
                })
                .unwrap_or_default(),
        );
        let album_tree = Rc::new(RefCell::new(Tree::new(theme, album_tree_items)));

        let render_song_short =
            |song: &Song| -> String { [song.track.unwrap_or(0).to_string(), song.title.clone()].join(" - ") };

        let render_song_long = |song: &Song| -> String {
            [
                song.year.map(|y| y.to_string()).unwrap_or("(no_album)".to_string()),
                song.album.clone().unwrap_or("(no_album)".to_string()),
                song.track.unwrap_or(0).to_string(),
                song.title.clone(),
            ]
            .join(" - ")
        };

        song_list.render_fn(render_song_long);

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
        song_list.on_delete({
            let album_tree = Rc::downgrade(&album_tree);
            move |song, index| {
                log::debug!(
                    "deleted {}(at {index}) from song_list. will now delete from album_tree.",
                    song.title
                );

                let Some(album_tree) = album_tree.upgrade() else {
                    log::warn!("song_list.on_delete: album_tree is gone");
                    return;
                };

                // the following code is pretty terrible.
                // for it to work correctly, it depends on:
                //   - both lists being sorted in the same way, since we're removing by index
                //   - the song's album name in the song list and the album's name in the album tree matching, since we're comparing strings
                //   - the selection on the album_tree not changing after the song is deleted from the song_list,
                //     and before this callback is called (which is impossible TODAY, but nothing guarantees that)
                // if either isn't true, we'll crash if we're lucky, but, most likely, delete the incorrect thing.
                // TODO: give artists, albums and songs unique ids, and stop relying on indexes and strings.
                //   `album_tree.get_album_by_id_mut(song.album_unique_id) -> Option<&mut Album>`

                let album_tree = album_tree.borrow_mut();
                album_tree.with_selected_node_mut(|selected_node| {
                    match &mut selected_node.inner {
                        AlbumTreeItem::Folder(_category) => {
                            // TODO
                        }
                        AlbumTreeItem::Artist(_) => {
                            let album = selected_node
                                .children
                                .iter_mut()
                                .find_map(|item| match &mut item.inner {
                                    AlbumTreeItem::Album(album) if album.name == *song.album.as_ref().unwrap() => {
                                        Some(album)
                                    }
                                    _ => None,
                                });
                            if let Some(album) = album {
                                log::debug!("deleting from album {}", album.name);
                                album.songs.remove(index);
                            } else {
                                log::error!("couldn't find the song we're trying to delete! this is a bug.");
                            }
                        }
                        AlbumTreeItem::Album(album) => {
                            album.songs.remove(index);
                        }
                    };
                });
            }
        });
        let song_list = Rc::new(song_list);

        {
            let mut album_tree = album_tree.borrow_mut();
            album_tree.on_select({
                let song_list = song_list.clone();

                move |item| {
                    // log::trace!(target: "::library.album_tree.on_select", "selected {:#?}", item);

                    let songs = match &item.inner {
                        AlbumTreeItem::Folder(_category) => {
                            // TODO
                            song_list.render_fn(render_song_long);
                            vec![]
                        }
                        AlbumTreeItem::Artist(_) => {
                            song_list.render_fn(render_song_long);
                            item.children.iter().flat_map(|child| child.inner.songs()).collect()
                        }
                        AlbumTreeItem::Album(album) => {
                            song_list.render_fn(render_song_short);
                            album.songs.clone()
                        }
                    };
                    song_list.set_items(songs.clone());
                }
            });
            album_tree.on_confirm({
                let on_select_songs_fn = on_select_songs_fn.clone();

                move |item| {
                    log::trace!(target: "::library.album_tree.on_confirm", "artist confirmed {:?}", item);

                    let songs = match item {
                        AlbumTreeItem::Folder(_category) => {
                            // TODO
                            vec![]
                        }
                        AlbumTreeItem::Artist(artist) => artist.albums.iter().flat_map(|album| &album.songs).collect(),
                        AlbumTreeItem::Album(album) => album.songs.iter().collect(),
                    };
                    on_select_songs_fn.borrow_mut()(songs);
                }
            });
            album_tree.on_reorder({
                move |parent_path, old_index, new_index| {
                    log::debug!("album_tree.on_reorder({parent_path}, {old_index}, {new_index})")
                }
            });
            album_tree.on_delete({
                |ati, index| {
                    log::debug!("deleted {index} {:?}", ati);
                    // nothing to do here, because the list itself is the source of truth.
                    // but, right now, the tree doesn't allow deletions if there's no on_delete callback,
                    // so we need to pass this callback.
                    // TODO: album_tree.set_allow_deletions(true) or something like that.
                    // we'll still want this callback with the log::debug! anyways, but this refactor would
                    // make the code more intuitive and sensible
                }
            });
            album_tree.on_rename(|new_name| {
                log::debug!("album_tree: renamed to {new_name}");
            });
        }

        let focus_group = FocusGroup::new(vec![
            Component::Mut(album_tree.clone()),
            Component::Ref(song_list.clone()),
        ]);

        Self {
            theme,
            focus_group,

            on_select_songs_fn,

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

    pub fn add_songs(&self, mut songs: Vec<Song>) {
        log::debug!(
            "Library.add_songs({:?})",
            songs.iter().map(|s| s.title.as_str()).collect::<Vec<&str>>()
        );

        for song in &mut songs {
            song.library_id = Some(Uuid::new_v4());
        }
        let songs = song_vec_to_map(songs);

        self.album_tree.borrow_mut().with_nodes_mut(|artist_nodes| {
            for (artist, albums) in songs.into_iter() {
                if let Some(artist_node) = find_artist_node(artist_nodes, artist.as_str()) {
                    add_album_nodes(artist_node, artist, albums);
                } else {
                    add_artist_node(artist_nodes, artist, albums);
                }
            }

            save_lib(artist_nodes);
        });
    }
}

impl Drop for Library<'_> {
    fn drop(&mut self) {
        log::trace!("Library.drop()");
        self.album_tree.borrow_mut().with_nodes_mut(|nodes| {
            log::trace!("Library.drop() -> saving lib");
            save_lib(nodes)
        });
    }
}

fn song_vec_to_map(songs: Vec<Song>) -> HashMap<String, HashMap<String, Vec<Song>>> {
    let mut artist_album_map: HashMap<String, HashMap<String, Vec<Song>>> = HashMap::new();

    for song in songs.into_iter() {
        let (Some(artist), Some(album)) = (song.artist.clone(), song.album.clone()) else {
            log::warn!("Missing data for song {song:?}. Cannot add to library.");
            continue;
        };
        if let Some(artist_entry) = artist_album_map.get_mut(artist.as_str()) {
            if let Some(album_entry) = artist_entry.get_mut(album.as_str()) {
                album_entry.push(song);
            } else {
                artist_entry.insert(album, vec![song]);
            }
        } else {
            artist_album_map.insert(artist, HashMap::from([(album, vec![song])]));
        }
    }

    artist_album_map
}

fn find_artist_node<'a>(
    artist_nodes: &'a mut [TreeNode<AlbumTreeItem>],
    artist_name: &str,
) -> Option<&'a mut TreeNode<AlbumTreeItem>> {
    artist_nodes
        .iter_mut()
        .find_map(|artist_node| match &artist_node.inner {
            AlbumTreeItem::Artist(node_artist) if node_artist.name == artist_name => Some(artist_node),
            _ => None,
        })
}

fn add_artist_node(
    artist_nodes: &mut Vec<TreeNode<AlbumTreeItem>>,
    artist: String,
    albums: HashMap<String, Vec<Song>>,
) {
    artist_nodes.push({
        TreeNode::new_with_children(
            AlbumTreeItem::Artist(Artist {
                library_id: Some(Uuid::new_v4()),
                name: artist.clone(),
                albums: vec![], // the albums are stored as children nodes
            }),
            albums
                .into_iter()
                .map(|(album_name, album_songs)| {
                    TreeNode::new(AlbumTreeItem::Album(Album {
                        library_id: Some(Uuid::new_v4()),
                        artist: artist.clone(),
                        name: album_name,
                        year: album_songs.first().and_then(|s| s.year),
                        songs: album_songs,
                    }))
                })
                .collect(),
        )
    });
}

fn find_album<'a>(artist_node: &'a mut TreeNode<AlbumTreeItem>, album_name: &str) -> Option<&'a mut Album> {
    artist_node
        .children
        .iter_mut()
        .find_map(|album_node| match &mut album_node.inner {
            AlbumTreeItem::Album(album) if album.name == album_name => Some(album),
            _ => None,
        })
}

fn add_album_nodes(artist_node: &mut TreeNode<AlbumTreeItem>, artist_name: String, albums: HashMap<String, Vec<Song>>) {
    for (album_name, songs) in albums.into_iter() {
        if let Some(album) = find_album(artist_node, album_name.as_str()) {
            album.songs.extend(songs);
        } else {
            artist_node.children.push(TreeNode::new(AlbumTreeItem::Album(Album {
                library_id: Some(Uuid::new_v4()),
                artist: artist_name.clone(),
                name: album_name,
                year: songs.iter().find_map(|song| song.year),
                songs,
            })))
        }
    }
}

impl Focusable for Library<'_> {}

fn load_lib() -> Vec<TreeNode<AlbumTreeItem>> {
    let path = home::home_dir()
        .map(|path| path.as_path().join(".config/jolteon/library.json"))
        .unwrap();
    let string = match read_to_string(&path) {
        Ok(a) => a,
        Err(e) => {
            log::error!("read_to_string error {e:?}");
            return vec![];
        }
    };
    match serde_json::from_str::<Vec<TreeNode<AlbumTreeItem>>>(&string) {
        Ok(a) => a,
        Err(e) => {
            log::error!("from_str error {e:?}");
            vec![]
        }
    }
}

fn save_lib(nodes: &Vec<TreeNode<AlbumTreeItem>>) {
    log::trace!("Library save_lib");
    let path = home::home_dir()
        .map(|path| path.as_path().join(".config/jolteon/library.json"))
        .unwrap();

    let string = match serde_json::to_string_pretty(nodes) {
        Ok(a) => a,
        Err(e) => {
            log::error!("from_str error {e:?}");
            return;
        }
    };
    match write(&path, &string) {
        Ok(_) => {}
        Err(e) => {
            log::error!("write error {e:?}");
        }
    };
}
