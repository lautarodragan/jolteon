use std::{
    env,
    error::Error,
    path::PathBuf,
    rc::Rc,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc::Receiver,
        Arc, Mutex,
    },
    thread,
    thread::JoinHandle,
    time::Duration,
};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    prelude::{Style, Widget},
    widgets::{Block, WidgetRef},
};
use rodio::OutputStream;

use crate::{
    components::{FileBrowser, Help, Library, Playlists, Queue},
    config::Theme,
    player::Player,
    state::State,
    structs::{Action, Actions, OnAction, OnActionMut, ScreenAction},
    term::set_terminal,
    ui::{Component, KeyboardHandlerMut, KeyboardHandlerRef, TopBar},
    Command,
};

pub struct App<'a> {
    must_quit: bool,

    theme: Theme,
    actions: Actions,
    frame: Arc<AtomicU64>,

    _music_output: OutputStream,

    screens: Vec<(String, Component<'a>)>,
    focused_screen: usize,
    focus_trap: Arc<AtomicBool>,

    player: Arc<Player>,
    queue: Arc<Queue>,
    browser: Rc<FileBrowser<'a>>,

    player_command_receiver: Arc<Mutex<Receiver<Command>>>,
    player_command_receiver_thread: Option<JoinHandle<()>>,
}

impl<'a> App<'a> {
    pub fn new(player_command_receiver: Receiver<Command>) -> Self {
        let state = State::from_file();
        let actions = Actions::from_file_or_default();
        assert!(actions.contains(Action::Quit), "No key binding for Action::Quit! Would not be able to close gracefully. This is 100% a bug.");

        let (output_stream, output_stream_handle) = OutputStream::try_default().unwrap(); // Indirectly this spawns the cpal_alsa_out thread, and creates the mixer tied to it

        let theme = include_str!("../assets/theme.toml");
        let theme: Theme = toml::from_str(theme).unwrap();

        let queue = Arc::new(Queue::new(state.queue_items, theme));
        let player = Arc::new(Player::new(queue.clone(), output_stream_handle, theme));

        let current_directory = match &state.last_visited_path {
            Some(s) => PathBuf::from(s),
            None => env::current_dir().unwrap(),
        };

        let focus_trap = Arc::new(AtomicBool::new(false));

        let library = Rc::new(Library::new(theme));
        library.on_enter({
            let queue = queue.clone();

            move |song| {
                queue.add_back(song);
            }
        });
        library.on_enter_alt({
            let player = player.clone();

            move |song| {
                player.play_song(song);
            }
        });
        library.on_select_songs_fn({
            // selected artist/album
            let queue = queue.clone();
            let library = library.clone();

            move |songs| {
                log::trace!(target: "::app.library", "on_select_songs_fn -> adding songs to queue");
                queue.append(&mut std::collections::VecDeque::from(songs));
                // hackish way to "select_next()":
                library.on_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
            }
        });

        let playlist = Rc::new(Playlists::new(theme));
        playlist.on_enter_song({
            let queue = queue.clone();
            move |song| {
                queue.add_back(song);
            }
        });
        playlist.on_enter_song_alt({
            let player = player.clone();
            move |song| {
                player.play_song(song);
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

        let browser = Rc::new(FileBrowser::new(theme, current_directory));
        browser.on_enqueue({
            let queue = queue.clone();

            move |songs| {
                queue.append(&mut std::collections::VecDeque::from(songs));
            }
        });
        browser.on_add_to_lib({
            let library = library.clone();

            move |songs| {
                library.add_songs(songs);
            }
        });
        browser.on_add_to_playlist({
            let playlist = playlist.clone();

            move |mut songs| {
                playlist.add_songs(&mut songs);
            }
        });

        let help = Arc::new(Mutex::new(Help::new(theme)));

        Self {
            must_quit: false,

            theme,
            actions,
            frame: Arc::new(AtomicU64::new(0)),

            _music_output: output_stream,

            screens: vec![
                ("Library".to_string(), Component::RefRc(library.clone())),
                ("Playlists".to_string(), Component::RefRc(playlist.clone())),
                ("Queue".to_string(), Component::RefArc(queue.clone())),
                ("File Browser".to_string(), Component::RefRc(browser.clone())),
                ("Help".to_string(), Component::Mut(help.clone())),
            ],
            focused_screen: 0,
            focus_trap,

            player,
            queue,
            browser,

            player_command_receiver: Arc::new(Mutex::new(player_command_receiver)),
            player_command_receiver_thread: None,
        }
    }

    fn to_state(&self) -> State {
        let queue_items = self.queue.songs().clone();

        State {
            last_visited_path: self.browser.current_directory().to_str().map(String::from),
            queue_items: Vec::from(queue_items),
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

        // self.actions.to_file();

        Ok(())
    }

    fn spawn_media_key_receiver_thread(&mut self) {
        let player_command_receiver = self.player_command_receiver.clone();
        let player = self.player.clone();

        let t = thread::Builder::new()
            .name("media_key_rx".to_string())
            .spawn(move || {
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
            })
            .unwrap();

        self.player_command_receiver_thread = Some(t);
    }
}

impl<'a> KeyboardHandlerMut<'a> for App<'a> {
    fn on_key(&mut self, key: KeyEvent) {
        // let Some(action) = self.actions.by_key(key) else {
        //     return;
        // };
        let action = self.actions.action_by_key(key);

        log::debug!("app.on_key action=('{:?}')", action);

        if let Some(action) = action {
            if action == Action::Quit {
                self.must_quit = true;
                return;
            }
            if !self.focus_trap.load(Ordering::Acquire) {
                match action {
                    Action::Screen(_) => {
                        self.on_action(action);
                        return;
                    }
                    Action::Player(_) => {
                        self.player.on_action(action);
                        return;
                    }
                    _ => {}
                }
            }
        }

        let Some((_, component)) = self.screens.get(self.focused_screen) else {
            log::error!("focused_screen is {}, which is out of bounds.", self.focused_screen);
            return;
        };

        component.on_key(key);
    }
}

impl<'a> WidgetRef for &App<'a> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        Block::default()
            .style(Style::default().bg(self.theme.background))
            .render(area, buf);

        let [area_top, _, area_center, area_bottom] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .areas(area);

        let screen_titles: Vec<&str> = self.screens.iter().map(|screen| screen.0.as_str()).collect();

        let top_bar = TopBar::new(
            self.theme,
            &screen_titles,
            self.focused_screen,
            self.frame.load(Ordering::Relaxed),
        );
        top_bar.render(area_top, buf);

        let Some((_, component)) = self.screens.get(self.focused_screen) else {
            log::error!("focused_screen is {}, which is out of bounds.", self.focused_screen);
            return;
        };

        match component {
            Component::RefArc(ref s) => {
                s.render_ref(area_center, buf);
            }
            Component::RefRc(ref s) => {
                s.render_ref(area_center, buf);
            }
            Component::Mut(ref s) => {
                s.lock().unwrap().render_ref(area_center, buf);
            }
        }

        self.player.render(area_bottom, buf);
    }
}

impl<'a> OnActionMut for App<'a> {
    fn on_action(&mut self, action: Action) {
        if let Action::Screen(action) = action {
            match action {
                ScreenAction::Library => self.focused_screen = 0,
                ScreenAction::Playlists => self.focused_screen = 1,
                ScreenAction::Queue => self.focused_screen = 2,
                ScreenAction::FileBrowser => self.focused_screen = 3,
                ScreenAction::Help => self.focused_screen = 4,
            }
        }
    }
}

impl Drop for App<'_> {
    fn drop(&mut self) {
        log::trace!("App.drop");

        self.queue.quit();

        if let Some(a) = self.player_command_receiver_thread.take() {
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
