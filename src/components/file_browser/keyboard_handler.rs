use crate::{
    actions::{Action, FileBrowserAction, OnAction, OnActionMut},
    components::FileBrowserSelection,
    spawn_terminal::spawn_terminal,
};

use super::{AddMode, FileBrowser};

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
            }
        } else {
            self.focus_group.on_action(actions);
        }
    }
}
