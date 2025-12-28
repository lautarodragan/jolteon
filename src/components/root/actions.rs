use super::Root;
use crate::actions::{Action, OnActionMut, ScreenAction};

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
