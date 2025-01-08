use std::{collections::HashMap, fs::read_to_string, hash::Hash, sync::LazyLock};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::{Deserialize, Serialize};

use crate::toml::{get_config_file_path, TomlFileError};

static DEFAULT_ACTIONS_STR: &str = include_str!("../../assets/actions.kv");
static DEFAULT_ACTIONS: LazyLock<HashMap<KeyBinding, Vec<Action>>> =
    LazyLock::new(|| Actions::from_str(DEFAULT_ACTIONS_STR).actions);

#[derive(Eq, PartialEq, Copy, Clone, Debug, Serialize, Deserialize, Hash)]
pub struct KeyBinding {
    code: KeyCode,
    modifiers: KeyModifiers,
}

impl KeyBinding {
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

impl From<KeyEvent> for KeyBinding {
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
    FocusNext,
    FocusPrevious,
    Screen(ScreenAction),
    Navigation(NavigationAction),
    Player(PlayerAction),
    ListAction(ListAction),
    Playlists(PlaylistsAction),
    FileBrowser(FileBrowserAction),
}

#[derive(Eq, PartialEq, Copy, Clone, Debug, Hash)]
pub enum NavigationAction {
    FocusNext,
    FocusPrevious,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    NextSpecial,
    PreviousSpecial,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug, Hash)]
pub enum ListAction {
    Primary,
    Secondary,
    RenameCancel,
    Insert,
    Delete,
    SwapUp,
    SwapDown,
    RenameStart,
    RenameChar(char),
    RenameDeleteCharBack,
    RenameClear,
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
    RepeatOff,
    RepeatOne,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug, Hash)]
pub enum FileBrowserAction {
    AddToQueue,
    AddToLibrary,
    AddToPlaylist,
    ToggleMode,
    OpenTerminal,
    NavigateUp,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug, Hash)]
pub enum PlaylistsAction {
    ShowHideGraveyard,
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
        } else if parent == "Screen" {
            ScreenAction::try_from(child).map(Action::Screen)
        } else if parent == "Navigation" {
            NavigationAction::try_from(child).map(Action::Navigation)
        } else if parent == "List" {
            ListAction::try_from(child).map(Action::ListAction)
        } else if parent == "Playlists" {
            PlaylistsAction::try_from(child).map(Action::Playlists)
        } else if parent == "FileBrowser" {
            FileBrowserAction::try_from(child).map(Action::FileBrowser)
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
            "RepeatOff" => Ok(Self::RepeatOff),
            "RepeatOne" => Ok(Self::RepeatOne),
            _ => Err(()),
        }
    }
}

impl TryFrom<&str> for NavigationAction {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, ()> {
        match value {
            "FocusNext" => Ok(Self::FocusNext),
            "FocusPrevious" => Ok(Self::FocusPrevious),
            "Up" => Ok(Self::Up),
            "Down" => Ok(Self::Down),
            "Left" => Ok(Self::Left),
            "Right" => Ok(Self::Right),
            "Home" => Ok(Self::Home),
            "End" => Ok(Self::End),
            "PageUp" => Ok(Self::PageUp),
            "PageDown" => Ok(Self::PageDown),
            "NextSpecial" => Ok(Self::NextSpecial),
            "PreviousSpecial" => Ok(Self::PreviousSpecial),
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
            "Insert" => Ok(Self::Insert),
            "Delete" => Ok(Self::Delete),
            "SwapUp" => Ok(Self::SwapUp),
            "SwapDown" => Ok(Self::SwapDown),
            "RenameStart" => Ok(Self::RenameStart),
            "RenameCancel" => Ok(Self::RenameCancel),
            "RenameDeleteCharBack" => Ok(Self::RenameDeleteCharBack),
            "RenameClear" => Ok(Self::RenameClear),
            _ => Err(()),
        }
    }
}

impl TryFrom<&str> for FileBrowserAction {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, ()> {
        match value {
            "NavigateUp" => Ok(Self::NavigateUp),
            "AddToPlaylist" => Ok(Self::AddToPlaylist),
            "AddToLibrary" => Ok(Self::AddToLibrary),
            "AddToQueue" => Ok(Self::AddToQueue),
            "ToggleMode" => Ok(Self::ToggleMode),
            "OpenTerminal" => Ok(Self::OpenTerminal),
            _ => Err(()),
        }
    }
}

impl TryFrom<&str> for PlaylistsAction {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, ()> {
        match value {
            "ShowHideGraveyard" => Ok(Self::ShowHideGraveyard),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Default)]
pub struct Actions {
    actions: HashMap<KeyBinding, Vec<Action>>,
}

impl Actions {
    fn from_str(s: &str) -> Self {
        // log::trace!("from str {s}");

        let mut actions: HashMap<KeyBinding, Vec<Action>> = HashMap::new();

        s.lines()
            .filter(|line| line.len() >= 3 && !line.trim().starts_with('#'))
            .map(|line| line.split('=').collect::<Vec<&str>>())
            .filter_map(str_to_action_keys)
            .for_each(|(action, bindings)| {
                bindings.split(' ').filter_map(str_to_binding).for_each(|binding| {
                    actions
                        .entry(binding)
                        .and_modify(|actions| actions.push(action))
                        .or_insert(vec![action]);
                });
            });

        // log::trace!("actions '{:#?}'", actions);

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
        let sc = KeyBinding::from(key);
        // log::trace!("key {key:?}");
        self.actions
            .get(&sc)
            .or(DEFAULT_ACTIONS.get(&sc))
            .map(|actions| actions[0])
    }

    pub fn key_by_action(&self, action: Action) -> Option<KeyBinding> {
        self.actions.iter().chain(DEFAULT_ACTIONS.iter()).find_map(|(k, v)| {
            if v.iter().any(|a| *a == action) {
                Some(*k)
            } else {
                None
            }
        })
    }

    pub fn contains(&self, action: Action) -> bool {
        self.actions
            .values()
            .chain(DEFAULT_ACTIONS.values())
            .flatten()
            .any(|a| *a == action)
    }

    pub fn list_primary(&self) -> KeyBinding {
        self.key_by_action(Action::ListAction(ListAction::Primary)).unwrap()
    }

    pub fn list_secondary(&self) -> KeyBinding {
        self.key_by_action(Action::ListAction(ListAction::Secondary)).unwrap()
    }
}

pub trait OnAction<T = Action> {
    fn on_action(&self, action: T);
}

pub trait OnActionMut<T = Action> {
    fn on_action(&mut self, action: T);
}

fn str_to_action_keys(split: Vec<&str>) -> Option<(Action, &str)> {
    if let [value, keys] = split[..] && let Ok(action) = Action::try_from(value) {
        Some((action, keys))
    } else {
        None
    }
}

fn str_to_binding(binding: &str) -> Option<KeyBinding> {
    str_to_modifiers(binding)
        .and_then(|(modifiers, key)| str_to_key(key, modifiers).map(|code| KeyBinding::new(code, modifiers)))
}

fn str_to_modifiers(key: &str) -> Option<(KeyModifiers, &str)> {
    let mut modifiers = KeyModifiers::NONE;
    let mut key = key;

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

    Some((modifiers, key))
}

fn str_to_key(key: &str, modifiers: KeyModifiers) -> Option<KeyCode> {
    let code: KeyCode;

    if key.len() == 1 {
        let char = key.chars().next()?;

        if char.is_ascii_alphabetic() {
            if modifiers.contains(KeyModifiers::SHIFT) {
                code = KeyCode::Char(char);
            } else {
                code = KeyCode::Char(char.to_ascii_lowercase());
            }
        } else {
            code = KeyCode::Char(char);
        }
    } else if (key.len() == 2 || key.len() == 3)
        && key.starts_with('F')
        && let Ok(num) = key[1..].parse::<u8>()
    {
        code = KeyCode::F(num);
    } else if key == "Enter" {
        code = KeyCode::Enter;
    } else if key == "Esc" {
        code = KeyCode::Esc;
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
    } else if key == "Home" {
        code = KeyCode::Home;
    } else if key == "End" {
        code = KeyCode::End;
    } else if key == "PageUp" {
        code = KeyCode::PageUp;
    } else if key == "PageDown" {
        code = KeyCode::PageDown;
    } else if key == "Backspace" {
        code = KeyCode::Backspace;
    } else if key == "Tab" {
        code = KeyCode::Tab;
    } else if key == "BackTab" {
        code = KeyCode::BackTab;
    } else if key == "Insert" {
        code = KeyCode::Insert;
    } else if key == "Delete" {
        code = KeyCode::Delete;
    } else {
        return None;
    }
    Some(code)
}
