use std::{cell::RefCell, rc::Rc};

use ratatui::widgets::WidgetRef;

use crate::{
    actions::{OnAction, OnActionMut},
    ui::Focusable,
};

pub trait ComponentRef<'a>: WidgetRef + OnAction + Focusable {}
pub trait ComponentMut<'a>: WidgetRef + OnActionMut + Focusable {}

impl<T: OnAction + WidgetRef + Focusable> ComponentRef<'_> for T {}
impl<T: OnActionMut + WidgetRef + Focusable> ComponentMut<'_> for T {}

#[derive(Clone)]
pub enum Component<'a> {
    Ref(Rc<dyn 'a + ComponentRef<'a>>),
    Mut(Rc<RefCell<dyn 'a + ComponentMut<'a>>>),
}

impl Focusable for Component<'_> {
    fn set_is_focused(&self, v: bool) {
        match self {
            Component::Ref(e) => e.set_is_focused(v),
            Component::Mut(e) => e.borrow_mut().set_is_focused(v),
        }
    }

    fn is_focused(&self) -> bool {
        match self {
            Component::Ref(e) => e.is_focused(),
            Component::Mut(e) => e.borrow().is_focused(),
        }
    }
}

impl PartialEq for Component<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (&self, other) {
            (Component::Ref(a), Component::Ref(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}
