use std::{
    collections::VecDeque,
    sync::{
        Arc,
        Mutex,
        mpsc::{Sender, channel},
    },
    thread,
    thread::JoinHandle,
    time::Duration,
};

use crate::{
    actions::{OnAction, PlayerAction},
    mpris::Mpris,
    player::SingleTrackPlayer,
    structs::{Queue, Song},
};

#[derive(Debug)]
enum MainPlayerCommand {
    Quit,
}

#[derive(Debug)]
enum MainPlayerEvent {
    PlaybackEnded(Song),
    QueueChanged,
}

#[derive(Debug)]
enum MainPlayerMessage {
    Action(PlayerAction),
    Event(MainPlayerEvent),
    Command(MainPlayerCommand),
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum RepeatMode {
    Off,
    One,
    Queue,
}

pub struct MainPlayer {
    thread: JoinHandle<()>,
    sender: Sender<MainPlayerMessage>,
    player: Arc<SingleTrackPlayer>,
    queue: Arc<Queue>,
    on_queue_changed: Arc<Mutex<Option<Box<dyn Fn() + Send + 'static>>>>,
    on_error: Arc<Mutex<Option<Box<dyn Fn(String) + Send + 'static>>>>,
    repeat_mode: Arc<Mutex<RepeatMode>>,
}

impl MainPlayer {
    pub fn spawn(mpris: Option<Mpris>, queue_songs: Vec<Song>) -> Self {
        let (tx, rx) = channel::<MainPlayerMessage>();

        let mpris = mpris.map(Arc::new);
        let player = Arc::new(SingleTrackPlayer::spawn(mpris.clone()));
        let queue = Arc::new(Queue::new(queue_songs));
        let on_error = Arc::new(Mutex::new(None::<Box<dyn Fn(String) + Send + 'static>>));

        if let Some(mpris) = &mpris {
            mpris.on_play_pause({
                let player = player.clone();
                move || {
                    player.toggle_is_paused();
                }
            });
            mpris.on_stop({
                let player = player.clone();
                move || {
                    player.stop();
                }
            });
        }

        player.on_playback_end({
            let tx = tx.clone();
            move |song| {
                tx.send(MainPlayerMessage::Event(MainPlayerEvent::PlaybackEnded(song)))
                    .unwrap();
            }
        });

        player.on_error({
            let on_error = Arc::clone(&on_error);
            move |error| {
                log::warn!("Error reported by single_track_player: {error}");
                on_error.lock().unwrap().as_ref().inspect(|f| f(error));
            }
        });

        let on_queue_changed = Arc::new(Mutex::new(None));
        let repeat_mode = Arc::new(Mutex::new(RepeatMode::Off));

        let t = thread::Builder::new()
            .name("main_player".to_string())
            .spawn({
                let player = player.clone();
                let queue = queue.clone();
                let on_queue_changed = on_queue_changed.clone();
                let repeat_mode = Arc::clone(&repeat_mode);

                move || {
                    let mut song: Option<Song> = None;

                    loop {
                        let repeat_mode_lock = repeat_mode.lock().unwrap();

                        if *repeat_mode_lock == RepeatMode::One && song.is_some() {
                            player.play_song(song.clone().unwrap());
                        } else {
                            song = queue.pop();
                            if let Some(ref song) = song {
                                if *repeat_mode_lock == RepeatMode::Queue {
                                    queue.add_back(song.clone());
                                }
                                log::debug!("song_player grabbed song from queue {song:?}");
                                player.play_song(song.clone());
                                on_queue_changed.lock().unwrap().as_ref().inspect(|f| f());
                            } else {
                                log::debug!("song_player queue was empty. will wait for changes.");
                                // UX: Configurable Skip on Paused Behavior (TODO: configurable. See TODO.md)
                                player.set_is_paused(false);
                            }
                        }

                        drop(repeat_mode_lock);

                        loop {
                            match rx.recv().unwrap() {
                                MainPlayerMessage::Command(MainPlayerCommand::Quit) => {
                                    return;
                                }
                                MainPlayerMessage::Event(MainPlayerEvent::PlaybackEnded(song)) => {
                                    log::debug!("playback ended {song:?}");
                                    break;
                                }
                                MainPlayerMessage::Event(MainPlayerEvent::QueueChanged) => {
                                    if player.playing_song().lock().unwrap().is_none() {
                                        log::debug!("MainPlayerEvent::QueueChanged");
                                        break;
                                    }
                                }
                                MainPlayerMessage::Action(PlayerAction::RepeatOne) => {
                                    log::debug!("will repeat one song");
                                    *repeat_mode.lock().unwrap() = RepeatMode::One;
                                }
                                MainPlayerMessage::Action(PlayerAction::RepeatOff) => {
                                    log::debug!("will not repeat");
                                    *repeat_mode.lock().unwrap() = RepeatMode::Off;
                                }
                                MainPlayerMessage::Action(PlayerAction::RepeatQueue) => {
                                    log::debug!("will repeat entire queue");
                                    *repeat_mode.lock().unwrap() = RepeatMode::Queue;
                                }
                                m => {
                                    log::warn!("MainPlayer received unknown message {m:?}");
                                }
                            }
                        }

                        while let Ok(msg) = rx.try_recv() {
                            log::error!("had more messages {msg:?}");
                        }
                    }
                }
            })
            .unwrap();

        Self {
            thread: t,
            sender: tx,
            player,
            on_queue_changed,
            on_error,
            queue,
            repeat_mode,
        }
    }

    pub fn quit(self) {
        if let Some(player) = Arc::into_inner(self.player) {
            player.quit();
        } else {
            log::error!("Dangling references to player! Could not quit it gracefully.")
        }

        self.sender
            .send(MainPlayerMessage::Command(MainPlayerCommand::Quit))
            .unwrap();
        if let Err(err) = self.thread.join() {
            log::error!("error joining player thread {err:?}");
        };
    }

    pub fn single_track_player(&self) -> Arc<SingleTrackPlayer> {
        self.player.clone()
    }

    pub fn queue(&self) -> Arc<Queue> {
        self.queue.clone()
    }

    pub fn volume(&self) -> u32 {
        self.single_track_player().get_volume()
    }

    //// Playback Management

    pub fn playing_song(&self) -> Option<Song> {
        self.player.playing_song().lock().unwrap().clone()
    }

    pub fn playing_position(&self) -> Duration {
        self.player.playing_position()
    }

    pub fn is_paused(&self) -> bool {
        self.player.is_paused()
    }

    pub fn is_repeating(&self) -> bool {
        // self.is_repeating.load(Ordering::Acquire)
        *self.repeat_mode.lock().unwrap() == RepeatMode::One
    }

    pub fn repeat_mode(&self) -> RepeatMode {
        *self.repeat_mode.lock().unwrap()
    }

    pub fn play(&self, song: Song) {
        self.single_track_player().play_song(song);
    }

    pub fn stop(&self) {
        self.player.stop()
    }

    //// Queue Management

    pub fn on_queue_changed(&self, f: impl Fn() + Send + 'static) {
        *self.on_queue_changed.lock().unwrap() = Some(Box::new(f));
    }

    pub fn on_error(&self, f: impl Fn(String) + Send + 'static) {
        *self.on_error.lock().unwrap() = Some(Box::new(f));
    }

    fn notify_queue_changed(&self) {
        self.sender
            .send(MainPlayerMessage::Event(MainPlayerEvent::QueueChanged))
            .unwrap();
    }

    pub fn add_front(&self, song: Song) {
        self.queue.add_front(song);
        self.notify_queue_changed();
    }

    pub fn add_back(&self, song: Song) {
        self.queue.add_back(song);
        self.notify_queue_changed();
    }

    pub fn append(&self, songs: &mut VecDeque<Song>) {
        self.queue.append(songs);
        self.notify_queue_changed();
    }

    pub fn remove(&self, index: usize) {
        self.queue.remove(index);
        self.notify_queue_changed();
    }
}

impl OnAction<PlayerAction> for MainPlayer {
    fn on_action(&self, action: Vec<PlayerAction>) {
        match action[0] {
            PlayerAction::RepeatOff | PlayerAction::RepeatOne | PlayerAction::RepeatQueue => {
                self.sender.send(MainPlayerMessage::Action(action[0])).unwrap();
            }
            _ => {}
        }
    }
}
