use std::{num::NonZero, path::Path, time::Duration};

use game_music_emu::GameMusicEmu;
use rodio::{Source, source::SeekError};

const SAMPLE_RATE: u32 = 44100;
const CHANNELS: u16 = 2;
/// Refill chunk size (must be even for stereo interleaving).
const CHUNK_SAMPLES: usize = 2048;

pub struct ChiptuneSource {
    emu: GameMusicEmu,
    buf: Vec<i16>,
    buf_pos: usize,
    exhausted: bool,
}

impl ChiptuneSource {
    pub fn from_file(path: &Path) -> Result<Self, String> {
        let emu = GameMusicEmu::from_file(path, SAMPLE_RATE).map_err(|e| format!("gme open: {e:?}"))?;
        emu.start_track(0).map_err(|e| format!("gme start_track: {e:?}"))?;
        Ok(Self {
            emu,
            buf: Vec::with_capacity(CHUNK_SAMPLES),
            buf_pos: 0,
            exhausted: false,
        })
    }

    fn refill(&mut self) {
        if self.exhausted || self.emu.track_ended() {
            self.exhausted = true;
            self.buf.clear();
            return;
        }
        self.buf.resize(CHUNK_SAMPLES, 0);
        if let Err(e) = self.emu.play(CHUNK_SAMPLES, &mut self.buf) {
            log::error!("gme play error: {e:?}");
            self.exhausted = true;
            self.buf.clear();
            return;
        }
        self.buf_pos = 0;
    }
}

impl Iterator for ChiptuneSource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        if self.buf_pos >= self.buf.len() {
            self.refill();
            if self.buf.is_empty() {
                return None;
            }
        }
        let s = self.buf[self.buf_pos];
        self.buf_pos += 1;
        Some(s as f32 / i16::MAX as f32)
    }
}

impl Source for ChiptuneSource {
    fn current_span_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> NonZero<u16> {
        NonZero::new(CHANNELS).unwrap()
    }

    fn sample_rate(&self) -> NonZero<u32> {
        NonZero::new(SAMPLE_RATE).unwrap()
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }

    fn try_seek(&mut self, _pos: Duration) -> Result<(), SeekError> {
        Err(SeekError::NotSupported {
            underlying_source: "ChiptuneSource",
        })
    }
}
