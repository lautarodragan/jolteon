use std::{
    fs::File,
    io::BufReader,
    num::NonZero,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Duration,
};

use rodio::{
    Decoder,
    Source as RodioSource,
    source::{Amplify, Pausable, PeriodicAccess, SeekError, Skippable, Speed, Stoppable, TrackPosition},
};

use crate::components::path_is_chiptune;

mod chiptune;
mod chiptune_ffi;

use chiptune::ChiptuneSource;
pub use chiptune_ffi::read_track_info as read_chiptune_track_info;

type InnerSource = Box<dyn RodioSource<Item = f32> + Send>;
type FullRodioSource = Stoppable<Skippable<Amplify<Pausable<TrackPosition<Speed<InnerSource>>>>>>;
type PeriodicRodioSource<F> = PeriodicAccess<FullRodioSource, F>;

pub struct Controls<'a> {
    src: &'a mut FullRodioSource,
    shared_pos: &'a Arc<Mutex<Duration>>,
}

impl Controls<'_> {
    #[inline]
    pub fn stop(&mut self) {
        self.src.stop();
        self.set_pos(Duration::ZERO);
    }

    #[inline]
    pub fn skip(&mut self) {
        self.src.inner_mut().skip();
    }

    #[inline]
    pub fn pos(&self) -> Duration {
        self.src.inner().inner().inner().inner().get_pos()
    }

    #[inline]
    pub fn set_pos(&self, pos: Duration) {
        *self.shared_pos.lock().unwrap() = pos;
    }

    #[inline]
    pub fn refresh_pos(&self) {
        self.set_pos(self.pos());
    }

    #[inline]
    pub fn set_volume(&mut self, factor: f32) {
        self.src.inner_mut().inner_mut().set_factor(factor)
    }

    #[inline]
    pub fn set_paused(&mut self, paused: bool) {
        self.src.inner_mut().inner_mut().inner_mut().set_paused(paused)
    }

    #[inline]
    pub fn seek(&mut self, position: Duration) -> Result<(), SeekError> {
        self.src.try_seek(position)
    }
}

pub struct Source<F> {
    input: PeriodicRodioSource<F>,
    on_playback_end: Option<Box<dyn FnOnce() + Send + 'static>>,
}

fn build_inner(path: &Path) -> Result<InnerSource, String> {
    if path_is_chiptune(path) {
        let src = ChiptuneSource::from_file(path).map_err(|e| e.to_string())?;
        Ok(Box::new(src))
    } else {
        let file = BufReader::new(File::open(path).map_err(|e| e.to_string())?);
        let decoder = Decoder::new(file).map_err(|e| e.to_string())?;
        Ok(Box::new(decoder))
    }
}

impl Source<()> {
    pub fn from_file(
        path: PathBuf,
        periodic_access: impl Fn(&mut Controls) + Send,
        shared_pos: Arc<Mutex<Duration>>,
        on_playback_end: impl FnOnce() + Send + 'static,
    ) -> Result<Source<Box<impl FnMut(&mut FullRodioSource) + Send>>, String> {
        let periodic_access_inner = {
            Box::new(move |src: &mut FullRodioSource| {
                let mut controls = Controls {
                    src,
                    shared_pos: &shared_pos,
                };
                controls.refresh_pos();
                periodic_access(&mut controls);
            })
        };

        let source = build_inner(&path)?;
        let input = source
            .speed(1.0)
            .track_position()
            .pausable(false)
            .amplify(1.0)
            .skippable()
            .stoppable()
            .periodic_access(Duration::from_millis(5), periodic_access_inner);

        Ok(Source {
            input,
            on_playback_end: Some(Box::new(on_playback_end)),
        })
    }
}

impl<F: FnMut(&mut FullRodioSource) + Send> Source<F>
where
    F: FnMut(&mut FullRodioSource) + Send,
{
    #[inline]
    pub fn _inner_mut(&mut self) -> &mut PeriodicRodioSource<F> {
        &mut self.input
    }

    pub fn seek(&mut self, pos: Duration) -> Result<(), SeekError> {
        let i = self.input.inner_mut().inner_mut().inner_mut();
        i.try_seek(pos)
    }
}

impl<F> Iterator for Source<F>
where
    F: FnMut(&mut FullRodioSource),
{
    type Item = f32;

    #[inline]
    fn next(&mut self) -> Option<f32> {
        let n = self.input.next();

        if n.is_none() {
            if let Some(cb) = self.on_playback_end.take() {
                cb();
            }
        }

        n
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.input.size_hint()
    }
}

impl<F> RodioSource for Source<F>
where
    F: FnMut(&mut FullRodioSource),
{
    #[inline]
    fn current_span_len(&self) -> Option<usize> {
        self.input.current_span_len()
    }

    #[inline]
    fn channels(&self) -> NonZero<u16> {
        self.input.channels()
    }

    #[inline]
    fn sample_rate(&self) -> NonZero<u32> {
        self.input.sample_rate()
    }

    #[inline]
    fn total_duration(&self) -> Option<Duration> {
        self.input.total_duration()
    }

    #[inline]
    fn try_seek(&mut self, pos: Duration) -> Result<(), SeekError> {
        self.input.try_seek(pos)
    }
}

impl<F> Drop for Source<F> {
    fn drop(&mut self) {
        log::trace!("Source.drop()");
    }
}
