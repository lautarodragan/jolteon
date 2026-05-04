use super::Root;
use crate::actions::{Action, OnActionMut, ScreenAction};

impl OnActionMut for Root<'_> {
    fn on_action(&mut self, actions: Vec<Action>) {
        if self.command_line.borrow().query().is_some() {
            self.command_line.borrow_mut().on_action(actions);
        } else {
            match actions[0] {
                Action::Screen(action) if !self.is_focus_trapped.get() => match action {
                    ScreenAction::Next => {
                        if self.focused_screen < 5 {
                            self.focused_screen += 1;
                        } else {
                            self.focused_screen = 0;
                        }
                    }
                    ScreenAction::Previous => {
                        if self.focused_screen > 0 {
                            self.focused_screen -= 1;
                        } else {
                            self.focused_screen = 5;
                        }
                    }
                    ScreenAction::Library => self.focused_screen = 0,
                    ScreenAction::Soundtracks => self.focused_screen = 1,
                    ScreenAction::Playlists => self.focused_screen = 2,
                    ScreenAction::Queue => self.focused_screen = 3,
                    ScreenAction::FileBrowser => self.focused_screen = 4,
                    ScreenAction::Help => self.focused_screen = 5,
                },
                _ => {
                    let mut c = self.screens[self.focused_screen].1.borrow_mut();
                    c.on_action(actions);
                }
            }
        }
    }
}
