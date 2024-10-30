use ratatui::style::Color;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct Config {
    pub theme: Theme,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct Theme {
    pub top_bar_background: Color,
    pub top_bar_foreground_selected: Color,
    pub foreground: Color,
    pub foreground_selected: Color,
    pub foreground_secondary: Color,
    pub background: Color,
    pub background_selected: Color,
    pub background_selected_blur: Color,
    pub search: Color,
}
