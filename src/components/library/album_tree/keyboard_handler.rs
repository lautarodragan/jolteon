use std::sync::atomic::Ordering;

use crossterm::event::{KeyCode, KeyEvent};

use crate::ui::KeyboardHandlerRef;

use super::{album_tree_item::AlbumTreeItem, component::AlbumTree};

impl<'a> KeyboardHandlerRef<'a> for AlbumTree<'a> {
    fn on_key(&self, key: KeyEvent) {
        let target = "::ArtistList.on_key";
        log::trace!(target: target, "{:?}", key);

        match key.code {
            KeyCode::Up | KeyCode::Down | KeyCode::Home | KeyCode::End => {
                self.filter.lock().unwrap().clear(); // todo: same as file browser - JUMP TO NEXT MATCH
                self.on_artist_list_directional_key(key);
                let Some(item) = self.selected_item() else {
                    log::warn!(target: target, "No selected item");
                    return;
                };
                self.on_select_fn.lock().unwrap()(item);
            }
            KeyCode::Enter => {
                let Some(item) = self.selected_item() else {
                    log::warn!(target: target, "No selected item");
                    return;
                };
                self.on_confirm_fn.lock().unwrap()(item);
            }
            KeyCode::Char(' ') => {
                let selected_artist = self.selected_artist.load(Ordering::SeqCst);
                let mut artist_list = self.artist_list.lock().unwrap();
                let selected_artist = &mut artist_list[selected_artist];
                selected_artist.is_open = !selected_artist.is_open;

                self.selected_album.store(0, Ordering::SeqCst);

                if !selected_artist.is_open {
                    let item = AlbumTreeItem::Artist(selected_artist.artist.clone());
                    self.on_select_fn.lock().unwrap()(item);
                } else {
                    let album = selected_artist.albums.first().unwrap();
                    let item = AlbumTreeItem::Album(selected_artist.artist.clone(), album.clone());
                    self.on_select_fn.lock().unwrap()(item);
                }
            }
            KeyCode::Delete => {
                let selected_artist = self.selected_artist.load(Ordering::SeqCst);
                let selected_album = self.selected_album.load(Ordering::SeqCst);

                let mut artists = self.artist_list.lock().unwrap();
                let Some(artist) = artists.get_mut(selected_artist) else {
                    log::error!("selected_artist {selected_artist} >= len {}", artists.len());
                    return;
                };

                let removed_item;
                let newly_selected_item;
                let newly_selected_album_index;

                if artist.is_open && artist.albums.len() > 1 {
                    let removed_album = artist.albums.remove(selected_album);
                    removed_item = AlbumTreeItem::Album(artist.artist.clone(), removed_album);

                    newly_selected_album_index = selected_album.min(artist.albums.len().saturating_sub(1));
                    newly_selected_item =
                        AlbumTreeItem::Album(artist.artist.clone(), artist.albums[newly_selected_album_index].clone());
                } else {
                    let removed_artist = artists.remove(selected_artist);
                    removed_item = AlbumTreeItem::Artist(removed_artist.artist);

                    let newly_selected_artist_index = selected_artist.saturating_sub(1);
                    let newly_selected_artist = artists.get(newly_selected_artist_index).unwrap();

                    self.selected_artist
                        .store(newly_selected_artist_index, Ordering::SeqCst);

                    if newly_selected_artist.is_open {
                        newly_selected_album_index = newly_selected_artist.albums.len().saturating_sub(1);
                        newly_selected_item = AlbumTreeItem::Album(
                            newly_selected_artist.artist.clone(),
                            newly_selected_artist.albums[newly_selected_album_index].clone(),
                        );
                    } else {
                        newly_selected_album_index = 0;
                        newly_selected_item = AlbumTreeItem::Artist(newly_selected_artist.artist.clone());
                    }
                };
                self.selected_album.store(newly_selected_album_index, Ordering::SeqCst);

                // TODO: may need to re-calculate the offset,
                // self.offset.store(0, Ordering::SeqCst); // TODO: fix me (same as "select one above")

                drop(artists);

                self.on_delete_fn.lock().unwrap()(removed_item);
                self.on_select_fn.lock().unwrap()(newly_selected_item);
            }
            KeyCode::Char(char) => {
                let item = {
                    let mut filter = self.filter.lock().unwrap();
                    filter.push(char);

                    // todo: also search albums
                    let mut artists = self.artist_list.lock().unwrap();

                    for i in 0..artists.len() {
                        let entry = &mut artists[i];
                        entry.is_match = entry.artist.contains(filter.as_str())
                            || entry.artist.to_lowercase().contains(filter.to_lowercase().as_str());
                    }

                    let selected_artist_index = self.selected_artist.load(Ordering::SeqCst);
                    let selected_artist = &artists[selected_artist_index];

                    if !selected_artist.is_match {
                        artists
                            .iter()
                            .position(|entry| entry.is_match)
                            .map(|n| (AlbumTreeItem::Artist(artists[n].artist.clone()), n))
                    } else {
                        None
                    }
                };

                if let Some((item, n)) = item {
                    self.selected_artist.store(n, Ordering::SeqCst);
                    self.on_select_fn.lock().unwrap()(item);
                }
            }
            KeyCode::Esc => {
                let mut filter = self.filter.lock().unwrap();
                filter.clear();

                let mut artists = self.artist_list.lock().unwrap();
                for i in 0..artists.len() {
                    let entry = &mut artists[i];
                    entry.is_match = false;
                }
            }
            _ => {}
        }
    }
}

impl AlbumTree<'_> {
    fn on_artist_list_directional_key(&self, key: KeyEvent) {
        let artists = self.artist_list.lock().unwrap();
        let length = {
            let visible_albums: usize = artists.iter().filter(|a| a.is_open).map(|a| a.albums.len()).sum();
            visible_albums + artists.len()
        } as i32;

        let height = self.height.load(Ordering::Relaxed) as i32;
        let padding = 5;

        let mut offset = self.offset.load(Ordering::SeqCst) as i32;
        let mut i = self.selected_artist.load(Ordering::SeqCst) as i32;
        let mut j = self.selected_album.load(Ordering::SeqCst) as i32;

        match key.code {
            KeyCode::Up | KeyCode::Down => {
                let artist = artists.get(i.max(0) as usize).unwrap();

                if key.code == KeyCode::Up {
                    if artist.is_open && j > 0 {
                        j -= 1;
                    } else if i > 0 {
                        i -= 1;
                        let artist = artists.get(i.max(0) as usize).unwrap();
                        j = if artist.is_open {
                            artist.albums.len().saturating_sub(1) as i32
                        } else {
                            0
                        };
                    }
                } else if artist.is_open {
                    if j < artist.albums.len().saturating_sub(1) as i32 {
                        j += 1;
                    } else if i < artists.len().saturating_sub(1) as i32 {
                        j = 0;
                        i += 1;
                    }
                } else {
                    j = 0;
                    i += 1;
                }

                let padding = if key.code == KeyCode::Up {
                    padding
                } else {
                    height.saturating_sub(padding).saturating_sub(1)
                };

                let visible_items: usize = artists
                    .iter()
                    .take(i as usize)
                    .filter(|a| a.is_open)
                    .map(|a| a.albums.len())
                    .sum();
                let visible_items = visible_items as i32 + i + j;

                if (key.code == KeyCode::Up && visible_items < offset + padding + 1)
                    || (key.code == KeyCode::Down && visible_items > offset + padding)
                {
                    offset = if visible_items > padding {
                        visible_items - padding
                    } else {
                        0
                    };
                }
            }
            KeyCode::Home => {
                i = 0;
                j = 0;
                offset = 0;
            }
            KeyCode::End => {
                i = artists.len() as i32 - 1;
                let artist = artists.get(i.max(0) as usize).unwrap();
                j = if artist.is_open {
                    artist.albums.len().saturating_sub(1)
                } else {
                    0
                } as i32;
                offset = length - 1 - height + padding;
            }
            _ => {}
        }

        offset = offset.min(length - height).max(0);
        i = i.min(artists.len() as i32 - 1).max(0);

        self.offset.store(offset as usize, Ordering::SeqCst);
        self.selected_artist.store(i as usize, Ordering::SeqCst);
        self.selected_album.store(j as usize, Ordering::SeqCst);
    }
}
