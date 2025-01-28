use crossterm::event::KeyCode;

use crate::actions::NavigationAction;

#[derive(Eq, PartialEq)]
pub enum Direction {
    Backwards,
    Forwards,
}

impl From<KeyCode> for Direction {
    fn from(key_code: KeyCode) -> Self {
        if key_code == KeyCode::Up || key_code == KeyCode::Home || key_code == KeyCode::PageUp {
            Self::Backwards
        } else {
            Self::Forwards
        }
    }
}

impl From<NavigationAction> for Direction {
    fn from(action: NavigationAction) -> Self {
        if action == NavigationAction::Up
            || action == NavigationAction::Home
            || action == NavigationAction::PageUp
            || action == NavigationAction::PreviousSpecial
        {
            Self::Backwards
        } else {
            Self::Forwards
        }
    }
}
