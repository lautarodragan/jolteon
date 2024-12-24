use std::sync::Arc;
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::thread::JoinHandle;

use crate::player::Player;
use crate::structs::{Queue, Song};

enum Action {
    Quit,
    PlaybackEnded(Song),
    QueueChanged,
    RepeatOff,
    RepeatOne,
}

pub struct MainPlayer {
    thread: JoinHandle<()>,
    sender: Sender<Action>,
}

impl MainPlayer {
    pub fn spawn(player: Arc<Player>, queue: Arc<Queue>) -> Self {
        let (tx, rx) = channel::<Action>();

        player.on_playback_end({
            let tx = tx.clone();
            move |song| {
                tx.send(Action::PlaybackEnded(song)).unwrap();
            }
        });

        queue.on_queue_changed({
            let tx = tx.clone();
            move || {
                tx.send(Action::QueueChanged).unwrap();
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
                            Action::Quit => {
                                return;
                            }
                            Action::PlaybackEnded(song) => {
                                log::debug!("playback ended {song:?}");
                                break;
                            }
                            Action::QueueChanged => {
                                if player.currently_playing().lock().unwrap().is_none() {
                                    break;
                                }
                            }
                            Action::RepeatOne => {
                                log::debug!("will repeat one song");
                                repeat = true;
                            }
                            Action::RepeatOff => {
                                log::debug!("will not repeat");
                                repeat = false;
                            }
                        }
                    }
                }
            })
            .unwrap();

        Self {
            thread: t,
            sender: tx,
        }
    }

    pub fn quit(self) {
        self.sender.send(Action::Quit).unwrap();
        if let Err(err) = self.thread.join() {
            log::error!("error joining player thread {err:?}");
        };
    }

    pub fn repeat_one(&self) {
        self.sender.send(Action::RepeatOne).unwrap();
    }

    pub fn repeat_off(&self) {
        self.sender.send(Action::RepeatOff).unwrap();
    }
}
