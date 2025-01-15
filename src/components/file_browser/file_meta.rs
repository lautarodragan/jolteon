use std::cell::RefCell;

use ratatui::{buffer::Buffer, layout::Rect, widgets::WidgetRef};

use crate::{
    components::{FileBrowserSelection, List},
    config::Theme,
    cue::CueSheet,
    duration::duration_to_string,
    structs::{Action, Jolt, OnAction, Song},
    ui::Focusable,
};

pub struct FileMeta<'a> {
    file: RefCell<Option<FileBrowserSelection>>,
    list: List<'a, String>,
}

impl FileMeta<'_> {
    pub fn new(theme: Theme) -> Self {
        Self {
            file: RefCell::new(None),
            list: List::new(theme, vec![]),
        }
    }

    pub fn set_file(&self, file: FileBrowserSelection) {
        match file {
            FileBrowserSelection::Song(ref song) => {
                self.set_song(song);
            }
            FileBrowserSelection::CueSheet(ref cue) => {
                self.set_cue(cue);
            }
            FileBrowserSelection::Jolt(ref jolt) => {
                self.set_jolt(jolt);
            }
            FileBrowserSelection::Other(ref path) => {
                self.list
                    .set_items(vec!["File:".to_string(), format!("  {}", path.to_string_lossy())]);
                if let Ok(meta) = path.metadata() {
                    let mut unit = "bytes";
                    let mut size = meta.len() as f64;

                    if size > 1024.0 {
                        size /= 1024.0;
                        unit = "KiB";
                    }

                    if size > 1024.0 {
                        size /= 1024.0;
                        unit = "MiB";
                    }

                    self.list.push_item(format!("  Size: {size:.2} {unit}"))
                }
            }
            FileBrowserSelection::Directory(ref path) => {
                self.list
                    .set_items(vec!["Folder:".to_string(), format!("  {}", path.to_string_lossy())]);
                if let Ok(children) = path.read_dir() {
                    let (files, folders) = children
                        .filter_map(|c| c.ok())
                        .filter_map(|c| c.file_type().ok())
                        .fold((0, 0), |(files, folders), ft| {
                            (files + ft.is_file() as usize, folders + ft.is_dir() as usize)
                        });

                    self.list.push_item(format!("  Files: {files}"));
                    self.list.push_item(format!("  Folders: {folders}"));
                }
            }
        }

        let mut s = self.file.borrow_mut();
        *s = Some(file);
    }

    pub fn set_song(&self, song: &Song) {
        let tags = song.get_tags();

        let mut tags: Vec<(String, String)> = tags
            .iter()
            .flat_map(|tag| {
                tag.items().map(|item| {
                    let key = format!("{:?} {:?}", tag.tag_type(), item.key());
                    let value = format!("{:?}", item.value());
                    (key, value)
                })
            })
            .collect();

        tags.sort_by(|a, b| a.0.cmp(&b.0));

        let max_key_len: usize = tags.iter().fold(0, |acc, e| acc.max(e.0.len()));
        let tags = tags.iter().map(|(k, v)| format!("{k:<max_key_len$} {v}")).collect();
        self.list.set_items(tags);
    }

    pub fn set_cue(&self, cue: &CueSheet) {
        let mut items: Vec<String> = vec![];

        if let Some(performer) = cue.performer() {
            items.push(format!("Performer: {performer}"));
        }
        if let Some(title) = cue.title() {
            items.push(format!("Title: {title}"));
        }

        if let Some(file) = cue.file() {
            items.push(format!("File: {}", file.name()));
            items.push(" ".to_string());
            items.push("Tracks:".to_string());
            for track in file.tracks() {
                items.push(format!("  {} {}", track.index(), track.title()));
                items.push(format!("    Start: {}", duration_to_string(track.start_time())));
                if let Some(per) = track.performer() {
                    items.push(format!("    Performer: {per}"));
                }
            }
        }

        items.push(" ".to_string());
        items.push("Comments:".to_string());
        for comment in cue.comments() {
            items.push(format!("  {comment}"));
        }

        items.push(" ".to_string());
        items.push("Unknown Tags:".to_string());
        let u = cue.unknowns();
        if u.is_empty() {
            items.push("  (no unknown tags)".to_string());
        } else {
            for unknown in cue.unknowns() {
                items.push(format!("  {unknown}"));
            }
        }

        self.list.set_items(items);
    }

    pub fn set_jolt(&self, jolt: &Jolt) {
        let mut items: Vec<String> = vec![];

        items.push(format!("Artist: {:?}", jolt.artist));
        items.push(format!("Album: {:?}", jolt.album));
        items.push(format!("Disc Number: {:?}", jolt.disc_number));

        self.list.set_items(items);
    }

    pub fn clear(&self) {
        self.list.set_items(vec![]);
        let mut s = self.file.borrow_mut();
        *s = None;
    }
}

impl WidgetRef for FileMeta<'_> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        self.list.render_ref(area, buf);
    }
}

impl OnAction for FileMeta<'_> {
    fn on_action(&self, action: Action) {
        self.list.on_action(action);
    }
}

impl Focusable for FileMeta<'_> {
    fn set_is_focused(&self, v: bool) {
        self.list.set_is_focused(v);
    }

    fn is_focused(&self) -> bool {
        self.list.is_focused()
    }
}
