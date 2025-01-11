use std::{cell::RefCell, rc::Rc};

use crate::{
    components::list::Direction,
    structs::{Action, NavigationAction, OnAction},
    ui::ComponentRefFocusable,
};

pub struct ChildrenComponents<'a> {
    pub(super) children: Vec<Rc<dyn 'a + ComponentRefFocusable<'a>>>,
    pub(super) focused: RefCell<Rc<dyn 'a + ComponentRefFocusable<'a>>>,
}

impl<'a> ChildrenComponents<'a> {
    pub fn new(children: Vec<Rc<dyn 'a + ComponentRefFocusable<'a>>>) -> Self {
        let focused = children[0].clone();
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
            if Rc::ptr_eq(&self.children[i], &*current_focus) {
                focus = i;
            }
        }

        if direction == Direction::Forwards {
            if focus > 1 {
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

impl OnAction for ChildrenComponents<'_> {
    fn on_action(&self, action: Action) {
        match action {
            Action::Navigation(NavigationAction::FocusNext) => {
                self.focus(Direction::Forwards);
            }
            Action::Navigation(NavigationAction::FocusPrevious) => {
                self.focus(Direction::Backwards);
            }
            _ => {
                self.focused.borrow().on_action(action);
            }
        }
    }
}
