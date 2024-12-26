use std::{
    cell::{Cell, RefCell},
    env,
    error::Error,
    path::PathBuf,
    rc::Rc,
    sync::Arc,
    time::Duration,
};

use async_std::task;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    prelude::{Style, Widget},
    widgets::{Block, WidgetRef},
};
use rodio::OutputStream;

use crate::{
    components::{FileBrowser, Help, Library, Playlists, Queue as QueueScreen},
    config::Theme,
    main_player::MainPlayer,
    mpris::Mpris,
    player::SingleTrackPlayer,
    state::State,
    structs::{Action, Actions, OnAction, OnActionMut, Queue, ScreenAction},
    term::set_terminal,
    ui::{Component, KeyboardHandlerMut, KeyboardHandlerRef, TopBar},
};

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

    main_player: RefCell<Option<MainPlayer>>,
    player: Arc<SingleTrackPlayer>,
    queue: Arc<Queue>,

    queue_ui: Rc<QueueScreen<'a>>,
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
        let player = Arc::new(SingleTrackPlayer::new(output_stream_handle, mpris.clone()));
        let queue_ui = Rc::new(QueueScreen::new(state.queue_items.clone(), theme));
        let queue = Arc::new(Queue::new(state.queue_items));
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

        queue_ui.on_enter({
            let player = player.clone();
            let queue = queue.clone();

            move |song| {
                queue.add_front(song);
                player.stop();
            }
        });
        queue_ui.on_delete({
            let queue = queue.clone();

            move |_song, index| {
                queue.remove(index);
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

        let help = Rc::new(RefCell::new(Help::new(theme)));

        Self {
            must_quit: false,

            theme,
            actions,
            frame: 0,

            output_stream,
            mpris,

            screens: vec![
                ("Library".to_string(), Component::Ref(library.clone())),
                ("Playlists".to_string(), Component::Ref(playlist.clone())),
                ("Queue".to_string(), Component::Ref(queue_ui.clone())),
                ("File Browser".to_string(), Component::Ref(browser.clone())),
                ("Help".to_string(), Component::Mut(help.clone())),
            ],
            focused_screen: 0,
            is_focus_trapped,

            main_player: RefCell::new(None),
            player,
            queue,

            queue_ui,
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

        let player_something = MainPlayer::spawn(self.player.clone(), self.queue.clone());

        *self.main_player.borrow_mut() = Some(player_something);

        while !self.must_quit {
            if self.queue.length() != self.queue_ui.len() {
                // See src/README.md to make sense of this
                self.queue.with_items(|songs| {
                    self.queue_ui.set_items(Vec::from(songs.clone()));
                });
            }

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

        let player_something = self.main_player.borrow_mut().take().unwrap();

        player_something.quit();

        Ok(())
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
                    Action::Screen(action) => {
                        self.on_action(action);
                        return;
                    }
                    Action::Player(action) => {
                        self.player.on_action(action);
                        return;
                    }
                    Action::MainPlayer(action) => {
                        self.main_player.borrow().as_ref().unwrap().on_action(action);
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
            Component::Ref(ref s) => {
                s.render_ref(area_center, buf);
            }
            Component::Mut(ref s) => {
                s.borrow().render_ref(area_center, buf);
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

impl OnActionMut<ScreenAction> for App<'_> {
    fn on_action(&mut self, action: ScreenAction) {
        match action {
            ScreenAction::Library => self.focused_screen = 0,
            ScreenAction::Playlists => self.focused_screen = 1,
            ScreenAction::Queue => self.focused_screen = 2,
            ScreenAction::FileBrowser => self.focused_screen = 3,
            ScreenAction::Help => self.focused_screen = 4,
        }
    }
}

impl Drop for App<'_> {
    fn drop(&mut self) {
        log::trace!("App.drop");
    }
}

pub async fn run() -> Result<(), Box<dyn Error>> {
    let mpris = Mpris::new().await?;

    task::spawn_blocking(move || {
        let mut app = App::new(mpris);
        if let Err(err) = app.start() {
            // dyn Error cannot be shared between threads.
            // TODO: stop using dyn Error. Use an Enum instead.
            log::error!("app.start error :( \n{:#?}", err);
        }
    })
    .await;

    Ok(())
}
