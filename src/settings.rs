use serde::{Deserialize, Serialize};
use crate::toml::read_toml_file_or_default;
use serde_default::DefaultFromSerde;

#[serde_inline_default::serde_inline_default]
#[derive(Serialize, Deserialize, Debug, Copy, Clone, DefaultFromSerde)]
pub struct Settings {
    #[serde_inline_default(false)]
    pub debug_frame_counter: bool,

    #[serde_inline_default(true)]
    pub clock_display: bool,
}

impl Settings {
    pub fn from_file() -> Self {
        read_toml_file_or_default("settings")
    }
}
