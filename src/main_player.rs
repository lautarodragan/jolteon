use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{channel, Sender},
        Arc,
        Mutex,
    },
    thread,
    thread::JoinHandle,
    time::Duration,
};

use rodio::OutputStreamHandle;

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

pub struct MainPlayer {
    thread: JoinHandle<()>,
    sender: Sender<MainPlayerMessage>,
    player: Arc<SingleTrackPlayer>,
    queue: Arc<Queue>,
    on_queue_changed: Arc<Mutex<Option<Box<dyn Fn() + Send + 'static>>>>,
    is_repeating: Arc<AtomicBool>,
}

impl MainPlayer {
    pub fn spawn(output_stream_handle: OutputStreamHandle, mpris: Option<Mpris>, queue_songs: Vec<Song>) -> Self {
        let (tx, rx) = channel::<MainPlayerMessage>();

        let mpris = mpris.map(Arc::new);
        let player = Arc::new(SingleTrackPlayer::spawn(output_stream_handle, mpris.clone()));
        let queue = Arc::new(Queue::new(queue_songs));

        if let Some(mpris) = &mpris {
            mpris.on_play_pause({
                let player = player.clone();
                move || {
                    player.toggle();
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

        let on_queue_changed = Arc::new(Mutex::new(None));
        let is_repeating = Arc::new(AtomicBool::new(false));

        let t = thread::Builder::new()
            .name("main_player".to_string())
            .spawn({
                let player = player.clone();
                let queue = queue.clone();
                let on_queue_changed = on_queue_changed.clone();
                let is_repeating = Arc::clone(&is_repeating);

                move || {
                    let mut repeat = false;
                    let mut song: Option<Song> = None;

                    loop {
                        if repeat && song.is_some() {
                            player.play_song(song.clone().unwrap());
                        } else {
                            song = queue.pop();
                            if let Some(ref song) = song {
                                log::debug!("song_player grabbed song from queue {song:?}");
                                player.play_song(song.clone());
                                on_queue_changed.lock().unwrap().as_ref().inspect(|f| f());
                            } else {
                                log::debug!("song_player queue was empty. will wait for changes.");
                            }
                        }

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
                                    repeat = true;
                                    is_repeating.store(true, Ordering::Release);
                                }
                                MainPlayerMessage::Action(PlayerAction::RepeatOff) => {
                                    log::debug!("will not repeat");
                                    repeat = false;
                                    is_repeating.store(false, Ordering::Release);
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
            queue,
            is_repeating,
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
        self.is_repeating.load(Ordering::Acquire)
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
            PlayerAction::RepeatOne | PlayerAction::RepeatOff => {
                self.sender.send(MainPlayerMessage::Action(action[0])).unwrap();
            }
            _ => {}
        }
    }
}
