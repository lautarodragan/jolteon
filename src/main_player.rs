use std::collections::VecDeque;
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

use rodio::OutputStreamHandle;

use crate::{
    mpris::Mpris,
    player::SingleTrackPlayer,
    structs::{MainPlayerAction, OnAction, Queue, Song},
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
    Action(MainPlayerAction),
    Event(MainPlayerEvent),
    Command(MainPlayerCommand),
}

pub struct MainPlayer {
    thread: JoinHandle<()>,
    sender: Sender<MainPlayerMessage>,
    player: Arc<SingleTrackPlayer>,
    queue: Arc<Queue>,
    on_queue_changed: Arc<Mutex<Option<Box<dyn Fn() + Send + 'static>>>>,
}

impl MainPlayer {
    pub fn spawn(output_stream_handle: OutputStreamHandle, queue: Arc<Queue>, mpris: Mpris) -> Self {
        let (tx, rx) = channel::<MainPlayerMessage>();

        let mpris = Arc::new(mpris);
        let player = Arc::new(SingleTrackPlayer::new(output_stream_handle, mpris.clone()));

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

        player.spawn();

        player.on_playback_end({
            let tx = tx.clone();
            move |song| {
                tx.send(MainPlayerMessage::Event(MainPlayerEvent::PlaybackEnded(song)))
                    .unwrap();
            }
        });

        queue.on_queue_changed({
            let tx = tx.clone();
            move || {
                tx.send(MainPlayerMessage::Event(MainPlayerEvent::QueueChanged))
                    .unwrap();
            }
        });

        let on_queue_changed = Arc::new(Mutex::new(None));

        let t = thread::Builder::new()
            .name("main_player".to_string())
            .spawn({
                let player = player.clone();
                let queue = queue.clone();
                let on_queue_changed = on_queue_changed.clone();

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
                                    if player.currently_playing().lock().unwrap().is_none() {
                                        log::debug!("MainPlayerEvent::QueueChanged");
                                        break;
                                    }
                                }
                                MainPlayerMessage::Action(MainPlayerAction::RepeatOne) => {
                                    log::debug!("will repeat one song");
                                    repeat = true;
                                }
                                MainPlayerMessage::Action(MainPlayerAction::RepeatOff) => {
                                    log::debug!("will not repeat");
                                    repeat = false;
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
        }
    }

    pub fn quit(self) {
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

    pub fn is_paused(&self) -> bool {
        self.player.is_paused()
    }

    pub fn currently_playing(&self) -> Arc<Mutex<Option<Song>>> {
        self.player.currently_playing()
    }

    pub fn get_pos(&self) -> Duration {
        self.player.get_pos()
    }

    pub fn stop(&self) {
        self.player.stop()
    }

    pub fn play(&self, song: Song) {
        self.single_track_player().play_song(song);
    }

    pub fn queue_changed(&self) {
        self.sender
            .send(MainPlayerMessage::Event(MainPlayerEvent::QueueChanged))
            .unwrap();
    }

    pub fn on_queue_changed(&self, f: impl Fn() + Send + 'static) {
        *self.on_queue_changed.lock().unwrap() = Some(Box::new(f));
    }

    pub fn add_front(&self, song: Song) {
            self.queue.add_front(song);
    }

    pub fn add_back(&self, song: Song) {
            self.queue.add_back(song);
    }

    pub fn append(&self, songs: &mut VecDeque<Song>) {
            self.queue.append(songs);
    }

    pub fn remove(&self, index: usize) {
            self.queue.remove(index);
    }
}

impl OnAction<MainPlayerAction> for MainPlayer {
    fn on_action(&self, action: MainPlayerAction) {
        self.sender.send(MainPlayerMessage::Action(action)).unwrap();
    }
}
