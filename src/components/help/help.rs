use std::collections::HashMap;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Style,
    text::Line,
    widgets::WidgetRef,
};

use crate::{
    actions::{Action, Actions, KeyBinding, OnAction, OnActionMut},
    components::List,
    config::Theme,
    settings::Settings,
    ui::Focusable,
};

pub struct Help<'a> {
    actions: List<'a, String>,
    theme: Theme,
    settings: Settings,
}

impl Help<'_> {
    pub fn new(actions: Actions, settings: Settings, theme: Theme) -> Self {
        let mut actions_by_action: HashMap<Action, Vec<KeyBinding>> = HashMap::new();

        for (k, v) in actions.actions() {
            for action in v {
                let entry = actions_by_action.get_mut(&action);
                if let Some(entry) = entry {
                    entry.push(k);
                } else {
                    actions_by_action.insert(action, vec![k]);
                }
            }
        }

        let mut actions_by_action: Vec<(Action, Vec<KeyBinding>)> = actions_by_action.into_iter().collect();
        actions_by_action.sort_by_key(|e| e.0);

        let actions: Vec<String> = actions_by_action
            .into_iter()
            .map(|(action, key_bindings)| {
                format!(
                    "{:32} {}",
                    format!("{:?}", action),
                    key_bindings
                        .iter()
                        .map(|kb| kb.to_string())
                        .collect::<Vec<String>>()
                        .join(" ")
                )
            })
            .collect();

        let actions = List::new(theme, actions);
        actions.set_is_focused(true);

        Self {
            actions,
            theme,
            settings,
        }
    }
}

impl OnActionMut for Help<'_> {
    fn on_action(&mut self, action: Vec<Action>) {
        self.actions.on_action(action);
    }
}

impl WidgetRef for Help<'_> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let [area_top, area_main] = Layout::vertical([Constraint::Max(5), Constraint::Min(5)])
            .horizontal_margin(2)
            .areas(area);

        ratatui::widgets::Paragraph::new(vec![
            Line::raw("Hi! Welcome to Jolteon."),
            Line::raw("This project is a work in progress. It's generally usable, but not thoroughly documented, yet."),
            Line::raw(
                "This screen will eventually have some help, documentation and display the active settings and theme.",
            ),
            Line::raw("For now, here are the key bindings:"),
        ])
        .style(Style::new().fg(self.theme.foreground_secondary))
        .wrap(ratatui::widgets::Wrap { trim: true })
        .render_ref(area_top, buf);

        self.actions.render_ref(area_main, buf);
    }
}

impl Drop for Help<'_> {
    fn drop(&mut self) {
        log::trace!("HelpTab.drop()");
    }
}

impl Focusable for Help<'_> {
    fn set_is_focused(&self, v: bool) {
        todo!()
    }

    fn is_focused(&self) -> bool {
        todo!()
    }
}
