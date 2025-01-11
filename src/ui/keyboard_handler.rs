use std::{cell::RefCell, rc::Rc};

use ratatui::widgets::WidgetRef;

use crate::{
    structs::{Action, OnAction, OnActionMut},
    ui::Focusable,
};

pub trait ComponentRef<'a>: WidgetRef + OnAction {}
pub trait ComponentRefFocusable<'a>: ComponentRef<'a> + Focusable {}
pub trait ComponentMut<'a>: WidgetRef + OnActionMut {}

impl<T: OnAction + WidgetRef> ComponentRef<'_> for T {}
impl<T: OnAction + WidgetRef + Focusable> ComponentRefFocusable<'_> for T {}
impl<T: OnActionMut + WidgetRef> ComponentMut<'_> for T {}

pub enum Component<'a> {
    Ref(Rc<dyn 'a + ComponentRef<'a>>),
    Mut(Rc<RefCell<dyn 'a + ComponentMut<'a>>>),
}

impl OnAction for Component<'_> {
    fn on_action(&self, action: Action) {
        match self {
            Component::Ref(ref target) => {
                target.on_action(action);
            }
            Component::Mut(ref target) => {
                let mut target = target.borrow_mut();
                target.on_action(action);
            }
        }
    }
}
