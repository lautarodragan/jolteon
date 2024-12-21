use std::error::Error;
use std::sync::mpsc::Sender;
use mpris_server::Metadata;

use crate::Command;

pub async fn create_mpris_player(
    player_command_sender: Sender<Command>,
) -> Result<mpris_server::Player, Box<dyn Error>> {
    let player = mpris_server::Player::builder("com.taro-codes.jolteon")
        .can_play(true)
        .can_pause(true)
        .can_go_next(true)
        .build()
        .await?;

    player.set_identity("Jolteon").await?;

    // let mut meta = Metadata::new();
    // meta.set_album(Some("album"));
    // meta.set_artist(Some(["artist"]));
    // meta.set_title(Some("song title"));
    // player.set_metadata(meta).await?;

    player.connect_play_pause({
        let player_command_sender = player_command_sender.clone();
        move |_player| {
            if let Err(err) = player_command_sender.send(Command::PlayPause) {
                log::warn!("mpris: Failed to send play_pause! {:?}", err);
            }
        }
    });

    player.connect_next({
        let player_command_sender = player_command_sender.clone();
        move |_player| {
            if let Err(err) = player_command_sender.send(Command::Next) {
                log::warn!("mpris: Failed to send next! {:?}", err);
            }
        }
    });

    player.connect_quit(|_player| {
        log::trace!("mpris quit");
    });

    player.connect_stop({
        let player_command_sender = player_command_sender.clone();
        move |_player| {
            log::trace!("mpris stop");
            if let Err(err) = player_command_sender.send(Command::Next) {
                log::warn!("mpris: Failed to send next! {:?}", err);
            }
        }
    });

    Ok(player)
}
