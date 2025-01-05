use std::sync::atomic::Ordering;

use crossterm::event::KeyEvent;

use crate::structs::{Action, FileBrowserAction, NavigationAction, OnAction};
use crate::ui::KeyboardHandlerRef;

use super::{AddMode, FileBrowser};

impl<'a> KeyboardHandlerRef<'a> for FileBrowser<'a> {
    fn on_key(&self, key: KeyEvent) {
        log::error!("KeyboardHandlerRef called for file_browser!");
    }
}

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
                let mut focus = self.focus.load(Ordering::Acquire);

                if action == Action::Navigation(NavigationAction::FocusNext) {
                    if focus > 1 {
                        focus = 0
                    } else {
                        focus += 1;
                    }
                } else if action == Action::Navigation(NavigationAction::FocusPrevious) {
                    if focus == 0 {
                        focus = 2
                    } else {
                        focus -= 1;
                    }
                }

                self.focus.store(focus, Ordering::Release);

                if focus == 0 {
                    self.parents_list.focus();
                    self.children_list.blur();
                    self.file_meta.blur();
                } else if focus == 1 {
                    self.parents_list.blur();
                    self.children_list.focus();
                    self.file_meta.blur();
                } else if focus == 2 {
                    self.parents_list.blur();
                    self.children_list.blur();
                    self.file_meta.focus();
                }
            }
            _ => {
                let focus = self.focus.load(Ordering::Acquire);
                if focus == 0 {
                    self.parents_list.on_action(action)
                } else if focus == 1 {
                    self.children_list.on_action(action);
                } else if focus == 2 {
                    self.file_meta.on_action(action);
                }
            }
        }
    }
}
