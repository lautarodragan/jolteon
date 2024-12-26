use std::sync::mpsc::{channel, Sender};
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;

use crate::{
    player::Player,
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
}

impl MainPlayer {
    pub fn spawn(player: Arc<Player>, queue: Arc<Queue>) -> Self {
        let (tx, rx) = channel::<MainPlayerMessage>();

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
            .name("song_player".to_string())
            .spawn(move || {
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
            })
            .unwrap();

        Self { thread: t, sender: tx }
    }

    pub fn quit(self) {
        self.sender
            .send(MainPlayerMessage::Command(MainPlayerCommand::Quit))
            .unwrap();
        if let Err(err) = self.thread.join() {
            log::error!("error joining player thread {err:?}");
        };
    }
}

impl OnAction<MainPlayerAction> for MainPlayer {
    fn on_action(&self, action: MainPlayerAction) {
        self.sender.send(MainPlayerMessage::Action(action)).unwrap();
    }
}
