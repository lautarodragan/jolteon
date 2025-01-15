use ratatui::widgets::WidgetRef;

use crate::{
    structs::{OnAction, OnActionMut},
    ui::Focusable,
};

pub trait ComponentRef<'a>: WidgetRef + OnAction + Focusable {}
pub trait ComponentMut<'a>: WidgetRef + OnActionMut {}

impl<T: OnAction + WidgetRef + Focusable> ComponentRef<'_> for T {}
impl<T: OnActionMut + WidgetRef> ComponentMut<'_> for T {}
