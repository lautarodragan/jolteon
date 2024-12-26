use std::{collections::HashMap, fs::read_to_string, hash::Hash, sync::LazyLock};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::{Deserialize, Serialize};

use crate::toml::{get_config_file_path, TomlFileError};

static DEFAULT_ACTIONS_STR: &str = include_str!("../../assets/actions.kv");
static DEFAULT_ACTIONS: LazyLock<HashMap<Shortcut, Action>> =
    LazyLock::new(|| Actions::from_str(DEFAULT_ACTIONS_STR).actions);

#[derive(Eq, PartialEq, Copy, Clone, Debug, Serialize, Deserialize, Hash)]
pub struct Shortcut {
    code: KeyCode,
    modifiers: KeyModifiers,
}

impl Shortcut {
    pub fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { code, modifiers }
    }

    pub fn code(&self) -> KeyCode {
        self.code
    }

    pub fn modifiers(&self) -> KeyModifiers {
        self.modifiers
    }
}

impl From<KeyEvent> for Shortcut {
    fn from(key: KeyEvent) -> Self {
        Self {
            code: key.code,
            modifiers: key.modifiers,
        }
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug, Hash)]
#[allow(dead_code)]
pub enum Action {
    Error,
    Quit,
    QueueNext,
    Screen(ScreenAction),
    Player(PlayerAction),
    MainPlayer(MainPlayerAction),
    ListAction(ListAction),
    FileBrowser(FileBrowserAction),
}

#[derive(Eq, PartialEq, Copy, Clone, Debug, Hash)]
pub enum ListAction {
    Primary,
    Secondary,
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

#[derive(Eq, PartialEq, Copy, Clone, Debug, Hash)]
pub enum MainPlayerAction {
    RepeatOff,
    RepeatOne,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug, Hash)]
pub enum FileBrowserAction {
    AddToQueue,
    AddToLibrary,
    AddToPlaylist,
    ToggleMode,
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
            PlayerAction::try_from(child).map(Action::Player)
        } else if parent == "MainPlayer" {
            MainPlayerAction::try_from(child).map(Action::MainPlayer)
        } else if parent == "Screen" {
            ScreenAction::try_from(child).map(Action::Screen)
        } else if parent == "List" {
            ListAction::try_from(child).map(Action::ListAction)
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

impl TryFrom<&str> for MainPlayerAction {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, ()> {
        match value {
            "RepeatOff" => Ok(Self::RepeatOff),
            "RepeatOne" => Ok(Self::RepeatOne),
            _ => Err(()),
        }
    }
}

impl TryFrom<&str> for ListAction {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, ()> {
        match value {
            "Primary" => Ok(Self::Primary),
            "Secondary" => Ok(Self::Secondary),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Default)]
pub struct Actions {
    actions: HashMap<Shortcut, Action>,
}

impl Actions {
    fn from_str(s: &str) -> Self {
        log::trace!("from str {s}");

        let mut actions: HashMap<Shortcut, Action> = HashMap::new();

        for l in s.lines() {
            if l.len() < 3 {
                continue;
            }
            if l.trim().starts_with('#') {
                continue;
            }
            let splits: Vec<&str> = l.split('=').collect();
            let [mut key, value] = splits[..] else {
                log::debug!("ignoring invalid line, too few/many splits: {l}");
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
            } else if key == "Enter" {
                code = KeyCode::Enter;
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
            } else if key == "End" {
                code = KeyCode::End;
            } else {
                log::debug!("ignoring invalid line with key={key}");
                continue;
            }

            let shortcut = Shortcut::new(code, modifiers);
            let Ok(action) = Action::try_from(value) else {
                log::debug!("ignoring invalid line, unknown shortcut {value} for key {shortcut}");
                continue;
            };

            actions.insert(shortcut, action);
        }

        log::trace!("actions '{:#?}'", actions);

        Self { actions }
    }

    pub fn from_file() -> Result<Self, TomlFileError> {
        let path = get_config_file_path("shortcuts")?;
        let string = read_to_string(path)?;

        Ok(Self::from_str(string.as_str()))
    }

    pub fn from_file_or_default() -> Self {
        Self::from_file().unwrap_or_default()
    }

    #[allow(dead_code)]
    pub fn to_file(&self) {}

    pub fn action_by_key(&self, key: KeyEvent) -> Option<Action> {
        let sc = Shortcut::from(key);
        self.actions.get(&sc).or(DEFAULT_ACTIONS.get(&sc)).cloned()
    }

    pub fn key_by_action(&self, action: Action) -> Option<Shortcut> {
        self.actions
            .iter()
            .chain(DEFAULT_ACTIONS.iter())
            .find_map(|(k, v)| if *v == action { Some(*k) } else { None })
    }

    pub fn contains(&self, action: Action) -> bool {
        self.actions
            .values()
            .chain(DEFAULT_ACTIONS.values())
            .any(|a| *a == action)
    }

    pub fn list_primary(&self) -> Shortcut {
        self.key_by_action(Action::ListAction(ListAction::Primary)).unwrap()
    }

    pub fn list_secondary(&self) -> Shortcut {
        self.key_by_action(Action::ListAction(ListAction::Secondary)).unwrap()
    }
}

pub trait OnAction<T = Action> {
    fn on_action(&self, action: T);
}

pub trait OnActionMut {
    fn on_action(&mut self, action: Action);
}
