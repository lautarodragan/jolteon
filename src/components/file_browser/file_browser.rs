use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    path::PathBuf,
    rc::Rc,
    sync::atomic::AtomicUsize,
};

use crate::{
    components::{
        file_browser::{file_meta::FileMeta, help::FileBrowserHelp},
        List,
    },
    config::Theme,
    structs::Song,
    ui::Focusable,
};

use super::{
    current_directory::CurrentDirectory,
    file_browser_selection::{directory_to_songs_and_folders, FileBrowserSelection},
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
    pub(super) current_directory: Rc<CurrentDirectory>,
    pub(super) on_enqueue_fn: Rc<RefCell<Option<Box<dyn Fn(Vec<Song>) + 'a>>>>,
    pub(super) on_add_to_lib_fn: Rc<RefCell<Option<Box<dyn Fn(Vec<Song>) + 'a>>>>,
    pub(super) on_add_to_playlist_fn: Rc<RefCell<Option<Box<dyn Fn(Vec<Song>) + 'a>>>>,
    pub(super) history: Rc<RefCell<HashMap<PathBuf, (usize, usize)>>>,
    pub(super) focus: AtomicUsize,
    pub(super) add_mode: Rc<Cell<AddMode>>,
    pub(super) help: FileBrowserHelp,
}

impl<'a> FileBrowser<'a> {
    pub fn new(theme: Theme, current_directory: PathBuf) -> Self {
        let items = directory_to_songs_and_folders(&current_directory);
        let parents_list = Rc::new(List::new(theme, items));
        let children_list = Rc::new(List::new(theme, vec![]));
        let file_meta = Rc::new(FileMeta::new(theme));
        let current_directory = Rc::new(CurrentDirectory::new(theme, current_directory));
        let history = Rc::new(RefCell::new(HashMap::new()));
        let on_enqueue_fn: Rc<RefCell<Option<Box<dyn Fn(Vec<Song>) + 'a>>>> = Rc::new(RefCell::new(None));
        let on_add_to_lib_fn: Rc<RefCell<Option<Box<dyn Fn(Vec<Song>) + 'a>>>> = Rc::new(RefCell::new(None));
        let on_add_to_playlist_fn: Rc<RefCell<Option<Box<dyn Fn(Vec<Song>) + 'a>>>> = Rc::new(RefCell::new(None));
        let add_mode = Rc::new(Cell::new(AddMode::AddToLibrary));

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
        children_list.on_enter({
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
        children_list.on_enter_alt({
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

        parents_list.set_is_focused(true);
        parents_list.set_auto_select_next(false);

        parents_list.on_select({
            let children_list = children_list.clone();
            let file_meta = file_meta.clone();
            move |item| {
                if let FileBrowserSelection::Directory(path) = item {
                    let files = directory_to_songs_and_folders(path.as_path());

                    if let Some(f) = files.first() {
                        file_meta.set_file(f.clone());
                    } else {
                        file_meta.clear();
                    }

                    children_list.set_items(files);
                } else {
                    children_list.set_items(vec![]);
                }
            }
        });
        parents_list.on_enter({
            let on_enqueue_fn = Rc::clone(&on_enqueue_fn);
            let current_directory = Rc::clone(&current_directory);
            let parents_list = Rc::clone(&parents_list);
            let children_list = children_list.clone();
            let history = Rc::clone(&history);

            move |item| match item {
                FileBrowserSelection::Directory(path) => {
                    let files = directory_to_songs_and_folders(path.as_path());

                    if !files.iter().any(|f| matches!(f, FileBrowserSelection::Directory(_))) {
                        // UX:
                        //   Do not navigate into a directory if it has no directories inside.
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
                        directory_to_songs_and_folders(path.as_path())
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
        parents_list.on_enter_alt({
            let on_enter_alt_fn = Rc::clone(&on_add_to_lib_fn);

            move |item| {
                let on_enter_alt_fn = on_enter_alt_fn.borrow();
                let Some(on_enter_alt_fn) = &*on_enter_alt_fn else {
                    return;
                };

                match item {
                    FileBrowserSelection::Directory(path) => {
                        let songs = Song::from_dir(path.as_path());
                        on_enter_alt_fn(songs);
                    }
                    FileBrowserSelection::Song(song) => {
                        on_enter_alt_fn(vec![song]);
                    }
                    FileBrowserSelection::CueSheet(cue_sheet) => {
                        let songs = Song::from_cue_sheet(cue_sheet);
                        on_enter_alt_fn(songs);
                    }
                    _ => {}
                }
            }
        });

        Self {
            theme,
            parents_list,
            children_list,
            file_meta,
            current_directory,
            on_enqueue_fn,
            on_add_to_lib_fn,
            on_add_to_playlist_fn,
            history,
            focus: AtomicUsize::new(0),
            add_mode,
            help: FileBrowserHelp::new(theme),
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
        let mut on_add_to_lib_fn = self.on_add_to_lib_fn.borrow_mut();
        *on_add_to_lib_fn = Some(Box::new(cb));
    }

    pub fn on_add_to_playlist(&self, cb: impl Fn(Vec<Song>) + 'a) {
        let mut on_add_to_playlist_fn = self.on_add_to_playlist_fn.borrow_mut();
        *on_add_to_playlist_fn = Some(Box::new(cb));
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

        let parents = directory_to_songs_and_folders(parent);

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
