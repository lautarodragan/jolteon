use std::{
    cell::{Cell, RefCell},
    env,
    path::PathBuf,
    rc::Rc,
    sync::Weak,
};

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    prelude::{Style, Widget},
    widgets::Block,
};

use crate::{
    actions::{Action, Actions, OnActionMut, ScreenAction},
    components::{FileBrowser, Help, Library, Playlists, Queue as QueueScreen},
    config::Theme,
    main_player::MainPlayer,
    settings::Settings,
    state::State,
    structs::Song,
    ui::{ComponentMut, TopBar},
};

#[derive(Debug)]
pub enum QueueChange {
    AddFront(Song),
    AddBack(Song),
    Append(Vec<Song>),
    Remove(usize),
}

struct Callback<'a, T>(RefCell<Option<Box<dyn Fn(T) + 'a>>>);

impl<'a, T> Callback<'a, T> {
    pub fn call(&self, v: T) {
        self.0.borrow().as_ref().inspect(|f| f(v));
    }

    pub fn set(&self, f: impl Fn(T) + 'a) {
        *self.0.borrow_mut() = Some(Box::new(f));
    }
}

impl<T> Default for Callback<'_, T> {
    fn default() -> Self {
        Self(RefCell::new(None))
    }
}

pub struct Root<'a> {
    settings: Settings,
    theme: Theme,
    frame: u64,

    screens: Vec<(String, Rc<RefCell<dyn 'a + ComponentMut<'a>>>)>,
    focused_screen: usize,
    is_focus_trapped: Rc<Cell<bool>>,

    player: Weak<MainPlayer>,

    queue_screen: Rc<RefCell<QueueScreen<'a>>>,
    browser_screen: Rc<RefCell<FileBrowser<'a>>>,

    on_queue_changed_fn: Rc<Callback<'a, QueueChange>>,
}

impl<'a> Root<'a> {
    pub fn new(actions: &'a Actions, settings: Settings, theme: Theme, player: Weak<MainPlayer>) -> Self {
        let state = State::from_file();

        let current_directory = match &state.last_visited_path {
            Some(s) => PathBuf::from(s),
            None => env::current_dir().unwrap(),
        };

        let is_focus_trapped = Rc::new(Cell::new(false));

        let queue_screen = Rc::new(RefCell::new(QueueScreen::new(state.queue_items.clone(), theme)));
        let library = Rc::new(RefCell::new(Library::new(theme)));
        let playlist = Rc::new(RefCell::new(Playlists::new(theme)));
        let browser = Rc::new(RefCell::new(FileBrowser::new(actions, theme, current_directory)));

        let on_queue_changed_fn = Rc::new(Callback::default());

        {
            let library = library.borrow_mut();
            library.on_enter({
                let queue_screen = queue_screen.clone();
                let on_queue_changed_fn = on_queue_changed_fn.clone();

                move |song| {
                    queue_screen.borrow_mut().append(vec![song.clone()]);
                    on_queue_changed_fn.call(QueueChange::AddBack(song));
                }
            });
            library.on_enter_alt({
                let player = player.clone();
                move |song| {
                    player.upgrade().inspect(|p| p.play(song));
                }
            });
            library.on_select_songs_fn({
                // selected artist/album
                let queue_screen = queue_screen.clone();
                let on_queue_changed_fn = on_queue_changed_fn.clone();

                move |songs| {
                    log::trace!(target: "::app.library", "on_select_songs_fn -> adding songs to queue");
                    let songs: Vec<Song> = songs.into_iter().cloned().collect();
                    queue_screen.borrow_mut().append(songs.clone());
                    on_queue_changed_fn.call(QueueChange::Append(songs));
                }
            });
        }

        {
            let playlist = playlist.borrow_mut();
            playlist.on_enter_song({
                let queue_screen = queue_screen.clone();
                let on_queue_changed_fn = on_queue_changed_fn.clone();
                move |song| {
                    queue_screen.borrow_mut().append(vec![song.clone()]);
                    on_queue_changed_fn.call(QueueChange::AddBack(song));
                }
            });
            playlist.on_enter_song_alt({
                let player = player.clone();
                move |song| {
                    player.upgrade().inspect(|p| p.play(song));
                }
            });
            playlist.on_enter_playlist({
                let queue_screen = queue_screen.clone();
                let on_queue_changed_fn = on_queue_changed_fn.clone();
                move |playlist| {
                    queue_screen.borrow_mut().append(playlist.songs.clone());
                    on_queue_changed_fn.call(QueueChange::Append(playlist.songs));
                }
            });
            playlist.on_request_focus_trap_fn({
                let is_focus_trapped = is_focus_trapped.clone();
                move |v| {
                    is_focus_trapped.set(v);
                }
            });
        }

        {
            let queue_screen = queue_screen.borrow_mut();
            queue_screen.on_enter({
                let player = player.clone();
                move |song| {
                    player.upgrade().inspect(|p| p.play(song));
                }
            });
            queue_screen.on_delete({
                let on_queue_changed_fn = on_queue_changed_fn.clone();
                move |_song, index| {
                    on_queue_changed_fn.call(QueueChange::Remove(index));
                }
            });
        }

        {
            let browser = browser.borrow_mut();
            browser.on_enqueue({
                let queue_screen = queue_screen.clone();
                let on_queue_changed_fn = on_queue_changed_fn.clone();
                move |songs| {
                    queue_screen.borrow_mut().append(songs.clone());
                    on_queue_changed_fn.call(QueueChange::Append(songs));
                }
            });
            browser.on_add_to_lib({
                let library = library.clone();

                move |songs| {
                    library.borrow_mut().add_songs(songs);
                }
            });
            browser.on_add_to_playlist({
                let playlist = playlist.clone();

                move |mut songs| {
                    playlist.borrow_mut().add_songs(&mut songs);
                }
            });
        }

        let help = Rc::new(RefCell::new(Help::new(actions, settings, theme)));

        Self {
            settings,
            theme,
            frame: 0,

            screens: vec![
                ("Library".to_string(), library.clone()),
                ("Playlists".to_string(), playlist.clone()),
                ("Queue".to_string(), queue_screen.clone()),
                ("File Browser".to_string(), browser.clone()),
                ("Help".to_string(), help.clone()),
            ],
            focused_screen: 0,
            is_focus_trapped,

            player,

            queue_screen,
            browser_screen: browser,

            on_queue_changed_fn,
        }
    }

    pub fn browser_directory(&self) -> PathBuf {
        self.browser_screen.borrow().current_directory()
    }

    pub fn on_queue_changed(&self, f: impl Fn(QueueChange) + 'a) {
        self.on_queue_changed_fn.set(f);
    }

    pub fn set_queue(&self, songs: Vec<Song>) {
        self.queue_screen.borrow_mut().set_items(songs);
    }
}

impl OnActionMut for Root<'_> {
    fn on_action(&mut self, action: Vec<Action>) {
        match action[0] {
            Action::Screen(action) if !self.is_focus_trapped.get() => match action {
                ScreenAction::Library => self.focused_screen = 0,
                ScreenAction::Playlists => self.focused_screen = 1,
                ScreenAction::Queue => self.focused_screen = 2,
                ScreenAction::FileBrowser => self.focused_screen = 3,
                ScreenAction::Help => self.focused_screen = 4,
            },
            _ => {
                let mut c = self.screens[self.focused_screen].1.borrow_mut();
                c.on_action(action);
            }
        }
    }
}

impl Widget for &mut Root<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
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
            self.settings,
            self.theme,
            &screen_titles,
            self.focused_screen,
            self.frame,
        );
        top_bar.render(area_top, buf);

        let Some((_, component)) = self.screens.get(self.focused_screen) else {
            log::error!("focused_screen is {}, which is out of bounds.", self.focused_screen);
            return;
        };

        component.borrow().render_ref(area_center, buf);

        let Some(player) = self.player.upgrade() else {
            return;
        };

        let is_paused = player.is_paused() && {
            const ANIM_LEN: u64 = 6 * 16;
            let step = self.frame % ANIM_LEN;
            step % 12 < 6 || step >= ANIM_LEN / 2 // toggle visible/hidden every 6 frames, for half the length of the animation; then stay visible until the end.
        };

        let is_repeating = player.is_repeating();

        crate::ui::CurrentlyPlaying::new(
            self.theme,
            player.playing_song(),
            player.playing_position(),
            self.queue_screen.borrow().duration(),
            self.queue_screen.borrow().len(),
            is_paused,
            is_repeating,
        )
        .render(area_bottom, buf);

        self.frame += 1;
    }
}

impl Drop for Root<'_> {
    fn drop(&mut self) {
        log::trace!("Root.drop");
    }
}
