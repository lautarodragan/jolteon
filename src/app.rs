use std::{
    cell::Cell,
    env,
    error::Error,
    path::PathBuf,
    rc::Rc,
    sync::{
        mpsc::{channel, Sender},
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
    mpris::Mpris,
    player::Player,
    state::State,
    structs::{Action, Actions, OnAction, OnActionMut, ScreenAction},
    term::set_terminal,
    ui::{Component, KeyboardHandlerMut, KeyboardHandlerRef, TopBar},
};

enum SongPlayerThreadCommand {
    Quit,
    PlaybackEnded,
    QueueSongAdded,
}

pub struct App<'a> {
    must_quit: bool,

    theme: Theme,
    actions: Actions,
    frame: u64,

    #[allow(unused)]
    output_stream: OutputStream,
    #[allow(unused)]
    mpris: Arc<Mpris>,

    screens: Vec<(String, Component<'a>)>,
    focused_screen: usize,
    is_focus_trapped: Rc<Cell<bool>>,

    player: Arc<Player>,
    queue: Arc<Queue>,
    browser: Rc<FileBrowser<'a>>,
}

impl App<'_> {
    pub fn new(mpris: Mpris) -> Self {
        let state = State::from_file();
        let actions = Actions::from_file_or_default();
        assert!(
            actions.contains(Action::Quit),
            "No key binding for Action::Quit! User would not be able to exit Jolteon. This is 100% a bug."
        );

        let (output_stream, output_stream_handle) = OutputStream::try_default().unwrap(); // Indirectly this spawns the cpal_alsa_out thread, and creates the mixer tied to it

        let theme = include_str!("../assets/theme.toml");
        let theme: Theme = toml::from_str(theme).unwrap();

        let current_directory = match &state.last_visited_path {
            Some(s) => PathBuf::from(s),
            None => env::current_dir().unwrap(),
        };

        let is_focus_trapped = Rc::new(Cell::new(false));

        let mpris = Arc::new(mpris);
        let player = Arc::new(Player::new(output_stream_handle, mpris.clone()));
        let queue = Arc::new(Queue::new(state.queue_items, theme));
        let library = Rc::new(Library::new(theme));
        let playlist = Rc::new(Playlists::new(theme));
        let browser = Rc::new(FileBrowser::new(theme, current_directory));

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

        library.on_enter({
            let queue = queue.clone();

            move |song| {
                queue.add_back(song);
            }
        });
        library.on_enter_alt({
            let player = player.clone();
            let queue = queue.clone();
            move |song| {
                queue.add_front(song);
                player.stop();
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

        playlist.on_enter_song({
            let queue = queue.clone();
            move |song| {
                queue.add_back(song);
            }
        });
        playlist.on_enter_song_alt({
            let player = player.clone();
            let queue = queue.clone();
            move |song| {
                queue.add_front(song);
                player.stop();
            }
        });
        playlist.on_enter_playlist({
            let queue = queue.clone();
            move |playlist| {
                queue.append(&mut std::collections::VecDeque::from(playlist.songs));
            }
        });
        playlist.on_request_focus_trap_fn({
            let is_focus_trapped = is_focus_trapped.clone();
            move |v| {
                is_focus_trapped.set(v);
            }
        });

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
            frame: 0,

            output_stream,
            mpris,

            screens: vec![
                ("Library".to_string(), Component::RefRc(library.clone())),
                ("Playlists".to_string(), Component::RefRc(playlist.clone())),
                ("Queue".to_string(), Component::RefArc(queue.clone())),
                ("File Browser".to_string(), Component::RefRc(browser.clone())),
                ("Help".to_string(), Component::Mut(help.clone())),
            ],
            focused_screen: 0,
            is_focus_trapped,

            player,
            queue,
            browser,
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

        self.player.spawn();
        let (queue_thread, queue_thread_sender) = self.spawn_song_player();

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
                self.frame += 1;
            }
        }

        log::trace!("App.start() -> exiting");

        self.to_state().to_file()?;
        // self.actions.to_file();

        queue_thread_sender.send(SongPlayerThreadCommand::Quit).unwrap();
        if let Err(err) = queue_thread.join() {
            log::error!("error joining queue_thread {err:?}");
        };

        Ok(())
    }

    fn spawn_song_player(&self) -> (JoinHandle<()>, Sender<SongPlayerThreadCommand>) {
        let queue = self.queue.clone();
        let player = self.player.clone();
        let (tx, rx) = channel::<SongPlayerThreadCommand>();

        player.on_playback_end({
            let tx = tx.clone();
            move |song| {
                log::debug!("player.on_playback_end called {song:?}");
                tx.send(SongPlayerThreadCommand::PlaybackEnded).unwrap();
            }
        });

        queue.on_queue_changed({
            let tx = tx.clone();
            move || {
                tx.send(SongPlayerThreadCommand::QueueSongAdded).unwrap();
            }
        });

        let t = thread::Builder::new()
            .name("song_player".to_string())
            .spawn(move || loop {
                let song = queue.pop();
                if let Some(song) = song {
                    log::debug!("song_player grabbed song from queue {song:?}");
                    player.play_song(song);
                }
                loop {
                    match rx.recv().unwrap() {
                        SongPlayerThreadCommand::Quit => {
                            return;
                        }
                        SongPlayerThreadCommand::PlaybackEnded => {
                            break;
                        }
                        SongPlayerThreadCommand::QueueSongAdded => {
                            if player.currently_playing().lock().unwrap().is_none() {
                                break;
                            }
                        }
                    }
                }
            })
            .unwrap();
        (t, tx)
    }
}

impl<'a> KeyboardHandlerMut<'a> for App<'a> {
    fn on_key(&mut self, key: KeyEvent) {
        let action = self.actions.action_by_key(key);

        log::debug!("app.on_key action=('{:?}')", action);

        if let Some(action) = action {
            if action == Action::Quit {
                self.must_quit = true;
                return;
            }
            if !self.is_focus_trapped.get() {
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

impl WidgetRef for &App<'_> {
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

        let top_bar = TopBar::new(self.theme, &screen_titles, self.focused_screen, self.frame);
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

        let frame = self.frame;

        // let is_paused = self.player.is_paused() && {
        //     const ANIM_LEN: u64 = 6 * 16;
        //     let step = (frame - self.paused_animation_start_frame.load(Ordering::Relaxed)) % (ANIM_LEN);
        //     step % 12 < 6 || step >= 6 * 8 // toggle visible/hidden every 6 frames, for half the length of the animation; then stay visible until the end.
        // };

        let is_paused = self.player.is_paused();

        crate::ui::CurrentlyPlaying::new(
            self.theme,
            self.player.currently_playing().lock().unwrap().clone(),
            self.player.get_pos(),
            self.queue.total_time(),
            self.queue.length(),
            is_paused,
        )
        .render(area_bottom, buf);
    }
}

impl OnActionMut for App<'_> {
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
    }
}
