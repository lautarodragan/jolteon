use crate::{components::List, config::Theme, structs::Song};

pub struct Queue<'a> {
    pub(super) song_list: List<'a, Song>,
}

impl<'a> Queue<'a> {
    pub fn new(songs: Vec<Song>, theme: Theme) -> Self {
        let song_list = List::new(theme, songs);

        Self { song_list }
    }

    pub fn len(&self) -> usize {
        self.song_list.with_items(|items| items.len())
    }

    pub fn set_items(&self, items: Vec<Song>) {
        self.song_list.set_items_k(items);
    }

    pub fn on_enter(&self, cb: impl Fn(Song) + 'a) {
        self.song_list.on_enter(cb);
    }

    pub fn on_delete(&self, cb: impl Fn(Song, usize) + 'a) {
        self.song_list.on_delete(cb);
    }

    pub fn append(&self, songs: Vec<Song>) {
        self.song_list.append_items(songs);
    }
}

impl Drop for Queue<'_> {
    fn drop(&mut self) {
        log::trace!("QueueUi drop");
    }
}
