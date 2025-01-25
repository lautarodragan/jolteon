use crate::actions::{Action, FileBrowserAction, OnAction, OnActionMut};

use super::{AddMode, FileBrowser};

impl OnActionMut for FileBrowser<'_> {
    fn on_action(&mut self, action: Action) {
        match action {
            Action::FileBrowser(action) => match action {
                FileBrowserAction::NavigateUp => {
                    self.navigate_up();
                }
                FileBrowserAction::OpenTerminal => {
                    log::error!("FileBrowserAction::OpenTerminal {action:?} not implemented");
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
            },
            _ => {
                self.focus_group.on_action(action);
            }
        }
    }
}
