use std::{error::Error, sync::Arc, thread};

use mpris_server::{
    LoopStatus,
    Metadata,
    PlaybackRate,
    PlaybackStatus,
    PlayerInterface,
    Property,
    RootInterface,
    Server,
    Time,
    TrackId,
    Volume,
    zbus,
};
use tokio::sync::Mutex;

use crate::structs::Song;

#[derive(Default)]
pub struct MprisState {
    on_play_pause: Arc<Mutex<Option<Box<dyn Fn() + Send + Sync + 'static>>>>,
    on_stop: Arc<Mutex<Option<Box<dyn Fn() + Send + Sync + 'static>>>>,
}

impl MprisState {
    pub fn on_play_pause(&self, f: impl Fn() + Send + Sync + 'static) {
        *self.on_play_pause.blocking_lock() = Some(Box::new(f));
    }
    pub fn on_stop(&self, f: impl Fn() + Send + Sync + 'static) {
        *self.on_stop.blocking_lock() = Some(Box::new(f));
    }
}

#[allow(unused)]
impl RootInterface for MprisState {
    async fn raise(&self) -> zbus::fdo::Result<()> {
        Ok(())
    }

    async fn quit(&self) -> zbus::fdo::Result<()> {
        Ok(())
    }

    async fn can_quit(&self) -> zbus::fdo::Result<bool> {
        Ok(false)
    }

    async fn fullscreen(&self) -> zbus::fdo::Result<bool> {
        Ok(false)
    }

    async fn set_fullscreen(&self, fullscreen: bool) -> zbus::Result<()> {
        Ok(())
    }

    async fn can_set_fullscreen(&self) -> zbus::fdo::Result<bool> {
        Ok(false)
    }

    async fn can_raise(&self) -> zbus::fdo::Result<bool> {
        Ok(false)
    }

    async fn has_track_list(&self) -> zbus::fdo::Result<bool> {
        Ok(false)
    }

    async fn identity(&self) -> zbus::fdo::Result<String> {
        Ok("Jolteon".to_string())
    }

    async fn desktop_entry(&self) -> zbus::fdo::Result<String> {
        Ok("".to_string())
    }

    async fn supported_uri_schemes(&self) -> zbus::fdo::Result<Vec<String>> {
        Ok(vec!["file".to_string()])
    }

    async fn supported_mime_types(&self) -> zbus::fdo::Result<Vec<String>> {
        Ok(vec![
            "audio/flac".to_string(),
            "audio/x-flac".to_string(),
            "audio/mpeg".to_string(),
            "audio/x-wav".to_string(),
            "audio/ogg".to_string(),
            "audio/vorbis".to_string(),
            "audio/x-ape".to_string(),
            "application/ogg".to_string(),
        ])
    }
}

#[allow(unused)]
impl PlayerInterface for MprisState {
    async fn next(&self) -> zbus::fdo::Result<()> {
        let on_stop = self.on_stop.lock().await;
        let Some(on_stop) = &*on_stop else {
            return Ok(());
        };
        on_stop();
        Ok(())
    }

    async fn previous(&self) -> zbus::fdo::Result<()> {
        Ok(())
    }

    async fn pause(&self) -> zbus::fdo::Result<()> {
        let on_play_pause = self.on_play_pause.lock().await;
        let Some(on_play_pause) = &*on_play_pause else {
            return Ok(());
        };
        on_play_pause();
        Ok(())
    }

    async fn play_pause(&self) -> zbus::fdo::Result<()> {
        let on_play_pause = self.on_play_pause.lock().await;
        let Some(on_play_pause) = &*on_play_pause else {
            return Ok(());
        };
        on_play_pause();
        Ok(())
    }

    async fn stop(&self) -> zbus::fdo::Result<()> {
        let on_stop = self.on_stop.lock().await;
        let Some(on_stop) = &*on_stop else {
            return Ok(());
        };
        on_stop();
        Ok(())
    }

    async fn play(&self) -> zbus::fdo::Result<()> {
        let on_play_pause = self.on_play_pause.lock().await;
        let Some(on_play_pause) = &*on_play_pause else {
            return Ok(());
        };
        on_play_pause();
        Ok(())
    }

    async fn seek(&self, offset: Time) -> zbus::fdo::Result<()> {
        Ok(())
    }

    async fn set_position(&self, track_id: TrackId, position: Time) -> zbus::fdo::Result<()> {
        Ok(())
    }

    async fn open_uri(&self, uri: String) -> zbus::fdo::Result<()> {
        Ok(())
    }

    async fn playback_status(&self) -> zbus::fdo::Result<PlaybackStatus> {
        Ok(PlaybackStatus::Stopped)
    }

    async fn loop_status(&self) -> zbus::fdo::Result<LoopStatus> {
        Ok(LoopStatus::None)
    }

    async fn set_loop_status(&self, loop_status: LoopStatus) -> zbus::Result<()> {
        Ok(())
    }

    async fn rate(&self) -> zbus::fdo::Result<PlaybackRate> {
        Ok(1.0)
    }

    async fn set_rate(&self, rate: PlaybackRate) -> zbus::Result<()> {
        Ok(())
    }

    async fn shuffle(&self) -> zbus::fdo::Result<bool> {
        Ok(true)
    }

    async fn set_shuffle(&self, shuffle: bool) -> zbus::Result<()> {
        Ok(())
    }

    async fn metadata(&self) -> zbus::fdo::Result<Metadata> {
        Ok(Metadata::new())
    }

    async fn volume(&self) -> zbus::fdo::Result<Volume> {
        Ok(1.0)
    }

    async fn set_volume(&self, volume: Volume) -> zbus::Result<()> {
        Ok(())
    }

    async fn position(&self) -> zbus::fdo::Result<Time> {
        Ok(Time::ZERO)
    }

    async fn minimum_rate(&self) -> zbus::fdo::Result<PlaybackRate> {
        Ok(1.0)
    }

    async fn maximum_rate(&self) -> zbus::fdo::Result<PlaybackRate> {
        Ok(1.0)
    }

    async fn can_go_next(&self) -> zbus::fdo::Result<bool> {
        Ok(false)
    }

    async fn can_go_previous(&self) -> zbus::fdo::Result<bool> {
        Ok(false)
    }

    async fn can_play(&self) -> zbus::fdo::Result<bool> {
        Ok(false)
    }

    async fn can_pause(&self) -> zbus::fdo::Result<bool> {
        Ok(false)
    }

    async fn can_seek(&self) -> zbus::fdo::Result<bool> {
        Ok(false)
    }

    async fn can_control(&self) -> zbus::fdo::Result<bool> {
        Ok(true)
    }
}

pub struct Mpris {
    server: Arc<Mutex<Server<MprisState>>>,
}

impl Mpris {
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        let server = Server::new("jolteon", MprisState::default()).await?;

        Ok(Self {
            server: Arc::new(Mutex::new(server)),
        })
    }

    pub fn on_play_pause(&self, f: impl Fn() + Send + Sync + 'static) {
        let s = self.server.blocking_lock();
        s.imp().on_play_pause(f);
    }

    pub fn on_stop(&self, f: impl Fn() + Send + Sync + 'static) {
        let s = self.server.blocking_lock();
        s.imp().on_stop(f);
    }

    fn emit_properties_changed(&self, properties: impl IntoIterator<Item = Property> + Send + Sync + 'static) {
        let server = self.server.clone();
        thread::spawn(move || {
            futures::executor::block_on(async move {
                let server = server.lock().await;
                server.properties_changed(properties).await.unwrap();
            });
        });
    }

    pub fn set_song(&self, song: Song) {
        let mut metadata = Metadata::new();
        metadata.set_title(Some(song.title));
        metadata.set_artist(song.artist.map(|a| vec![a]));
        metadata.set_album(song.album);
        metadata.set_length(Some(Time::from_secs(song.length.as_secs() as i64)));

        self.emit_properties_changed(vec![
            Property::Metadata(metadata),
            Property::CanPlay(true),
            Property::CanPause(true),
            Property::CanGoNext(true),
            Property::CanSeek(true),
        ]);
    }

    pub fn clear_song(&self) {
        self.emit_properties_changed(vec![
            Property::Metadata(Metadata::new()),
            Property::PlaybackStatus(PlaybackStatus::Stopped),
            Property::CanPlay(false),
            Property::CanGoNext(false),
            Property::CanSeek(false),
        ]);
    }

    pub fn play(&self) {
        self.emit_properties_changed(vec![Property::PlaybackStatus(PlaybackStatus::Playing)]);
    }

    pub fn pause(&self) {
        self.emit_properties_changed(vec![Property::PlaybackStatus(PlaybackStatus::Paused)])
    }
}
