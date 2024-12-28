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

enum MainPlayerCommand {
    Quit,
}

enum MainPlayerEvent {
    PlaybackEnded(Song),
    QueueChanged,
}

enum MainPlayerMessage {
    Action(MainPlayerAction),
    Event(MainPlayerEvent),
    Command(MainPlayerCommand),
}

pub struct MainPlayer {
    thread: JoinHandle<()>,
    sender: Sender<MainPlayerMessage>,
    player: Arc<SingleTrackPlayer>,
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

        let t = thread::Builder::new()
            .name("main_player".to_string())
            .spawn({
                let player = player.clone();
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
                    }
                }
            })
            .unwrap();

        Self {
            thread: t,
            sender: tx,
            player,
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
}

impl OnAction<MainPlayerAction> for MainPlayer {
    fn on_action(&self, action: MainPlayerAction) {
        self.sender.send(MainPlayerMessage::Action(action)).unwrap();
    }
}
