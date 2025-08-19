use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    path::PathBuf,
    rc::Rc,
    sync::{
        Arc,
        Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::{RecvTimeoutError, channel},
    },
    thread,
    time::Duration,
};

use super::{
    current_directory::CurrentDirectory,
    file_browser_selection::{FileBrowserSelection, directory_to_songs_and_folders},
};
use crate::{
    actions::Actions,
    components::{
        FocusGroup,
        List,
        file_browser::{file_meta::FileMeta, help::FileBrowserHelp},
    },
    structs::Song,
    theme::Theme,
    ui::{Component, Focusable},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AddMode {
    AddToLibrary,
    AddToPlaylist,
}

pub struct FileBrowser<'a> {
    #[allow(unused)]
    pub(super) theme: Theme,

    pub(super) parents_list: Rc<List<'a, FileBrowserSelection>>,
    pub(super) children_list: Rc<List<'a, FileBrowserSelection>>,
    pub(super) file_meta: Rc<FileMeta<'a>>,
    pub(super) help: FileBrowserHelp<'a>,
    pub(super) focus_group: FocusGroup<'a>,

    pub(super) files_from_io_thread: Arc<Mutex<Vec<FileBrowserSelection>>>,

    pub(super) history: Rc<RefCell<HashMap<PathBuf, (usize, usize)>>>,

    pub(super) current_directory: Rc<CurrentDirectory>,
    pub(super) on_enqueue_fn: Rc<RefCell<Option<Box<dyn Fn(Vec<Song>) + 'a>>>>,
    pub(super) on_add_to_lib_fn: Rc<RefCell<Option<Box<dyn Fn(Vec<Song>) + 'a>>>>,
    pub(super) on_add_to_playlist_fn: Rc<RefCell<Option<Box<dyn Fn(Vec<Song>) + 'a>>>>,
    pub(super) add_mode: Rc<Cell<AddMode>>,

    pub(super) show_hidden_files: Arc<AtomicBool>,
}

impl<'a> FileBrowser<'a> {
    pub fn new(actions: &'a Actions, theme: Theme, current_directory: PathBuf) -> Self {
        let show_hidden_files = Arc::new(AtomicBool::new(false));
        let items = directory_to_songs_and_folders(&current_directory, show_hidden_files.load(Ordering::Acquire));
        let mut children_list = List::new(theme, vec![]);
        let file_meta = Rc::new(FileMeta::new(theme));
        let current_directory = Rc::new(CurrentDirectory::new(theme, current_directory));
        let history = Rc::new(RefCell::new(HashMap::new()));
        let on_enqueue_fn: Rc<RefCell<Option<Box<dyn Fn(Vec<Song>) + 'a>>>> = Rc::new(RefCell::new(None));
        let on_add_to_lib_fn: Rc<RefCell<Option<Box<dyn Fn(Vec<Song>) + 'a>>>> = Rc::new(RefCell::new(None));
        let on_add_to_playlist_fn: Rc<RefCell<Option<Box<dyn Fn(Vec<Song>) + 'a>>>> = Rc::new(RefCell::new(None));
        let add_mode = Rc::new(Cell::new(AddMode::AddToLibrary));

        let (io_thread, files_from_io_thread) = {
            let (tx, rx) = channel::<PathBuf>();
            let files_from_io_thread = Arc::new(Mutex::new(vec![]));
            let show_hidden_files = Arc::clone(&show_hidden_files);
            thread::spawn({
                let files_from_io_thread = Arc::clone(&files_from_io_thread);
                move || loop {
                    let Ok(mut path) = rx.recv() else {
                        log::trace!("FileBrowser's IO thread will close now.");
                        break;
                    };
                    log::trace!("FileBrowser's IO thread: received {path:?}");
                    let path = loop {
                        match rx.recv_timeout(Duration::from_millis(100)) {
                            Ok(p) => {
                                log::trace!("FileBrowser's IO thread: debounced path {p:?}");
                                path = p;
                            }
                            Err(RecvTimeoutError::Timeout) => {
                                log::trace!("FileBrowser's IO thread: will now process path {path:?}");
                                break path;
                            }
                            _ => {
                                log::trace!("FileBrowser's IO thread (inner loop) will close now.");
                                return;
                            }
                        }
                    };
                    let files =
                        directory_to_songs_and_folders(path.as_path(), show_hidden_files.load(Ordering::Acquire));
                    *files_from_io_thread.lock().unwrap() = files;
                }
            });
            (tx, files_from_io_thread)
        };

        children_list.line_style(|i| match i {
            FileBrowserSelection::Song(_) | FileBrowserSelection::CueSheet(_) => None,
            _ => Some(ratatui::style::Style::new().add_modifier(ratatui::style::Modifier::DIM)),
        });
        children_list.on_select({
            let file_meta = file_meta.clone();
            move |s| {
                file_meta.set_file(s);
            }
        });
        children_list.on_confirm({
            let on_enqueue_fn = Rc::clone(&on_enqueue_fn);

            move |item| {
                let on_enqueue_fn = on_enqueue_fn.borrow();
                let Some(on_enqueue_fn) = &*on_enqueue_fn else {
                    return;
                };

                match item {
                    FileBrowserSelection::Song(song) => {
                        on_enqueue_fn(vec![song]);
                    }
                    FileBrowserSelection::CueSheet(cue) => {
                        let songs = Song::from_cue_sheet(cue);
                        on_enqueue_fn(songs);
                    }
                    _ => {}
                }
            }
        });
        children_list.on_confirm_alt({
            let on_add_to_lib_fn = Rc::clone(&on_add_to_lib_fn);
            let on_add_to_playlist_fn = Rc::clone(&on_add_to_playlist_fn);
            let mode = Rc::clone(&add_mode);

            move |item| {
                let cb = if mode.get() == AddMode::AddToLibrary {
                    on_add_to_lib_fn.borrow()
                } else {
                    on_add_to_playlist_fn.borrow()
                };

                let Some(cb) = &*cb else {
                    return;
                };

                match item {
                    FileBrowserSelection::Song(song) => {
                        cb(vec![song]);
                    }
                    FileBrowserSelection::CueSheet(cue_sheet) => {
                        let songs = Song::from_cue_sheet(cue_sheet);
                        cb(songs);
                    }
                    _ => {}
                }
            }
        });

        {
            // TODO: duplicated code from parents_list.on_select(...).
            //   Must create a FileList component that wraps a List and has a .set_directory etc.
            if let Some(first_parent) = items.first()
                && let FileBrowserSelection::Directory(path) = first_parent
            {
                let files = directory_to_songs_and_folders(path.as_path(), show_hidden_files.load(Ordering::Acquire));

                if let Some(f) = files.first() {
                    file_meta.set_file(f.clone());
                } else {
                    file_meta.clear();
                }

                children_list.set_items(files);
            }
        }

        let mut parents_list = List::new(theme, items);
        parents_list.set_auto_select_next(true); // TODO: only on confirm_alt

        let children_list = Rc::new(children_list);
        parents_list.on_select({
            let children_list = children_list.clone();
            // let file_meta = file_meta.clone();
            move |item| {
                if let FileBrowserSelection::Directory(path) = item {
                    if let Err(err) = io_thread.send(path.clone()) {
                        log::error!("FileBrowser: error sending path to IO thread {err:?}");
                    };
                } else {
                    children_list.set_items(vec![]);
                }
            }
        });

        let parents_list = Rc::new(parents_list);

        parents_list.on_confirm({
            let on_enqueue_fn = Rc::clone(&on_enqueue_fn);
            let current_directory = Rc::clone(&current_directory);
            let parents_list = Rc::downgrade(&parents_list);
            let children_list = children_list.clone();
            let history = Rc::clone(&history);
            let show_hidden_files = Arc::clone(&show_hidden_files);

            move |item| match item {
                FileBrowserSelection::Directory(path) => {
                    let Some(parents_list) = parents_list.upgrade() else {
                        return;
                    };

                    let files =
                        directory_to_songs_and_folders(path.as_path(), show_hidden_files.load(Ordering::Acquire));

                    if !files.iter().any(|f| matches!(f, FileBrowserSelection::Directory(_))) {
                        // UX:
                        //   Forbid navigating into a directory if it has no directories inside.
                        //   Use the right-side list to operate on its children instead.
                        return;
                    }

                    let mut history = history.borrow_mut();

                    // UX:
                    //   Save the current selected index and scroll position, associated with each directory.
                    history.insert(
                        current_directory.path(),
                        (parents_list.selected_index(), parents_list.scroll_position()),
                    );

                    // UX:
                    //   Automatically select the child of `path` that was last selected when `path` was last displayed.
                    let (selected_child, scroll_position) = history.get(&path).cloned().unwrap_or_default();

                    let children = if let Some(FileBrowserSelection::Directory(path)) = files.get(selected_child) {
                        directory_to_songs_and_folders(path.as_path(), show_hidden_files.load(Ordering::Acquire))
                    } else {
                        vec![]
                    };
                    children_list.set_items(children);

                    parents_list.set_items_s(files, selected_child, scroll_position);

                    current_directory.set_path(path);
                }
                FileBrowserSelection::Song(song) => {
                    let on_enqueue_fn = on_enqueue_fn.borrow();
                    if let Some(on_enqueue_fn) = &*on_enqueue_fn {
                        on_enqueue_fn(vec![song]);
                    }
                }
                FileBrowserSelection::CueSheet(cue_sheet) => {
                    let on_enqueue_fn = on_enqueue_fn.borrow();
                    if let Some(on_enqueue_fn) = &*on_enqueue_fn {
                        let songs = Song::from_cue_sheet(cue_sheet);
                        on_enqueue_fn(songs);
                    }
                }
                _ => {}
            }
        });
        parents_list.on_confirm_alt({
            let mode = Rc::clone(&add_mode);
            let on_add_to_lib_fn = Rc::clone(&on_add_to_lib_fn);
            let on_add_to_playlist_fn = Rc::clone(&on_add_to_playlist_fn);

            move |item| {
                let cb = if mode.get() == AddMode::AddToLibrary {
                    on_add_to_lib_fn.borrow()
                } else {
                    on_add_to_playlist_fn.borrow()
                };

                let Some(cb) = &*cb else {
                    return;
                };

                match item {
                    FileBrowserSelection::Directory(path) => {
                        let songs = Song::from_dir(path.as_path());
                        cb(songs);
                    }
                    FileBrowserSelection::Song(song) => {
                        cb(vec![song]);
                    }
                    FileBrowserSelection::CueSheet(cue_sheet) => {
                        let songs = Song::from_cue_sheet(cue_sheet);
                        cb(songs);
                    }
                    _ => {}
                }
            }
        });

        let focus_group = FocusGroup::new(vec![
            Component::Ref(parents_list.clone()),
            Component::Ref(children_list.clone()),
            Component::Ref(file_meta.clone()),
        ]);

        Self {
            theme,

            parents_list,
            children_list,
            file_meta,
            focus_group,

            files_from_io_thread,

            current_directory,
            on_enqueue_fn,
            on_add_to_lib_fn,
            on_add_to_playlist_fn,
            history,
            add_mode,
            help: FileBrowserHelp::new(actions, theme),

            show_hidden_files,
        }
    }

    #[allow(dead_code)]
    pub fn blur(&mut self) {
        unimplemented!();
    }

    #[allow(dead_code)]
    pub fn focus(&mut self) {
        unimplemented!();
    }

    pub fn on_enqueue(&self, cb: impl Fn(Vec<Song>) + 'a) {
        let mut on_enqueue_fn = self.on_enqueue_fn.borrow_mut();
        *on_enqueue_fn = Some(Box::new(cb));
    }

    pub fn on_add_to_lib(&self, cb: impl Fn(Vec<Song>) + 'a) {
        *self.on_add_to_lib_fn.borrow_mut() = Some(Box::new(cb));
    }

    pub fn on_add_to_playlist(&self, cb: impl Fn(Vec<Song>) + 'a) {
        *self.on_add_to_playlist_fn.borrow_mut() = Some(Box::new(cb));
    }

    pub fn navigate_up(&self) {
        let current_directory = self.current_directory.path();

        let Some(parent) = current_directory.parent() else {
            return;
        };

        self.parents_list.with_items(|parents| {
            let parents: Vec<_> = parents.into_iter().cloned().collect();

            if let Some(f) = parents.first() {
                self.file_meta.set_file(f.clone());
            } else {
                self.file_meta.clear();
            }

            self.children_list.set_items(parents);
        });

        let mut history = self.history.borrow_mut();
        history.insert(
            current_directory.clone(),
            (self.parents_list.selected_index(), self.parents_list.scroll_position()),
        );
        let history_entry = history.get(parent).cloned();

        let parents = directory_to_songs_and_folders(parent, self.show_hidden_files.load(Ordering::Acquire));

        let (selected_parent_index, selected_parent_scroll) = history_entry.unwrap_or({
            let selected_parent_index = parents.iter().position(|item| {
                let FileBrowserSelection::Directory(path) = item else {
                    return false;
                };
                current_directory
                    .to_string_lossy()
                    .contains(path.to_string_lossy().to_string().as_str())
            });
            (selected_parent_index.unwrap_or(0), 0)
        });

        self.parents_list
            .set_items_s(parents, selected_parent_index, selected_parent_scroll);

        self.current_directory.set_path(parent.to_path_buf());

        self.focus_group.focus_nth(0);
    }

    pub fn current_directory(&self) -> PathBuf {
        self.current_directory.path()
    }
}

impl Drop for FileBrowser<'_> {
    fn drop(&mut self) {
        log::trace!("FileBrowser.drop()");
    }
}

impl Focusable for FileBrowser<'_> {}
