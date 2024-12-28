use std::{
    cell::{Cell, RefCell},
    env,
    path::PathBuf,
    rc::Rc,
    sync::Arc,
};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    prelude::{Style, Widget},
    widgets::{Block, WidgetRef},
};

use crate::{
    components::{FileBrowser, Help, Library, Playlists, Queue as QueueScreen},
    config::Theme,
    main_player::MainPlayer,
    player::SingleTrackPlayer,
    state::State,
    structs::{Actions, OnActionMut, Queue, ScreenAction},
    ui::{Component, KeyboardHandlerMut, KeyboardHandlerRef, TopBar},
};

pub struct Root<'a> {
    theme: Theme,
    frame: u64,

    screens: Vec<(String, Component<'a>)>,
    focused_screen: usize,
    is_focus_trapped: Rc<Cell<bool>>,

    main_player: RefCell<Option<MainPlayer>>,
    player: Arc<SingleTrackPlayer>,
    queue: Arc<Queue>,

    pub queue_ui: Rc<QueueScreen<'a>>,
    pub browser: Rc<FileBrowser<'a>>,
}

impl Root<'_> {
    pub fn new(theme: Theme, queue: Arc<Queue>, player: Arc<SingleTrackPlayer>) -> Self {
        let state = State::from_file();

        let current_directory = match &state.last_visited_path {
            Some(s) => PathBuf::from(s),
            None => env::current_dir().unwrap(),
        };

        let is_focus_trapped = Rc::new(Cell::new(false));

        let queue_ui = Rc::new(QueueScreen::new(state.queue_items.clone(), theme));
        let library = Rc::new(Library::new(theme));
        let playlist = Rc::new(Playlists::new(theme));
        let browser = Rc::new(FileBrowser::new(theme, current_directory));

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
            theme,
            frame: 0,

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
}

impl OnActionMut<ScreenAction> for Root<'_> {
    fn on_action(&mut self, action: ScreenAction) {
        if self.is_focus_trapped.get() {
            return;
        }
        match action {
            ScreenAction::Library => self.focused_screen = 0,
            ScreenAction::Playlists => self.focused_screen = 1,
            ScreenAction::Queue => self.focused_screen = 2,
            ScreenAction::FileBrowser => self.focused_screen = 3,
            ScreenAction::Help => self.focused_screen = 4,
        }
    }
}

impl<'a> KeyboardHandlerMut<'a> for Root<'a> {
    fn on_key(&mut self, key: KeyEvent) {
        let Some((_, component)) = self.screens.get(self.focused_screen) else {
            log::error!("focused_screen is {}, which is out of bounds.", self.focused_screen);
            return;
        };

        component.on_key(key);
    }
}

impl WidgetRef for &Root<'_> {
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

impl Drop for Root<'_> {
    fn drop(&mut self) {
        log::trace!("App.drop");
    }
}
