use std::{
    collections::HashMap,
    hash::Hash,
    fs::read_to_string,
    sync::LazyLock,
};

use crossterm::{
    event::{KeyCode, KeyEvent, KeyModifiers}
};
use serde::{Deserialize, Serialize};

use crate::toml::{TomlFileError, get_config_file_path};

static DEFAULT_ACTIONS_STR: &str = include_str!("../../assets/actions.kv");
static DEFAULT_ACTIONS: LazyLock<HashMap<Shortcut, Action>> = LazyLock::new(|| {
    Actions::from_str(DEFAULT_ACTIONS_STR).actions
});

#[derive(Eq, PartialEq, Copy, Clone, Debug, Serialize, Deserialize, Hash)]
pub struct Shortcut {
    code: KeyCode,
    modifiers: KeyModifiers,
}

impl Shortcut {
    fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { code, modifiers }
    }
}

impl From<KeyEvent> for Shortcut {
    fn from(key: KeyEvent) -> Self {
        Self { code: key.code, modifiers: key.modifiers }
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug, Hash)]
#[allow(dead_code)]
pub enum Action {
    Error,
    Quit,
    QueueNext,
    ScreenAction(ScreenAction),
    PlayerAction(PlayerAction),
}

#[derive(Eq, PartialEq, Copy, Clone, Debug, Hash)]
pub enum ScreenAction {
    Library,
    Playlists,
    Queue,
    FileBrowser,
    Help,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug, Hash)]
pub enum PlayerAction {
    Stop,
    PlayPause,
    VolumeUp,
    VolumeDown,
    SeekForwards,
    SeekBackwards,
}

impl TryFrom<&str> for Action {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, ()> {
        if value == "Quit" {
            return Ok(Self::Quit);
        }

        let parts: Vec<&str> = value.split('.').collect();
        let [parent, child]: [&str] = parts[..] else {
            return Err(());
        };

        if parent == "Player" {
            PlayerAction::try_from(child).map(|v| Action::PlayerAction(v))
        } else if parent == "Screen" {
            ScreenAction::try_from(child).map(|v| Action::ScreenAction(v))
        } else {
            Err(())
        }
    }
}

impl TryFrom<&str> for ScreenAction {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, ()> {
        match value {
            "Library" => Ok(Self::Library),
            "Playlists" => Ok(Self::Playlists),
            "Queue" => Ok(Self::Queue),
            "FileBrowser" => Ok(Self::FileBrowser),
            "Help" => Ok(Self::Help),
            _ => Err(()),
        }
    }
}

impl TryFrom<&str> for PlayerAction {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, ()> {
        match value {
            "PlayPause" => Ok(Self::PlayPause),
            "Stop" => Ok(Self::Stop),
            "VolumeUp" => Ok(Self::VolumeUp),
            "VolumeDown" => Ok(Self::VolumeDown),
            "SeekForwards" => Ok(Self::SeekForwards),
            "SeekBackwards" => Ok(Self::SeekBackwards),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub struct Actions {
    actions: HashMap<Shortcut, Action>,
}

impl Actions {
    pub fn from_str(s: &str) -> Self {
        log::debug!("from str {s}");

        let mut actions: HashMap<Shortcut, Action> = HashMap::new();

        for l in s.lines() {
            if s.len() < 3 {
                continue;
            }
            let splits: Vec<&str> = l.split('=').collect();
            let [mut key, value] = splits[..] else {
                log::error!("invalid line {l}");
                continue;
            };

            let mut modifiers = KeyModifiers::NONE;

            loop {
                if key.starts_with("Ctrl") {
                    modifiers.toggle(KeyModifiers::CONTROL);
                    key = &key[4..];
                } else if key.starts_with("Alt") {
                    modifiers.toggle(KeyModifiers::ALT);
                    key = &key[3..];
                } else if key.starts_with("Shift") {
                    modifiers.toggle(KeyModifiers::SHIFT);
                    key = &key[5..];
                } else {
                    break;
                }
            }

            let code: KeyCode;

            if key.len() == 1 {
                let mut chars = key.chars();

                let Some(char) = chars.nth(0) else {
                    continue;
                };

                if char.is_ascii_alphabetic() {
                    if modifiers.contains(KeyModifiers::SHIFT) {
                        code = KeyCode::Char(char);
                    } else {
                        code = KeyCode::Char(char.to_ascii_lowercase());
                    }
                } else {
                    code = KeyCode::Char(char);
                }
            } else if key == "Space" {
                code = KeyCode::Char(' ');
            } else if key == "Right" {
                code = KeyCode::Right;
            } else if key == "Left" {
                code = KeyCode::Left;
            } else if key == "Up" {
                code = KeyCode::Up;
            } else if key == "Down" {
                code = KeyCode::Down;
            } else {
                log::error!("invalid line {l}");
                continue;
            }

            let shortcut = Shortcut::new(code, modifiers);
            let Ok(action) = Action::try_from(value) else {
                log::error!("invalid line {l}");
                continue;
            };

            actions.insert(shortcut, action);
        }

        log::trace!("actions '{:#?}'", actions);

        Self {
            actions,
        }
    }

    #[allow(dead_code)]
    pub fn to_file(&self) {

    }

    pub fn from_file() -> Result<Self, TomlFileError> {
        let path = get_config_file_path("shortcuts")?;
        let string = read_to_string(path)?;

        Ok(Self::from_str(string.as_str()))
    }

    pub fn by_key(&self, key: KeyEvent) -> Option<Action> {
        let sc = Shortcut::from(key);
        self.actions.get(&sc).or(DEFAULT_ACTIONS.get(&sc)).cloned()
    }
}

pub trait OnAction {
    fn on_action(&self, action: Action);
}


pub trait OnActionMut {
    fn on_action(&mut self, action: Action);
}
