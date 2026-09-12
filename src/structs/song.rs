use std::{
    cmp::Ordering,
    fs::DirEntry,
    path::{Path, PathBuf},
    time::Duration,
};

use lofty::{
    error::FileParseError,
    file::{AudioFile, TaggedFileExt},
    probe::Probe,
    tag::Accessor,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    components::{FileBrowserSelection, dir_entry_is_song, directory_to_songs_and_folders, path_is_chiptune},
    cue::{CueFile, CueSheet},
    source::read_chiptune_track_info,
    structs::Jolt,
};

/// Default playback length for chiptune tracks when libgme reports no duration.
const CHIPTUNE_DEFAULT_LENGTH: Duration = Duration::from_secs(150);

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct Song {
    pub library_id: Option<Uuid>,
    pub path: PathBuf,
    pub start_time: Duration,
    pub length: Duration,
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub soundtrack_subject: Option<String>,
    pub disc_number: Option<u32>,
    pub track: Option<u32>,
    pub year: Option<u32>,
}

fn find_closest_jolt(path: &Path) -> Option<Jolt> {
    path.ancestors()
        .find_map(|ancestor| Jolt::from_path(ancestor.join(".jolt")).ok())
}

#[derive(Default)]
struct RawMeta {
    title: Option<String>,
    artist: Option<String>,
    album: Option<String>,
    soundtrack_subject: Option<String>,
    length: Duration,
    track: Option<u32>,
    disc_number: Option<u32>,
    year: Option<u32>,
}

fn read_chiptune_meta(path: &Path) -> RawMeta {
    match read_chiptune_track_info(path, 0) {
        Ok(i) => RawMeta {
            title: i.song,
            artist: i.author,
            album: i.game,
            soundtrack_subject: i.system,
            length: if i.play_length > Duration::ZERO {
                i.play_length
            } else {
                CHIPTUNE_DEFAULT_LENGTH
            },
            ..Default::default()
        },
        Err(err) => {
            log::warn!("chiptune metadata read failed for {path:?}: {err}");
            RawMeta {
                length: CHIPTUNE_DEFAULT_LENGTH,
                ..Default::default()
            }
        }
    }
}

fn read_lofty_meta(path: &Path) -> Result<RawMeta, FileParseError> {
    let tagged_file = Probe::open(path)?.read()?;
    let length = tagged_file.properties().duration();
    let raw = match tagged_file.primary_tag() {
        Some(t) => RawMeta {
            title: t.title().map(String::from),
            artist: t.artist().map(String::from),
            album: t.album().map(String::from),
            track: t.track(),
            year: t.date().map(|date| u32::from(date.year)),
            // TODO: disc number is sometimes stored as a Text, including disk sides ("A1"). `.disk` returns `None` in these cases.
            disc_number: t.disk(),
            length,
            ..Default::default()
        },
        None => RawMeta {
            length,
            ..Default::default()
        },
    };
    Ok(raw)
}

impl Song {
    pub fn from_file(path: &Path) -> Result<Self, FileParseError> {
        let meta = if path_is_chiptune(path) {
            read_chiptune_meta(path)
        } else {
            read_lofty_meta(path)?
        };
        let jolt = find_closest_jolt(path);

        Ok(Song {
            library_id: None,
            path: PathBuf::from(path),
            start_time: Duration::ZERO,
            length: meta.length,
            title: meta
                .title
                .unwrap_or_else(|| path.file_name().unwrap().to_str().unwrap().to_string()),
            artist: jolt.as_ref().and_then(|j| j.artist.clone()).or(meta.artist),
            album: jolt.as_ref().and_then(|j| j.album.clone()).or(meta.album),
            soundtrack_subject: jolt
                .as_ref()
                .and_then(|j| j.soundtrack_subject.clone())
                .or(meta.soundtrack_subject),
            disc_number: meta.disc_number,
            track: meta.track,
            year: jolt.as_ref().and_then(|j| j.year).or(meta.year),
        })
    }

    pub fn from_dir(path: &Path) -> Vec<Self> {
        // TODO: improve this. stop using the FileBrowser stuff.
        //   check for songs, cue
        let entries = directory_to_songs_and_folders(path, true);
        let jolt = find_closest_jolt(path);

        log::trace!(target: "::Song::from_dir", "{jolt:#?}");

        entries
            .iter()
            .filter_map(|s| {
                if let FileBrowserSelection::Song(song) = s {
                    let mut song = song.clone();

                    if let Some(ref jolt) = jolt {
                        if jolt.album.is_some() {
                            song.album.clone_from(&jolt.album);
                        }
                        if jolt.artist.is_some() {
                            song.artist.clone_from(&jolt.artist);
                        }
                    }

                    Some(song)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn from_cue_file(cue_sheet: &CueSheet, cue_file: CueFile) -> Vec<Self> {
        let performer = cue_sheet.performer();
        let file_name = cue_file.name();
        let tracks = cue_file.tracks();

        let cue_path = cue_sheet.cue_sheet_file_path();
        let mut song_path = cue_path.parent().unwrap().join(file_name);

        if !song_path.exists() {
            log::warn!("File path doesn't exist: {song_path:?}");
            let Ok(entries) = cue_path.parent().unwrap().read_dir() else {
                log::warn!("Error attempting to list files in dir of {song_path:?}");
                return Vec::new();
            };

            let candidates: Vec<DirEntry> = entries.filter_map(|e| e.ok()).filter(dir_entry_is_song).collect();

            if candidates.is_empty() {
                log::warn!("Found no candidates at all. Nothing to do.");
                return Vec::new();
            }

            if candidates.len() > 1 {
                log::warn!("Found more than one candidate. Will arbitrarily pick one.");
            }

            log::debug!("Found {candidates:?}");

            // TODO: communicate to user

            song_path = candidates[0].path();
        }

        let song = match Song::from_file(&song_path) {
            Ok(s) => s,
            Err(err) => {
                log::warn!(target: "::song.from_cue_sheet", "Could not load songs from cue sheet.");
                log::warn!(target: "::song.from_cue_sheet", "Cue sheet path: {cue_path:?}");
                log::warn!(target: "::song.from_cue_sheet", "Error: {err:#?}");
                log::warn!(target: "::song.from_cue_sheet", "Full cue sheet: {cue_sheet:#?}");
                return Vec::new();
            }
        };

        let jolt = find_closest_jolt(song_path.as_path());

        // TODO: attempt to read date from REM DATE comment
        let cue_date = cue_sheet
            .comments()
            .into_iter()
            .find(|comment| comment.starts_with("DATE "));

        log::debug!("DATE from cue sheet: {cue_date:?}");

        let cue_year: Option<u32> = cue_date.and_then(|date| date[5..].parse().ok());

        log::debug!("DATE from cue sheet: {cue_year:?}");

        let mut songs: Vec<Song> = tracks
            .iter()
            .map(|t| Song {
                library_id: None,
                path: song_path.clone(),
                length: Duration::ZERO,
                artist: jolt
                    .as_ref()
                    .and_then(|j| j.artist.clone())
                    .or(performer.clone())
                    .or(t.performer()),
                title: t.title(),
                start_time: t.start_time(),
                album: jolt.as_ref().and_then(|j| j.album.clone()).or(cue_sheet.title()),
                soundtrack_subject: jolt.as_ref().and_then(|j| j.soundtrack_subject.clone()),
                track: t.index().split_whitespace().nth(0).and_then(|i| i.parse().ok()),
                year: jolt.as_ref().and_then(|j| j.year).or(song.year).or(cue_year),
                disc_number: jolt.as_ref().and_then(|j| j.disc_number), // There seems to be no standard disc number field for Cue Sheets...
            })
            .collect();

        for i in 0..songs.len() {
            let next_start = if i < songs.len() - 1 {
                songs[i + 1].start_time
            } else {
                song.length
            };
            let this_start = songs[i].start_time;
            songs[i].length = next_start.saturating_sub(this_start);
        }

        songs
    }

    pub fn from_cue_sheet(cue_sheet: CueSheet) -> Vec<Self> {
        cue_sheet
            .files()
            .into_iter()
            .flat_map(|cue_file| Self::from_cue_file(&cue_sheet, cue_file))
            .collect()
    }

    pub fn get_tags(&self) -> Vec<lofty::tag::Tag> {
        if path_is_chiptune(&self.path) {
            return Vec::new();
        }
        let Ok(probe) = Probe::open(&self.path) else {
            return Vec::new();
        };
        match probe.read() {
            Ok(tagged_file) => tagged_file.tags().to_vec(),
            Err(err) => {
                log::warn!("get_tags: lofty read failed for {:?}: {err:?}", self.path);
                Vec::new()
            }
        }
    }
}

impl Ord for Song {
    fn cmp(&self, other: &Self) -> Ordering {
        match (&self.album, &other.album) {
            (Some(album_a), Some(album_b)) if album_a == album_b => match self.disc_number.cmp(&other.disc_number) {
                Ordering::Equal => match (&self.track, &other.track) {
                    (Some(a), Some(b)) => a.cmp(b),
                    (Some(_), None) => Ordering::Greater,
                    (None, Some(_)) => Ordering::Less,
                    _ => self.title.cmp(&other.title),
                },
                o => o,
            },
            (Some(album_a), Some(album_b)) if album_a != album_b => match (self.year, other.year) {
                (Some(ref year_a), Some(ref year_b)) => {
                    if year_a != year_b {
                        year_a.cmp(year_b)
                    } else {
                        album_a.cmp(album_b)
                    }
                }
                (Some(_), None) => Ordering::Greater,
                (None, Some(_)) => Ordering::Less,
                _ => album_a.cmp(album_b),
            },
            (Some(_), None) => Ordering::Greater,
            (None, Some(_)) => Ordering::Less,
            _ => self.title.cmp(&other.title),
        }
    }
}

impl PartialOrd for Song {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
