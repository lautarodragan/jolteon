use std::sync::atomic::Ordering;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{
    structs::Song,
    ui::KeyboardHandlerRef,
};

use super::component::SongList;

impl<'a> KeyboardHandlerRef<'a> for SongList<'a> {

    fn on_key(&self, key: KeyEvent) -> bool {
        let target = "::SongList.on_key";
        log::trace!(target: target, "{:?}", key);

        match key.code {
            KeyCode::Up | KeyCode::Down | KeyCode::Home | KeyCode::End => {
                self.on_song_list_directional_key(key);
            },
            KeyCode::Enter | KeyCode::Char(_) => {
                let songs = self.songs.lock().unwrap();

                let i = self.selected_song_index.load(Ordering::SeqCst);
                if i >= songs.len() {
                    log::error!(target: target, "library on_key_event_song_list enter: selected_song_index > song_list.len");
                    return true;
                }
                let song = songs[self.selected_song_index.load(Ordering::SeqCst)].clone();
                drop(songs);
                self.on_select_fn.lock().unwrap()((song, key));
            },
            _ => {},
        }

        true
    }
}


impl<'a> SongList<'a> {

    fn on_song_list_directional_key(&self, key: KeyEvent) {
        let songs = self.songs.lock().unwrap();
        let length = songs.len() as i32;

        let height = self.height.load(Ordering::Relaxed) as i32;
        let padding = 5;

        let mut offset = self.offset.load(Ordering::SeqCst) as i32;
        let mut i = self.selected_song_index.load(Ordering::SeqCst) as i32;

        match key.code {
            KeyCode::Up | KeyCode::Down => {
                if key.modifiers == KeyModifiers::NONE {
                    if key.code == KeyCode::Up {
                        i -= 1;
                    } else {
                        i += 1;
                    }
                } else if key.modifiers == KeyModifiers::ALT {
                    if let Some(next) = next_index_by_album(&*songs, i, key.code) {
                        i = next as i32;
                    }
                } else {
                    return;
                }

                let padding = if key.code == KeyCode::Up {
                    padding
                } else {
                    height.saturating_sub(padding).saturating_sub(1)
                };

                if (key.code == KeyCode::Up && i < offset + padding) || (key.code == KeyCode::Down && i > offset + padding) {
                    offset = if i > padding {
                        i - padding
                    } else {
                        0
                    };
                }

            },
            KeyCode::Home => {
                i = 0;
                offset = 0;
            },
            KeyCode::End => {
                i = length - 1;
                offset = i - height + padding;
            },
            _ => {},
        }

        offset = offset.min(length - height).max(0);
        i = i.min(length - 1).max(0);

        self.offset.store(offset as usize, Ordering::SeqCst);
        self.selected_song_index.store(i as usize, Ordering::SeqCst);
    }

}

fn next_index_by_album(songs: &Vec<Song>, i: i32, key: KeyCode) -> Option<usize> {
    let Some(song) = (*songs).get(i as usize) else {
        log::error!("no selected song");
        return None;
    };

    let Some(ref selected_album) = song.album else {
        log::warn!("no selected song album");
        return None;
    };

    let next_song_index = if key == KeyCode::Down {
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
