use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    prelude::{Style, Widget},
    widgets::Block,
};

use super::root::Root;
use crate::ui::TopBar;

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

        self.frame += 1;
    }
}
