use std::{
    cmp::Ordering,
    fs,
    fs::DirEntry,
    path::{Path, PathBuf},
};

use crate::{
    cue::CueSheet,
    structs::{Jolt, Song},
};

const VALID_EXTENSIONS: [&str; 7] = ["mp3", "mp4", "m4a", "wav", "flac", "ogg", "aac"];

#[derive(Debug, Clone, Eq)]
pub enum FileBrowserSelection {
    Song(Song),
    CueSheet(CueSheet),
    Directory(PathBuf),
    Jolt(Jolt),
    Other(PathBuf),
}

impl FileBrowserSelection {
    pub fn from_path(path: &PathBuf) -> Option<Self> {
        if path.is_dir() {
            Some(FileBrowserSelection::Directory(path.clone()))
        } else if path.extension().is_some_and(|e| e == "cue") {
            CueSheet::from_file(path).ok().map(FileBrowserSelection::CueSheet)
        } else {
            Song::from_file(path).ok().map(FileBrowserSelection::Song)
        }
    }

    pub fn to_path(&self) -> PathBuf {
        match self {
            FileBrowserSelection::Song(s) => s.path.clone(),
            FileBrowserSelection::CueSheet(cs) => cs.cue_sheet_file_path(),
            FileBrowserSelection::Directory(p) => p.clone(),
            FileBrowserSelection::Jolt(j) => j.path.clone(),
            FileBrowserSelection::Other(p) => p.clone(),
        }
    }
}

impl PartialEq for FileBrowserSelection {
    fn eq(&self, other: &Self) -> bool {
        match self {
            FileBrowserSelection::Directory(path) => match other {
                FileBrowserSelection::Directory(other_path) => path == other_path,
                _ => false,
            },
            FileBrowserSelection::Other(path) => match other {
                FileBrowserSelection::Other(other_path) => path == other_path,
                _ => false,
            },
            FileBrowserSelection::CueSheet(cue_sheet) => match other {
                FileBrowserSelection::CueSheet(other_cue_sheet) => {
                    cue_sheet.cue_sheet_file_path() == other_cue_sheet.cue_sheet_file_path()
                }
                _ => false,
            },
            FileBrowserSelection::Song(song) => match other {
                FileBrowserSelection::Song(other_song) => song.path == other_song.path,
                _ => false,
            },
            FileBrowserSelection::Jolt(jolt) => match other {
                FileBrowserSelection::Jolt(j) => jolt.path == j.path,
                _ => false,
            },
        }
    }
}

impl PartialOrd for FileBrowserSelection {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FileBrowserSelection {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            FileBrowserSelection::Directory(path) => {
                // Directories come first
                match other {
                    FileBrowserSelection::Directory(other_path) => path.cmp(other_path),
                    _ => Ordering::Less,
                }
            }
            FileBrowserSelection::Jolt(jolt) => {
                // then jolt files
                match other {
                    FileBrowserSelection::Directory(_) => Ordering::Greater,
                    FileBrowserSelection::Jolt(other_jolt) => jolt.path.cmp(&other_jolt.path),
                    _ => Ordering::Less,
                }
            }
            FileBrowserSelection::CueSheet(cue_sheet) => {
                // then queue sheets
                match other {
                    FileBrowserSelection::Directory(_) => Ordering::Greater,
                    FileBrowserSelection::Jolt(_) => Ordering::Greater,
                    FileBrowserSelection::CueSheet(other_cue_sheet) => cue_sheet
                        .cue_sheet_file_path()
                        .cmp(&other_cue_sheet.cue_sheet_file_path()),
                    _ => Ordering::Less,
                }
            }
            FileBrowserSelection::Song(song) => {
                // then songs
                match other {
                    FileBrowserSelection::Song(other_song) => song.path.cmp(&other_song.path),
                    FileBrowserSelection::Other(_) => Ordering::Less,
                    _ => Ordering::Greater,
                }
            }
            FileBrowserSelection::Other(path) => {
                // last, files we can't work with, but may want to show
                match other {
                    FileBrowserSelection::Other(other_path) => path.cmp(other_path),
                    _ => Ordering::Greater,
                }
            }
        }
    }
}

fn dir_entry_to_file_browser_selection(entry: &DirEntry) -> Option<FileBrowserSelection> {
    if dir_entry_is_dir(entry) {
        Some(FileBrowserSelection::Directory(entry.path()))
    } else if dir_entry_is_song(entry) {
        match Song::from_file(&entry.path()).map(FileBrowserSelection::Song) {
            Ok(a) => Some(a),
            Err(err) => {
                log::warn!("dir_entry_to_file_browser_selection {:#?} {:#?}", &entry.path(), err);
                None
            }
        }
    } else if dir_entry_is_cue(entry) {
        Some(FileBrowserSelection::CueSheet(
            CueSheet::from_file(&entry.path()).unwrap(),
        ))
    } else if dir_entry_is_jolt_file(entry) {
        match Jolt::from_path(entry.path()) {
            Ok(jolt) => Some(FileBrowserSelection::Jolt(jolt)),
            Err(err) => {
                log::error!("Could not read .jolt file {:#?}", err);
                None
            }
        }
    } else {
        Some(FileBrowserSelection::Other(entry.path()))
    }
}

pub fn directory_to_songs_and_folders(path: &Path) -> Vec<FileBrowserSelection> {
    let Ok(entries) = path.read_dir() else {
        return vec![];
    };

    let mut items: Vec<FileBrowserSelection> = entries
        .filter_map(|e| e.ok())
        // .filter(|e| path_is_not_hidden(&e.path()))
        .filter_map(|e| dir_entry_to_file_browser_selection(&e))
        .collect();

    items.sort_unstable();
    items
}

pub fn dir_entry_is_file(dir_entry: &DirEntry) -> bool {
    // TODO: resolve symlinks
    dir_entry.file_type().is_ok_and(|ft| ft.is_file())
}

pub fn dir_entry_is_dir(dir_entry: &DirEntry) -> bool {
    let Ok(ft) = dir_entry.file_type() else {
        log::error!(
            "dir_entry_is_dir: .file_type() returned error for {:?}",
            dir_entry.path()
        );
        return false;
    };

    if ft.is_symlink() {
        let ln = fs::canonicalize(dir_entry.path());
        ln.is_ok_and(|ln| ln.is_dir())
    } else {
        ft.is_dir()
    }
}

#[allow(dead_code)]
pub fn path_is_not_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|e| e.to_str())
        .map(|e| e.to_string())
        .is_some_and(|d| !d.starts_with('.'))
}

pub fn dir_entry_has_song_extension(dir_entry: &DirEntry) -> bool {
    dir_entry
        .path()
        .extension()
        .is_some_and(|e| VALID_EXTENSIONS.contains(&e.to_str().unwrap().to_lowercase().as_str()))
}

pub fn dir_entry_is_song(dir_entry: &DirEntry) -> bool {
    dir_entry_is_file(dir_entry) && dir_entry_has_song_extension(dir_entry)
}

pub fn dir_entry_has_cue_extension(dir_entry: &DirEntry) -> bool {
    dir_entry.path().extension().is_some_and(|e| e == "cue")
}

pub fn dir_entry_is_jolt_file(dir_entry: &DirEntry) -> bool {
    dir_entry.path().file_name().is_some_and(|e| e == ".jolt")
}

pub fn dir_entry_is_cue(dir_entry: &DirEntry) -> bool {
    dir_entry_is_file(dir_entry) && dir_entry_has_cue_extension(dir_entry)
}
