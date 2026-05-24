//! Thin FFI bindings to libgme symbols not exposed by the `game-music-emu` crate.
//!
//! libgme is already linked into the binary via the `game-music-emu` dependency;
//! we just declare additional `extern "C"` entry points here so we can read
//! per-track metadata (`gme_track_info`).

use std::{
    ffi::{CStr, CString},
    os::raw::{c_char, c_int},
    path::Path,
    ptr,
    time::Duration,
};

#[repr(C)]
struct MusicEmu {
    _private: [u8; 0],
}

#[repr(C)]
struct GmeInfo {
    length: c_int,
    intro_length: c_int,
    loop_length: c_int,
    play_length: c_int,
    // 12 reserved ints
    _reserved_int: [c_int; 12],
    system: *const c_char,
    game: *const c_char,
    song: *const c_char,
    author: *const c_char,
    copyright: *const c_char,
    comment: *const c_char,
    dumper: *const c_char,
    // 9 reserved pointers
    _reserved_ptr: [*const c_char; 9],
}

unsafe extern "C" {
    fn gme_open_file(path: *const c_char, out: *mut *mut MusicEmu, sample_rate: c_int) -> *const c_char;
    fn gme_delete(emu: *mut MusicEmu);
    fn gme_track_info(emu: *const MusicEmu, out: *mut *mut GmeInfo, track: c_int) -> *const c_char;
    fn gme_free_info(info: *mut GmeInfo);
    fn gme_track_count(emu: *const MusicEmu) -> c_int;
}

#[derive(Debug, Default, Clone)]
pub struct ChiptuneInfo {
    pub system: Option<String>,
    pub game: Option<String>,
    pub song: Option<String>,
    pub author: Option<String>,
    pub copyright: Option<String>,
    pub comment: Option<String>,
    /// libgme's `play_length` — uses file-declared length, else intro+loop*2, else 2.5min default.
    pub play_length: Duration,
    pub track_count: usize,
}

fn cstr_to_string(p: *const c_char) -> Option<String> {
    if p.is_null() {
        return None;
    }
    let s = unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned();
    if s.is_empty() { None } else { Some(s) }
}

fn check_err(err: *const c_char) -> Result<(), String> {
    if err.is_null() {
        Ok(())
    } else {
        let msg = unsafe { CStr::from_ptr(err) }.to_string_lossy().into_owned();
        Err(msg)
    }
}

/// Reads info for `track` (0-indexed) from the file at `path`.
pub fn read_track_info(path: &Path, track: usize) -> Result<ChiptuneInfo, String> {
    let path_cstr = CString::new(path.as_os_str().as_encoded_bytes()).map_err(|e| e.to_string())?;

    let mut emu: *mut MusicEmu = ptr::null_mut();
    unsafe {
        check_err(gme_open_file(path_cstr.as_ptr(), &mut emu as *mut _, 44100))?;
    }
    if emu.is_null() {
        return Err("gme_open_file returned null emu".to_string());
    }

    let result = (|| unsafe {
        let track_count = gme_track_count(emu) as usize;
        let mut info_ptr: *mut GmeInfo = ptr::null_mut();
        check_err(gme_track_info(emu, &mut info_ptr as *mut _, track as c_int))?;
        if info_ptr.is_null() {
            return Err("gme_track_info returned null info".to_string());
        }
        let info = &*info_ptr;
        let play_length_ms = if info.play_length < 0 {
            150_000
        } else {
            info.play_length
        };
        let parsed = ChiptuneInfo {
            system: cstr_to_string(info.system),
            game: cstr_to_string(info.game),
            song: cstr_to_string(info.song),
            author: cstr_to_string(info.author),
            copyright: cstr_to_string(info.copyright),
            comment: cstr_to_string(info.comment),
            play_length: Duration::from_millis(play_length_ms as u64),
            track_count,
        };
        gme_free_info(info_ptr);
        Ok(parsed)
    })();

    unsafe { gme_delete(emu) };
    result
}
