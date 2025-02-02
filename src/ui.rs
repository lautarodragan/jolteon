mod currently_playing;
mod keyboard_handler;
mod top_bar;

pub use currently_playing::*;
pub use keyboard_handler::*;
pub use top_bar::TopBar;

pub trait Focusable {
    fn set_is_focused(&self, _: bool) {}
    fn is_focused(&self) -> bool {
        false
    }
}
