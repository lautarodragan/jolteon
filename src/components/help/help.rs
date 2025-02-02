use std::collections::HashMap;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Style,
    text::Line,
    widgets::{WidgetRef, Wrap},
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
    #[allow(unused)]
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
        let [area_top, area_main] = Layout::vertical([Constraint::Max(10), Constraint::Min(5)])
            .horizontal_margin(2)
            .areas(area);

        // TODO:
        //   Ratatui has a nice WordWrapper that takes care of splitting lines,
        //   but it doesn't export it, and the line count after wrapping isn't
        //   exported by the Paragraph, so we have no way of measuring the rendered
        //   area height of the paragraph.
        //   Options:
        //     - Reimplement most of Paragraph to suit Jolteon's need
        //     - Contribute to Ratatui, either to make the WordWrapper public,
        //       or to have Paragraph somehow return the rendered line count (stateful widget?)
        //     - Set a fixed area for the paragraph, let it clip, and use Paragraph's scroll feature
        //   The latter would require some visual feedback — probably borders... which are ugly :(

        ratatui::widgets::Paragraph::new(vec![
            Line::raw("Hi! Welcome to Jolteon."),
            Line::raw("This project is a work in progress. It's generally usable, but not thoroughly documented, yet."),
            Line::raw(
                "This screen will eventually have some help, documentation and display the active settings and theme.",
            ),
            Line::raw("For now, here are the key bindings:"),
        ])
        .style(Style::new().fg(self.theme.foreground_secondary))
        .wrap(Wrap { trim: true })
        .render_ref(area_top, buf);

        self.actions.render_ref(area_main, buf);
    }
}

impl Drop for Help<'_> {
    fn drop(&mut self) {
        log::trace!("HelpTab.drop()");
    }
}

impl Focusable for Help<'_> {}
