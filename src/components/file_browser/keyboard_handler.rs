use std::rc::Rc;

use crate::{
    structs::{Action, FileBrowserAction, NavigationAction, OnAction},
    ui::Focusable,
};

use super::{AddMode, FileBrowser};

impl OnAction for FileBrowser<'_> {
    fn on_action(&self, action: Action) {
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
            Action::Navigation(NavigationAction::FocusNext) | Action::Navigation(NavigationAction::FocusPrevious) => {
                let children_components = &self.children_components;

                if children_components.len() < 2 {
                    return;
                }

                let mut current_focus = self.focused_child.borrow_mut();

                let mut focus = 0;
                for i in 0..children_components.len() {
                    if Rc::ptr_eq(&children_components[i], &*current_focus) {
                        focus = i;
                    }
                }

                if action == Action::Navigation(NavigationAction::FocusNext) {
                    if focus > 1 {
                        focus = 0
                    } else {
                        focus += 1;
                    }
                } else if action == Action::Navigation(NavigationAction::FocusPrevious) {
                    if focus == 0 {
                        focus = children_components.len() - 1
                    } else {
                        focus -= 1;
                    }
                }

                for i in 0..children_components.len() {
                    children_components[i].set_is_focused(i == focus);
                    if i == focus {
                        *current_focus = children_components[i].clone()
                    }
                }
            }
            _ => {
                let current_focus = self.focused_child.borrow_mut();
                current_focus.on_action(action);
            }
        }
    }
}
