use std::cell::RefCell;

use crate::{
    actions::{Action, NavigationAction, OnAction},
    structs::Direction,
    ui::*,
};

pub struct FocusGroup<'a> {
    pub(super) children: Vec<Component<'a>>,
    pub(super) focused: RefCell<Component<'a>>,
}

impl<'a> FocusGroup<'a> {
    pub fn new(children: Vec<Component<'a>>) -> Self {
        assert!(!children.is_empty(), "FocusGroup children cannot be empty");
        let focused = children[0].clone();

        for (e, i) in children.iter().enumerate() {
            i.set_is_focused(e == 0);
        }

        Self {
            children,
            focused: RefCell::new(focused),
        }
    }

    fn focus(&self, direction: Direction) {
        if self.children.len() < 2 {
            return;
        }

        let mut current_focus = self.focused.borrow_mut();

        let mut focus = 0;
        for i in 0..self.children.len() {
            if self.children[i] == *current_focus {
                focus = i;
            }
        }

        if direction == Direction::Forwards {
            if focus >= self.children.len() - 1 {
                focus = 0
            } else {
                focus += 1;
            }
        } else if direction == Direction::Backwards {
            if focus == 0 {
                focus = self.children.len() - 1
            } else {
                focus -= 1;
            }
        }

        for i in 0..self.children.len() {
            self.children[i].set_is_focused(i == focus);
            if i == focus {
                *current_focus = self.children[i].clone()
            }
        }
    }
}

impl OnAction for FocusGroup<'_> {
    fn on_action(&self, action: Vec<Action>) {
        match action[0] {
            Action::Navigation(NavigationAction::FocusNext) => {
                self.focus(Direction::Forwards);
            }
            Action::Navigation(NavigationAction::FocusPrevious) => {
                self.focus(Direction::Backwards);
            }
            _ => {
                let focused = self.focused.borrow();

                match &*focused {
                    Component::Ref(e) => e.on_action(action),
                    Component::Mut(e) => e.borrow_mut().on_action(action),
                }
            }
        }
    }
}
