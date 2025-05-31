use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};
use serde_default::DefaultFromSerde;

use crate::toml::read_toml_file_or_default;

#[serde_inline_default::serde_inline_default]
#[derive(Serialize, Deserialize, Debug, Copy, Clone, DefaultFromSerde)]
pub struct Settings {
    #[serde_inline_default(false)]
    pub debug_frame_counter: bool,

    #[serde_inline_default(true)]
    pub clock_display: bool,

    #[serde_inline_default(true)]
    pub paused_animation: bool,
}

impl Settings {
    pub fn from_file() -> Self {
        read_toml_file_or_default("settings")
    }
}

impl Display for Settings {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = toml::to_string_pretty(self);
        write!(f, "{}", s.unwrap())
    }
}
