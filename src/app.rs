use std::error::Error;
use std::sync::{mpsc::Receiver, Arc, Mutex, MutexGuard};
use std::{env, path::PathBuf, thread, time::Duration};
use std::io::BufRead;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread::JoinHandle;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    prelude::{Style, Widget},
    widgets::{Block, WidgetRef},
};
use rodio::OutputStream;

use crate::{
    config::Theme,
    structs::Song,
    player::Player,
    state::State,
    term::set_terminal,
    ui::{KeyboardHandler, KeyboardHandlerRef, KeyboardHandlerMut, TopBar},
    Command,
    components::{FileBrowser, FileBrowserSelection, Library, Playlists, Queue, Help},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppTab {
    Library,
    Playlists,
    Queue,
    FileBrowser,
    Help,
}

pub struct App<'a> {
    must_quit: bool,
    theme: Theme,
    frame: Arc<AtomicU64>,

    _music_output: OutputStream,
    player: Arc<Player>,
    queue: Arc<Queue>,
    player_command_receiver: Arc<Mutex<Receiver<Command>>>,
    media_rec_t: Option<JoinHandle<()>>,

    target: Option<KeyboardHandler<'a>>,
    active_tab: AppTab,
    focus_trap: Arc<AtomicBool>,

    library: Arc<Library<'a>>,
    playlist: Arc<Playlists<'a>>,
    browser: Arc<Mutex<FileBrowser<'a>>>,
    help: Arc<Mutex<Help<'a>>>,
}

impl<'a> App<'a> {
    pub fn new(player_command_receiver: Receiver<Command>) -> Self {
        let (output_stream, output_stream_handle) = OutputStream::try_default().unwrap(); // Indirectly this spawns the cpal_alsa_out thread, and creates the mixer tied to it

        let state = State::from_file();
        let theme = include_str!("../assets/theme.toml");
        let theme: Theme = toml::from_str(theme).unwrap();

        let queue = Arc::new(Queue::new(state.queue_items, theme));
        let player = Arc::new(Player::new(queue.clone(), output_stream_handle, theme));

        let current_directory = match &state.last_visited_path {
            Some(s) => PathBuf::from(s),
            None => env::current_dir().unwrap(),
        };

        let focus_trap = Arc::new(AtomicBool::new(false));

        let library = Arc::new(Library::new(theme));
        library.on_select({ // selected individual song
            let player = player.clone();
            let queue = queue.clone();
            let library = library.clone();

            move |song, key| {
                if key.code == KeyCode::Enter {
                    player.play_song(song);
                } else if key.code == KeyCode::Char('a') {
                    queue.add_back(song);
                    library.on_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)); // hackish way to "select_next()"
                }
            }
        });
        library.on_select_songs_fn({ // selected artist/album
            let queue = queue.clone();
            let library = library.clone();

            move |songs| {
                log::trace!(target: "::app.library", "on_select_songs_fn -> adding songs to queue");
                queue.append(&mut std::collections::VecDeque::from(songs));
                library.on_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)); // hackish way to "select_next()"
            }
        });

        let playlist = Arc::new(Playlists::new(theme));
        playlist.on_enter_song({
            let player = player.clone();
            let queue = queue.clone();
            move |song, key| {
                if key.code == KeyCode::Enter {
                    player.play_song(song);
                } else if key.code == KeyCode::Char('a') {
                    queue.add_back(song);
                }
            }
        });
        playlist.on_enter_playlist({
            let queue = queue.clone();
            move |playlist| {
                queue.append(&mut std::collections::VecDeque::from(playlist.songs));
            }
        });
        playlist.on_request_focus_trap_fn({
            let focus_trap = focus_trap.clone();
            move |v| {
                focus_trap.store(v, Ordering::Release);
            }
        });

        let mut browser = FileBrowser::new(theme, current_directory);
        browser.on_select({
            let player = player.clone();
            let queue = queue.clone();
            let playlists = playlist.clone();
            let media_library = Arc::clone(&library);

            move |(s, key_event)| {
                Self::on_file_browser_select(player.as_ref(), queue.as_ref(), playlists.as_ref(), media_library.as_ref(), s, key_event);
            }
        });

        Self {
            must_quit: false,
            theme,
            frame: Arc::new(AtomicU64::new(0)),

            _music_output: output_stream,
            player,
            queue,
            player_command_receiver: Arc::new(Mutex::new(player_command_receiver)),
            media_rec_t: None,

            target: Some(KeyboardHandler::Ref(library.clone())),
            active_tab: AppTab::Library,
            focus_trap,

            library,
            playlist,
            browser: Arc::new(Mutex::new(browser)),
            help: Arc::new(Mutex::new(Help::new(theme))),
        }
    }

    fn file_browser(&self) -> MutexGuard<FileBrowser<'a>>  {
        self.browser.lock().unwrap()
    }

    fn to_state(&self) -> State {
        let queue_items = self.queue.songs().clone();
        let playlists = self.playlist.playlists();

        State {
            last_visited_path: self.file_browser().current_directory().to_str().map(String::from),
            queue_items: Vec::from(queue_items),
            playlists,
        }
    }

    // Starts the player loop. Blocking.
    pub fn start(&mut self) -> Result<(), Box<dyn Error>> {
        let mut terminal = set_terminal()?;

        let tick_rate = Duration::from_millis(100);
        let mut last_tick = std::time::Instant::now();

        self.spawn_media_key_receiver_thread();
        self.player.spawn();

        while !self.must_quit {
            terminal.draw(|frame| {
                frame.render_widget_ref(&*self, frame.area());
            })?;

            let timeout = tick_rate.saturating_sub(last_tick.elapsed());

            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    self.on_key(key);
                }
            }

            if last_tick.elapsed() >= tick_rate {
                last_tick = std::time::Instant::now();
                self.frame.fetch_add(1, Ordering::Relaxed);
            }
        }

        log::trace!("App.start() -> exiting");

        self.to_state().to_file()?;

        self.playlist.save(); // TODO: save on each change, like the library

        Ok(())
    }

    fn spawn_media_key_receiver_thread(&mut self) {
        let player_command_receiver = self.player_command_receiver.clone();
        let player = self.player.clone();

        let t = thread::Builder::new().name("media_key_rx".to_string()).spawn(move || {
            loop {
                match player_command_receiver.lock().unwrap().recv() {
                    Ok(Command::PlayPause) => {
                        player.toggle();
                    }
                    Ok(Command::Next) => {
                        player.stop();
                    }
                    Ok(Command::Quit) => {
                        log::debug!("Received Command::Quit");
                        break;
                    }
                    Err(err) => {
                        log::error!("Channel error: {}", err);
                        break;
                    }
                }
            }
            log::trace!("spawn_media_key_receiver_thread loop exit");
        }).unwrap();

        self.media_rec_t = Some(t);
    }

    fn on_file_browser_select(
        player: &Player,
        queue: &Queue,
        playlists: &Playlists,
        media_library: &Library,
        file_browser_selection: FileBrowserSelection,
        key_event: KeyEvent,
    ) {
        match (file_browser_selection, key_event.code) {
            (FileBrowserSelection::Song(song), KeyCode::Enter) => {
                player.play_song(song);
            }
            (FileBrowserSelection::CueSheet(cue_sheet), KeyCode::Enter) => {
                let songs = Song::from_cue_sheet(cue_sheet);
                queue.append(&mut std::collections::VecDeque::from(songs));
            }

            (FileBrowserSelection::Song(song), KeyCode::Char('j')) => {
                media_library.add_song(song.clone());
            }
            (FileBrowserSelection::CueSheet(cue_sheet), KeyCode::Char('j')) => {
                media_library.add_cue(cue_sheet);
            }
            (FileBrowserSelection::Directory(path), KeyCode::Char('j')) => {
                media_library.add_directory(&path);
            }

            (FileBrowserSelection::Song(song), KeyCode::Char('a')) => {
                queue.add_back(song);
            }
            (FileBrowserSelection::CueSheet(cue_sheet), KeyCode::Char('a')) => {
                let songs = Song::from_cue_sheet(cue_sheet);
                queue.append(&mut std::collections::VecDeque::from(songs));
            }
            (FileBrowserSelection::Directory(path), KeyCode::Char('a')) => {
                log::debug!("TODO: file_browser().on_select(Directory({}), a)", path.display());
                // directory_to_songs_and_folders
            }

            (FileBrowserSelection::Song(song), KeyCode::Char('y')) => {
                playlists.add_song(song);
            }
            (FileBrowserSelection::CueSheet(cue_sheet), KeyCode::Char('y')) => {
                playlists.add_cue(cue_sheet);
            }
            (FileBrowserSelection::Directory(path), KeyCode::Char('y')) => {
                let mut songs = Song::from_dir(&path);
                playlists.add_songs(&mut songs);
            }
            _ => {}
        }
    }

    fn spawn_terminal(&self) {
        let cwd = self.file_browser().current_directory().clone();

        if let Err(err) = thread::Builder::new().name("term".to_string()).spawn(move || {
            log::debug!("spawning child process");

            let proc = std::process::Command::new("kitty")
                .current_dir(cwd)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn();

            if let Ok(mut proc) = proc {
                log::debug!("spawned child process");

                let stdout = proc.stdout.as_mut().unwrap();
                let stdout_reader = std::io::BufReader::new(stdout);

                for line in stdout_reader.lines() {
                    log::debug!("stdout: {:?}", line);
                }

                log::debug!("child process exited");
            } else if let Err(err) = proc {
                log::error!("error spawning thread {:?}", err);
            }
        }) {
            log::error!("Error spawning thread! {:?}", err);
        }
    }

}

impl<'a> KeyboardHandlerMut<'a> for App<'a> {
    fn on_key(&mut self, key: KeyEvent) {
        if key.code == KeyCode::Char('q') && key.modifiers == KeyModifiers::CONTROL  {
            self.must_quit = true;
            return;
        }

        let mut handled = true;

        let focus_trapped = self.focus_trap.load(Ordering::Acquire);
        if !focus_trapped {
            match key.code {
                KeyCode::Right => self.player.seek_forward(),
                KeyCode::Left => self.player.seek_backward(),
                KeyCode::Char('-') => self.player.change_volume(-0.05),
                KeyCode::Char('+') => self.player.change_volume(0.05),
                KeyCode::Char(' ') => self.player.toggle(),
                KeyCode::Char('g') if key.modifiers == KeyModifiers::CONTROL => self.player.stop(),
                KeyCode::Char('c') if key.modifiers == KeyModifiers::ALT => self.spawn_terminal(),
                KeyCode::Char('1') => {
                    self.active_tab = AppTab::Library;
                    self.target = Some(KeyboardHandler::Ref(self.library.clone()));
                }
                KeyCode::Char('2') => {
                    self.active_tab = AppTab::Playlists;
                    self.target = Some(KeyboardHandler::Ref(self.playlist.clone()));
                }
                KeyCode::Char('3') => {
                    self.active_tab = AppTab::Queue;
                    self.target = Some(KeyboardHandler::Ref(self.queue.clone()));
                }
                KeyCode::Char('4') => {
                    self.active_tab = AppTab::FileBrowser;
                    self.target = Some(KeyboardHandler::Mut(self.browser.clone()));
                }
                KeyCode::Char('5') => {
                    self.active_tab = AppTab::Help;
                    self.target = Some(KeyboardHandler::Mut(self.help.clone()));
                }
                _ => {
                    handled = false;
                }
            };
        };

        if focus_trapped || !handled {
            if let Some(target) = &self.target {
                match target {
                    KeyboardHandler::Ref(target) => {
                        target.on_key(key);
                    }
                    KeyboardHandler::Mut(target) => {
                        target.lock().unwrap().on_key(key);
                    }
                }
            }
        }
    }
}

impl<'a> WidgetRef for &App<'a> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        Block::default()
            .style(Style::default().bg(self.theme.background))
            .render(area, buf);

        let [area_top, _, area_center, area_bottom] =
            Layout::vertical([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(3),
            ]).areas(area);

        let top_bar = TopBar::new(self.theme, self.active_tab, self.frame.load(Ordering::Relaxed));
        top_bar.render(area_top, buf);

        match self.active_tab {
            AppTab::Library => {
                self.library.render_ref(area_center, buf);
            },
            AppTab::Playlists => {
                self.playlist.render_ref(area_center, buf);
            },
            AppTab::Queue => {
                self.queue.render_ref(area_center, buf);
            },
            AppTab::FileBrowser => {
                self.browser.lock().unwrap().render_ref(area_center, buf);
            },
            AppTab::Help => {
                self.help.lock().unwrap().render_ref(area_center, buf);
            },
        };

        self.player.render(area_bottom, buf);
    }
}

impl Drop for App<'_> {
    fn drop(&mut self) {
        log::trace!("App.drop");

        self.queue.quit();

        if let Some(a) = self.media_rec_t.take() {
            log::trace!("App.drop: joining media_key_rx thread");
            match a.join() {
                Ok(_) => {
                    // log::trace!("ok");
                }
                Err(err) => {
                    log::error!("{:?}", err);
                }
            }
        } else {
            log::warn!("No media_key_rx thread!?");
        }

        log::trace!("media_key_rx thread joined successfully");
    }
}
