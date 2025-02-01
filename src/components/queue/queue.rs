use std::{cell::Cell, time::Duration};

use crate::{
    components::List,
    config::Theme,
    structs::Song,
    ui::{Component, Focusable},
};

pub struct Queue<'a> {
    pub(super) song_list: List<'a, Song>,
    duration: Cell<Duration>,
}

impl<'a> Queue<'a> {
    pub fn new(songs: Vec<Song>, theme: Theme) -> Self {
        let song_list = List::new(theme, songs);

        Self {
            song_list,
            duration: Cell::new(Duration::default()),
        }
    }

    pub fn len(&self) -> usize {
        self.song_list.with_items(|items| items.len())
    }

    fn refresh_duration(&self) {
        self.duration
            .set(self.song_list.with_items(|items| items.iter().map(|s| s.length).sum()));
    }

    pub fn duration(&self) -> Duration {
        self.duration.get()
    }

    pub fn set_items(&self, items: Vec<Song>) {
        self.song_list.set_items_k(items);
        self.refresh_duration();
    }

    pub fn append(&self, songs: Vec<Song>) {
        self.song_list.append_items(songs);
        self.refresh_duration();
    }

    pub fn on_enter(&self, cb: impl Fn(Song) + 'a) {
        self.song_list.on_enter(cb);
    }

    pub fn on_delete(&self, cb: impl Fn(Song, usize) + 'a) {
        self.song_list.on_delete(cb);
    }
}

impl Drop for Queue<'_> {
    fn drop(&mut self) {
        log::trace!("QueueUi drop");
    }
}

impl Focusable for Queue<'_> {
    fn set_is_focused(&self, v: bool) {
        todo!()
    }

    fn is_focused(&self) -> bool {
        todo!()
    }
}
