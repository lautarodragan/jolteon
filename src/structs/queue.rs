use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicU64, AtomicUsize, Ordering},
        Arc, Mutex, MutexGuard,
    },
    time::Duration,
};

use crate::structs::Song;

pub struct Queue {
    songs: Arc<Mutex<VecDeque<Song>>>,

    queue_length: AtomicUsize,
    total_time: AtomicU64,

    on_queue_changed: Arc<Mutex<Option<Box<dyn Fn() + Send + 'static>>>>,
}

fn song_list_to_duration(items: &VecDeque<Song>) -> Duration {
    items.iter().map(|s| s.length).sum()
}

impl Queue {
    pub fn new(songs: Vec<Song>) -> Self {
        let songs = VecDeque::from(songs);
        let queue_length = AtomicUsize::new(songs.len());
        let total_time = song_list_to_duration(&songs);

        Self {
            songs: Arc::new(Mutex::new(songs)),

            queue_length,
            total_time: AtomicU64::new(total_time.as_secs()),

            on_queue_changed: Arc::new(Mutex::new(None)),
        }
    }

    pub fn on_queue_changed(&self, f: impl Fn() + Send + 'static) {
        *self.on_queue_changed.lock().unwrap() = Some(Box::new(f));
    }

    pub fn pop(&self) -> Option<Song> {
        let target = "::queue.pop()";

        let mut items = self.songs();

        let song = items.pop_front();

        if let Some(ref song) = song {
            log::trace!(target: target, "Got song {:?}", song.title);
            self.queue_length.fetch_sub(1, Ordering::SeqCst);
            self.set_total_time(song_list_to_duration(&items).as_secs());
        }

        song
    }

    pub fn with_items(&self, f: impl FnOnce(&VecDeque<Song>)) {
        let songs = self.songs();
        f(&songs);
    }

    fn mut_queue(&self, f: impl FnOnce(&mut VecDeque<Song>)) {
        log::trace!(target: "::queue.mut_queue", "acquiring lock on songs");
        let mut songs = self.songs();

        f(&mut songs);

        self.queue_length.store(songs.len(), Ordering::SeqCst);
        self.set_total_time(song_list_to_duration(&songs).as_secs());

        if let Some(on_queue_changed) = &*self.on_queue_changed.lock().unwrap() {
            on_queue_changed();
        }
    }

    pub fn songs(&self) -> MutexGuard<VecDeque<Song>> {
        self.songs.lock().unwrap()
    }

    pub fn length(&self) -> usize {
        self.queue_length.load(Ordering::SeqCst)
    }

    pub fn total_time(&self) -> Duration {
        Duration::new(self.total_time.load(Ordering::SeqCst), 0)
    }

    fn set_total_time(&self, seconds: u64) {
        self.total_time.store(seconds, Ordering::SeqCst);
    }

    pub fn add_front(&self, song: Song) {
        self.mut_queue(|queue_songs| {
            queue_songs.push_front(song);
        });
    }

    pub fn add_back(&self, song: Song) {
        self.mut_queue(|queue_songs| {
            queue_songs.push_back(song);
        });
    }

    pub fn append(&self, songs: &mut VecDeque<Song>) {
        self.mut_queue(|queue_songs| {
            queue_songs.append(songs);
        });
    }

    pub fn remove(&self, index: usize) {
        self.mut_queue(|queue_songs| {
            queue_songs.remove(index);
        });
    }
}

impl Drop for Queue {
    fn drop(&mut self) {
        log::trace!("Player.Queue drop");
    }
}
