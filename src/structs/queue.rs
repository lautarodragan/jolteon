use std::{
    collections::VecDeque,
    sync::{Arc, Mutex, MutexGuard},
};

use crate::structs::Song;

pub struct Queue {
    songs: Arc<Mutex<VecDeque<Song>>>,
}

impl Queue {
    pub fn new(songs: Vec<Song>) -> Self {
        let songs = VecDeque::from(songs);

        Self {
            songs: Arc::new(Mutex::new(songs)),
        }
    }

    pub fn pop(&self) -> Option<Song> {
        let target = "::queue.pop()";

        let mut items = self.songs();

        let song = items.pop_front();

        if let Some(ref song) = song {
            log::trace!(target: target, "Got song {:?}", song.title);
        }

        song
    }

    pub fn with_items(&self, f: impl FnOnce(&VecDeque<Song>)) {
        let songs = self.songs();
        f(&songs);
    }

    pub fn songs(&self) -> MutexGuard<VecDeque<Song>> {
        self.songs.lock().unwrap()
    }

    pub fn add_front(&self, song: Song) {
        let mut songs = self.songs();
        songs.push_front(song);
    }

    pub fn add_back(&self, song: Song) {
        let mut songs = self.songs();
        songs.push_back(song);
    }

    pub fn append(&self, songs: &mut VecDeque<Song>) {
        let mut queue_songs = self.songs();
        queue_songs.append(songs);
    }

    pub fn remove(&self, index: usize) {
        let mut songs = self.songs();
        songs.remove(index);
    }
}

impl Drop for Queue {
    fn drop(&mut self) {
        log::trace!("Player.Queue drop");
    }
}
