use std::{
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc::{channel, Receiver, RecvTimeoutError, Sender},
        Arc, Mutex,
    },
    thread,
    thread::JoinHandle,
    time::Duration,
};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use rodio::OutputStreamHandle;

use crate::{
    mpris::Mpris,
    source::{Controls, Source},
    structs::{OnAction, PlayerAction, Song},
    ui::KeyboardHandlerRef,
};

pub struct Player {
    output_stream: OutputStreamHandle,
    main_thread: Mutex<Option<JoinHandle<()>>>,

    currently_playing: Arc<Mutex<Option<Song>>>,
    currently_playing_start_time: Arc<AtomicU64>,
    command_sender: Sender<Command>,
    command_receiver: Arc<Mutex<Option<Receiver<Command>>>>,
    is_stopped: Arc<AtomicBool>,
    volume: Arc<Mutex<f32>>,
    pause: Arc<AtomicBool>,
    position: Arc<Mutex<Duration>>,

    frame: AtomicU64,
    paused_animation_start_frame: AtomicU64,

    mpris: Arc<Mpris>,

    on_playback_end: Arc<Mutex<Option<Box<dyn Fn(Song) + Send + 'static>>>>,
}

#[derive(Debug)]
#[allow(dead_code)]
enum Command {
    SetSong(Song),
    Play,
    Pause,
    Stop,
    Seek(i32),
    Quit,
}

impl Player {
    pub fn new(output_stream: OutputStreamHandle, mpris: Arc<Mpris>) -> Self {
        let (command_sender, command_receiver) = channel();

        Self {
            output_stream,
            main_thread: Mutex::new(None),

            currently_playing: Arc::new(Mutex::new(None)),
            currently_playing_start_time: Arc::new(AtomicU64::new(0)),
            command_sender,
            command_receiver: Arc::new(Mutex::new(Some(command_receiver))),
            on_playback_end: Arc::new(Mutex::new(None)),

            is_stopped: Arc::new(AtomicBool::new(true)),
            volume: Arc::new(Mutex::new(1.0)),
            pause: Arc::new(AtomicBool::new(false)),
            position: Arc::new(Mutex::new(Duration::ZERO)),

            frame: AtomicU64::new(0),
            paused_animation_start_frame: AtomicU64::new(0),

            mpris,
        }
    }

    fn send_command(&self, command: Command) {
        if let Err(err) = self.command_sender.send(command) {
            log::warn!("Player.send_command() failure: {:?}", err);
        }
    }

    pub fn get_pos(&self) -> Duration {
        let start_time = self.currently_playing_start_time.load(Ordering::Relaxed);
        let pos = self.position.lock().unwrap();
        pos.saturating_sub(Duration::from_secs(start_time))
    }

    pub fn currently_playing(&self) -> Arc<Mutex<Option<Song>>> {
        self.currently_playing.clone()
    }

    pub fn spawn(&self) {
        let output_stream = self.output_stream.clone();
        let command_receiver = self.command_receiver.lock().unwrap().take().unwrap();
        let currently_playing = self.currently_playing.clone();
        let song_start_time = self.currently_playing_start_time.clone();
        let mpris = self.mpris.clone();

        let position = self.position.clone();
        let volume = self.volume.clone();
        let pause = self.pause.clone();

        let (song_ended_tx, song_ended_rx) = channel::<()>();
        let is_stopped = self.is_stopped.clone();
        let must_stop = Arc::new(AtomicBool::new(false));
        let must_seek = Arc::new(Mutex::new(None));

        let set_currently_playing = {
            let mpris = mpris.clone();
            move |song: Option<Song>| {
                let start_time = song
                    .as_ref()
                    .map(|song| song.start_time)
                    .unwrap_or(Duration::ZERO)
                    .as_secs();
                song_start_time.store(start_time, Ordering::Relaxed);

                match song {
                    Some(ref song) => {
                        mpris.set_song(song.clone());
                        mpris.play();
                    }
                    None => mpris.clear_song(),
                }

                match currently_playing.lock() {
                    Ok(mut s) => {
                        *s = song;
                    }
                    Err(err) => {
                        log::error!("currently_playing.lock() returned an error! {:?}", err);
                    }
                };
            }
        };

        let on_playback_end = self.on_playback_end.clone();

        let thread = thread::Builder::new()
            .name("player".to_string())
            .spawn(move || loop {
                let song = loop {
                    match command_receiver.recv() {
                        Ok(Command::SetSong(song)) => break song,
                        Ok(Command::Quit) => return,
                        Err(_) => return,
                        _ => continue,
                    }
                };

                let path = song.path.clone();
                let start_time = song.start_time;
                let length = song.length;

                is_stopped.store(false, Ordering::SeqCst);

                set_currently_playing(Some(song.clone()));

                let periodic_access = {
                    let is_stopped = is_stopped.clone();
                    let must_stop = must_stop.clone();
                    let volume = volume.clone();
                    let pause = pause.clone();
                    let must_seek = must_seek.clone();

                    move |controls: &mut Controls| {
                        if must_stop.swap(false, Ordering::SeqCst) {
                            controls.stop();
                            controls.skip();
                            is_stopped.store(true, Ordering::SeqCst);
                            log::debug!("periodic access stop");
                            return;
                        }

                        controls.set_volume(*volume.lock().unwrap());
                        controls.set_paused(pause.load(Ordering::SeqCst));

                        if let Some(seek) = must_seek.lock().unwrap().take() {
                            if let Err(err) = controls.seek(seek) {
                                log::error!("periodic_access.try_seek() error. {:?}", err)
                            }
                        }
                    }
                };

                let wait_until_song_ends = || {
                    let target = "::wait_until_song_ends";
                    log::debug!(target: target, "start");
                    must_stop.store(true, Ordering::SeqCst);

                    if let Err(err) = song_ended_rx.recv() {
                        log::error!("ender_recv.recv {:?}", err);
                        return;
                    }

                    log::debug!(target: target, "ender signal received");

                    while song_ended_rx.try_recv().is_ok() {}

                    must_stop.store(false, Ordering::SeqCst);
                    must_seek.lock().unwrap().take();

                    set_currently_playing(None);

                    log::debug!(target: target, "done");
                };

                let mut source = Source::from_file(path, periodic_access, position.clone(), {
                    let song_ended_tx = song_ended_tx.clone();
                    move || {
                        log::trace!("source.on_playback_ended");
                        let _ = song_ended_tx.send(());
                    }
                });

                if start_time > Duration::ZERO {
                    log::debug!("start_time > Duration::ZERO, {:?}", start_time);
                    if let Err(err) = source.seek(start_time) {
                        log::error!("start_time > 0 try_seek() error. {:?}", err)
                    }
                }

                *position.lock().unwrap() = start_time;

                log::debug!("output_stream.play_raw()");
                if let Err(err) = output_stream.play_raw(source) {
                    // play_raw does `mixer.add(source)`. Mixer is tied to the CPAL thread, which starts consuming the source automatically.
                    log::error!("os.play_raw error! {:?}", err);
                    continue;
                }

                // Start looping until the current song ends OR something wakes us up.
                // When woken up, we check whether we need to immediately exit.
                // If we don't, we recalculate the remaining time until the song ends,
                // and then go back to bed.
                loop {
                    let sleepy_time = if pause.load(Ordering::SeqCst) {
                        Duration::MAX
                    } else {
                        let abs_pos = position.lock().unwrap().saturating_sub(start_time);
                        if abs_pos >= length {
                            log::debug!("inner loop: pos >= length, {:?} > {:?}", abs_pos, length);
                            break;
                        }
                        length - abs_pos
                    };

                    // log::debug!("inner loop: sleepy_time! {:?}", sleepy_time);

                    match command_receiver.recv_timeout(sleepy_time) {
                        Ok(command) => {
                            log::debug!("Player.Command({:?})", command);
                            match command {
                                Command::SetSong(song) => {
                                    log::error!("oops! received SetSong while playing! {song:?}");
                                }
                                Command::Quit => {
                                    log::trace!("Player: quitting main loop");
                                    return;
                                }
                                Command::Play => {
                                    pause.store(false, Ordering::SeqCst);
                                    mpris.play();
                                }
                                Command::Pause => {
                                    pause.store(true, Ordering::SeqCst);
                                    mpris.pause();
                                }
                                Command::Stop => {
                                    break;
                                }
                                Command::Seek(seek) => {
                                    // NOTE: "intense" seek causes `ALSA lib pcm.c:8740:(snd_pcm_recover) underrun occurred`.
                                    // See https://github.com/RustAudio/cpal/pull/909

                                    if seek == 0 {
                                        log::error!("Command::Seek(0)");
                                        continue;
                                    }

                                    if is_stopped.load(Ordering::SeqCst) || must_stop.load(Ordering::SeqCst) {
                                        continue;
                                    }

                                    let seek_abs = Duration::from_secs(seek.unsigned_abs() as u64);
                                    let mut pos = position.lock().unwrap();

                                    let target = if seek > 0 {
                                        pos.saturating_add(seek_abs)
                                    } else {
                                        pos.saturating_sub(seek_abs).max(start_time)
                                    };

                                    // If we'd seek past song end, skip seeking and just move to next song instead.
                                    if target > length + start_time {
                                        log::debug!("Seeking past end");
                                        break;
                                    }

                                    log::debug!("Seek({:?})", target);
                                    *must_seek.lock().unwrap() = Some(target);
                                    *pos = target; // optimistic update, otherwise sleepy_time will be off
                                }
                            }
                        }
                        Err(RecvTimeoutError::Timeout) => {
                            // Playing song reached its end. We want to move on to the next song.
                            log::trace!("Player Command Timeout");
                            break;
                        }
                        Err(RecvTimeoutError::Disconnected) => {
                            // Most of the time, not a real error. This can happen because the command_sender was dropped,
                            // which happens when the player itself was dropped, so we just want to exit.
                            log::warn!("RecvTimeoutError::Disconnected");
                            return;
                        }
                    }
                }

                // while command_receiver.try_recv().is_ok() {} // "drain" the command queue - dropping everything that might have accumulated.

                wait_until_song_ends();

                let on_playback_end = on_playback_end.lock().unwrap();
                if let Some(on_playback_end) = &*on_playback_end {
                    on_playback_end(song);
                }
            })
            .unwrap();

        *self.main_thread.lock().unwrap() = Some(thread);
    }

    pub fn on_playback_end(&self, f: impl Fn(Song) + Send + 'static) {
        *self.on_playback_end.lock().unwrap() = Some(Box::new(f));
    }

    pub fn play_song(&self, song: Song) {
        let mpris = self.mpris.clone();
        mpris.set_song(song.clone());
        mpris.play();
        self.send_command(Command::Stop);
        self.send_command(Command::SetSong(song));
    }

    pub fn toggle(&self) {
        if self.pause.load(Ordering::SeqCst) {
            self.send_command(Command::Play);
        } else {
            self.send_command(Command::Pause);
            self.paused_animation_start_frame
                .store(self.frame.load(Ordering::Relaxed), Ordering::Relaxed);
        }
    }

    pub fn is_paused(&self) -> bool {
        self.pause.load(Ordering::SeqCst)
    }

    pub fn stop(&self) {
        self.send_command(Command::Stop);
    }

    pub fn seek(&self, seek: i32) {
        // Avoid queueing seek commands if nothing is playing
        if self.is_stopped.load(Ordering::SeqCst) {
            return;
        }
        // Note: Symphonia seems to be the only decoder that supports seeking in Rodio (that we really care about), but it can fail.
        // Rodio's `Source for TrackPosition` does have its own `try_seek`, though, as well as `Source for SamplesBuffer`.
        // Are we using those (indirectly), or just Symphonia?
        self.send_command(Command::Seek(seek));
    }

    pub fn seek_forward(&self) {
        self.seek(5);
    }

    pub fn seek_backward(&self) {
        self.seek(-5);
    }

    pub fn change_volume(&self, amount: f32) {
        let mut volume = self.volume.lock().unwrap();
        *volume = (*volume + amount).clamp(0., 1.);
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        log::trace!("Player.drop()");

        self.send_command(Command::Quit);
        // TODO: break out of wait_until_song_ends loop?

        if let Some(thread) = self.main_thread.lock().unwrap().take() {
            log::trace!("Player.drop: joining main_thread thread");
            match thread.join() {
                Ok(_) => {
                    log::trace!("Player.drop: main_thread joined successfully");
                }
                Err(err) => {
                    log::error!("Player.drop: {:?}", err);
                }
            }
        } else {
            log::error!("No main_thread thread!?");
        }

        log::trace!("Player.drop()'ed");
    }
}

impl KeyboardHandlerRef<'_> for Player {
    fn on_key(&self, key: KeyEvent) {
        match key.code {
            KeyCode::Right => self.seek_forward(),
            KeyCode::Left => self.seek_backward(),
            KeyCode::Char('-') => self.change_volume(-0.05),
            KeyCode::Char('+') => self.change_volume(0.05),
            KeyCode::Char(' ') if key.modifiers == KeyModifiers::CONTROL => self.toggle(),
            KeyCode::Char('g') if key.modifiers == KeyModifiers::CONTROL => self.stop(),
            _ => {}
        };
    }
}

impl OnAction<PlayerAction> for Player {
    fn on_action(&self, action: PlayerAction) {
        match action {
            PlayerAction::PlayPause => {
                self.toggle();
            }
            PlayerAction::Stop => {
                self.stop();
            }
            PlayerAction::VolumeUp => {
                self.change_volume(0.05);
            }
            PlayerAction::VolumeDown => {
                self.change_volume(-0.05);
            }
            PlayerAction::SeekForwards => {
                self.seek_forward();
            }
            PlayerAction::SeekBackwards => {
                self.seek_backward();
            }
        }
    }
}
