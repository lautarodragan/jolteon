use std::sync::atomic::Ordering;

use super::{AddMode, FileBrowser};
use crate::{
    actions::{Action, FileBrowserAction, OnAction, OnActionMut},
    components::{FileBrowserSelection, directory_to_songs_and_folders},
    spawn_terminal::spawn_terminal,
};

impl OnActionMut for FileBrowser<'_> {
    fn on_action(&mut self, actions: Vec<Action>) {
        // log::debug!("FB action {actions:?}");

        if self.parents_list.filter().is_empty()
            && let Some(action) = actions.iter().find_map(|action| match action {
                Action::FileBrowser(a) => Some(a),
                _ => None,
            })
        {
            match action {
                FileBrowserAction::NavigateUp => {
                    self.navigate_up();
                }
                FileBrowserAction::OpenTerminal => {
                    let path = self
                        .parents_list
                        .with_selected_item(|item| match item {
                            FileBrowserSelection::Directory(dir) => Some(dir.clone()),
                            _ => None,
                        })
                        .unwrap_or(self.current_directory.path());
                    log::info!("FileBrowserAction::OpenTerminal at {path:?}");
                    spawn_terminal(path);
                }
                FileBrowserAction::AddToLibrary => {
                    log::error!("FileBrowserAction::AddToLibrary not implemented");
                }
                FileBrowserAction::AddToQueue => {
                    log::error!("FileBrowserAction::AddToQueue not implemented");
                }
                FileBrowserAction::AddToPlaylist => {
                    log::error!("FileBrowserAction::AddToPlaylist not implemented");
                }
                FileBrowserAction::ToggleMode => {
                    self.add_mode.set(match self.add_mode.get() {
                        AddMode::AddToLibrary => AddMode::AddToPlaylist,
                        AddMode::AddToPlaylist => AddMode::AddToLibrary,
                    });
                    self.help.set_add_mode(self.add_mode.get());
                }
                FileBrowserAction::ToggleShowHidden => {
                    // TODO: this is mostly duplicated code.
                    //   It'd probably make more sense to avoid IO altogether and just
                    //   for item in list { item.set_is_visible(show_hidden_files || !item.to_string().starts_with('.')) }

                    let show_hidden_files = !self.show_hidden_files.load(Ordering::Acquire);
                    let path = self.current_directory.path();
                    let files = directory_to_songs_and_folders(path.as_path(), show_hidden_files);

                    let history = self.history.borrow_mut();

                    // UX:
                    //   Automatically select the child of `path` that was last selected when `path` was last displayed.
                    let (selected_child, scroll_position) = history.get(&path).cloned().unwrap_or_default();

                    self.parents_list.set_items_s(files, selected_child, scroll_position);
                    self.show_hidden_files.store(show_hidden_files, Ordering::Release);
                }
            }
        } else {
            self.focus_group.on_action(actions);
        }
    }
}
