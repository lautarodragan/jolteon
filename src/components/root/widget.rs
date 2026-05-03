use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Margin, Offset, Rect},
    prelude::{Style, Widget},
    style::Modifier,
    text::{Line, Span},
    widgets::Block,
};

use super::root::Root;
use crate::{components::Query, ui::TopBar};

impl Widget for &mut Root<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Block::default()
            .style(Style::default().bg(self.theme.background))
            .render(area, buf);

        let [area_top, _, area_center, area_player] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .areas(area);

        let screen_titles: Vec<&str> = self.screens.iter().map(|screen| screen.0.as_str()).collect();

        let top_bar = TopBar::new(
            self.settings,
            self.theme,
            &screen_titles,
            self.focused_screen,
            self.frame,
        );
        top_bar.render(area_top, buf);

        let Some((_, component)) = self.screens.get(self.focused_screen) else {
            log::error!("focused_screen is {}, which is out of bounds.", self.focused_screen);
            return;
        };

        component.borrow().render_ref(area_center, buf);

        // Line::from("  Error: file not found.")
        //     .style(Style::new().fg(self.theme.foreground))
        //     .render(area_noti, buf);

        if let Some(query) = self.query.borrow().as_ref() {
            let area = area_player.inner(Margin::new(1, 1));
            let line = match query {
                Query::AddSongs { songs, target } => {
                    let s1 = Span::from(format!("Add {} song(s) to", songs.len()));
                    let s2 = Span::from(target.to_string()).style(Style::default().bg(self.theme.background_selected));
                    let s3 = Span::from("Enter to confirm, Left/Right Arrows to change selection, Esc to cancel")
                        .style(Style::default().add_modifier(Modifier::DIM));
                    Line::from(vec![s1, Span::from(" "), s2, Span::from("?"), Span::from(" "), s3])
                }
            };
            line.render(area, buf);
            let query_error = self.query_error.borrow();
            if let Some(error) = query_error.as_ref() {
                let area = area.offset(Offset::new(0, 1));
                Line::from(error.as_ref())
                    .style(Style::default().fg(self.theme.search))
                    .render(area, buf);
            }
        } else {
            let Some(player) = self.player.upgrade() else {
                return;
            };

            let is_paused = player.is_paused()
                && (!self.settings.paused_animation || {
                    const ANIM_LEN: u64 = 6 * 16;
                    let step = self.frame % ANIM_LEN;
                    step % 12 < 6 || step >= ANIM_LEN / 2 // toggle visible/hidden every 6 frames, for half the length of the animation; then stay visible until the end.
                });

            let repeat_mode = player.repeat_mode();

            crate::ui::CurrentlyPlaying::new(
                self.theme,
                player.playing_song(),
                player.playing_position(),
                self.queue_screen.borrow().duration(),
                self.queue_screen.borrow().len(),
                is_paused,
                repeat_mode,
                player.volume(),
                self.frame,
            )
            .render(area_player, buf);
        }

        self.frame += 1;
    }
}
