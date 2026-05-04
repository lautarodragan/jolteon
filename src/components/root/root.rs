use std::{
    cell::{Cell, RefCell},
    env,
    path::PathBuf,
    rc::Rc,
    sync::Weak,
};

use crate::{
    actions::Actions,
    components::{
        FileBrowser,
        Help,
        Library,
        Playlists,
        Queue as QueueScreen,
        Soundtracks,
        query::{CommandLine, Query, QueryAddSongsTarget},
    },
    main_player::MainPlayer,
    settings::Settings,
    state::State,
    structs::Song,
    theme::Theme,
    ui::ComponentMut,
};

#[derive(Debug)]
pub enum QueueChange {
    AddFront(Song),
    AddBack(Song),
    Append(Vec<Song>),
    Remove(usize),
}

pub struct Callback<'a, T>(RefCell<Option<Box<dyn Fn(T) + 'a>>>);

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
    pub(super) settings: Settings,
    pub(super) theme: Theme,
    pub(super) frame: u64,

    pub(super) screens: Vec<(String, Rc<RefCell<dyn 'a + ComponentMut<'a>>>)>,
    pub(super) focused_screen: usize,
    pub(super) is_focus_trapped: Rc<Cell<bool>>,

    pub(super) player: Weak<MainPlayer>,
    pub(super) command_line: Rc<RefCell<CommandLine<'a>>>,

    pub(super) queue_screen: Rc<RefCell<QueueScreen<'a>>>,
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
        let soundtracks = Rc::new(RefCell::new(Soundtracks::new(theme)));
        let playlist = Rc::new(RefCell::new(Playlists::new(theme)));
        let browser = Rc::new(RefCell::new(FileBrowser::new(actions, theme, current_directory)));
        let command_line = Rc::new(RefCell::new(CommandLine::new(theme)));

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
            let soundtracks = soundtracks.borrow_mut();
            soundtracks.on_enter({
                let queue_screen = queue_screen.clone();
                let on_queue_changed_fn = on_queue_changed_fn.clone();

                move |song| {
                    queue_screen.borrow_mut().append(vec![song.clone()]);
                    on_queue_changed_fn.call(QueueChange::AddBack(song));
                }
            });
            soundtracks.on_enter_alt({
                let player = player.clone();
                move |song| {
                    player.upgrade().inspect(|p| p.play(song));
                }
            });
            soundtracks.on_select_songs_fn({
                // selected artist/album
                let queue_screen = queue_screen.clone();
                let on_queue_changed_fn = on_queue_changed_fn.clone();

                move |songs| {
                    log::trace!(target: "::app.soundtracks", "on_select_songs_fn -> adding songs to queue");
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
                    let qs = queue_screen.borrow_mut();
                    if qs.with_items(|items| items.last().is_some_and(|last| last.path == song.path)) {
                        // ux: "debounce" repeat appends of the last song. TODO: debounce timeout
                        // better ux would be to "reset" debounce on key up
                        return;
                    }
                    qs.append(vec![song.clone()]);
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
            // TODO: file_browser shouldn't know there exists a queue, lib, etc.
            //   it should just have a "song(s) confirmed / confirmed_alt" event,
            //   and root should be responsible of what to do with that event.
            browser.on_enqueue({
                let queue_screen = queue_screen.clone();
                let on_queue_changed_fn = on_queue_changed_fn.clone();
                move |songs| {
                    queue_screen.borrow_mut().append(songs.clone());
                    on_queue_changed_fn.call(QueueChange::Append(songs));
                }
            });
            browser.on_add_to_lib({
                let command_line = Rc::clone(&command_line);
                move |songs| {
                    command_line.borrow_mut().set_query(Some(Query::AddSongs {
                        songs,
                        target: QueryAddSongsTarget::Library,
                    }));
                }
            });
        }

        let help = Rc::new(RefCell::new(Help::new(actions, settings, theme)));

        {
            let command_line = command_line.borrow();
            let library = Rc::clone(&library);
            let soundtracks = Rc::clone(&soundtracks);
            let playlist = Rc::clone(&playlist);

            command_line.on_confirm({
                move |query| match query {
                    Query::AddSongs { songs, target } => match target {
                        QueryAddSongsTarget::Library => {
                            let library = library.borrow_mut();
                            library.add_songs(songs);
                        }
                        QueryAddSongsTarget::Soundtracks => {
                            let soundtracks = soundtracks.borrow_mut();
                            soundtracks.add_songs(songs);
                        }
                        QueryAddSongsTarget::Playlist => {
                            let playlist = playlist.borrow_mut();
                            playlist.add_songs(songs);
                        }
                    },
                }
            });
        }

        Self {
            settings,
            theme,
            frame: 0,

            screens: vec![
                ("Library".to_string(), library.clone()),
                ("Soundtracks".to_string(), soundtracks.clone()),
                ("Playlists".to_string(), playlist.clone()),
                ("Queue".to_string(), queue_screen.clone()),
                ("File Browser".to_string(), browser.clone()),
                ("Help".to_string(), help.clone()),
            ],
            focused_screen: 0,
            is_focus_trapped,
            command_line,

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

impl Drop for Root<'_> {
    fn drop(&mut self) {
        log::trace!("Root.drop");
    }
}
