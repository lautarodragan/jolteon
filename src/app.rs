use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

use async_std::task;
use crossterm::event;
use crossterm::event::Event;
use rodio::OutputStream;

use crate::{
    components::*,
    config::Theme,
    main_player::MainPlayer,
    mpris::Mpris,
    player::SingleTrackPlayer,
    state::State,
    structs::{Action, Actions, OnAction, OnActionMut, Queue},
    term::set_terminal,
    ui::KeyboardHandlerMut,
};

pub async fn run() -> Result<(), Box<dyn Error>> {
    let mpris = Mpris::new().await?;

    task::spawn_blocking(move || {
        if let Err(err) = run_sync(mpris) {
            log::error!("{err:?}");
        }
    })
    .await;

    Ok(())
}

fn run_sync(mpris: Mpris) -> Result<(), Box<dyn Error>> {
    let actions = Actions::from_file_or_default();
    assert!(
        actions.contains(Action::Quit),
        "No key binding for Action::Quit! User would not be able to exit Jolteon. This is 100% a bug."
    );

    let mut terminal = set_terminal()?;

    let theme = include_str!("../assets/theme.toml");
    let theme: Theme = toml::from_str(theme).unwrap();

    let state = State::from_file();

    // if _output_stream is dropped playback will end & attached `OutputStreamHandle`s will no longer work.
    // Creating the output_stream indirectly spawns the cpal_alsa_out thread, and creates the mixer tied to it.
    let (_output_stream, output_stream_handle) = OutputStream::try_default()?;

    let mpris = Arc::new(mpris);
    let single_track_player = Arc::new(SingleTrackPlayer::new(output_stream_handle, mpris.clone()));
    let queue = Arc::new(Queue::new(state.queue_items));
    let main_player = MainPlayer::spawn(single_track_player.clone(), queue.clone());

    mpris.on_play_pause({
        let player = single_track_player.clone();
        move || {
            player.toggle();
        }
    });
    mpris.on_stop({
        let player = single_track_player.clone();
        move || {
            player.stop();
        }
    });

    single_track_player.spawn();

    let mut app = Root::new(theme, queue.clone(), single_track_player.clone());

    let tick_rate = Duration::from_millis(100);
    let mut last_tick = std::time::Instant::now();

    loop {
        if queue.length() != app.queue_ui.len() {
            queue.with_items(|songs| {
                // See src/README.md to make sense of this
                app.queue_ui.set_items(Vec::from(songs.clone()));
            });
        }

        terminal.draw(|frame| {
            frame.render_widget_ref(&app, frame.area());
        })?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if let Some(action) = actions.action_by_key(key) {
                    if action == Action::Quit {
                        break;
                    }

                    match action {
                        Action::Player(action) => {
                            single_track_player.on_action(action);
                        }
                        Action::MainPlayer(action) => {
                            main_player.on_action(action);
                        }
                        Action::Screen(action) => {
                            app.on_action(action);
                        }
                        _ => {
                            app.on_key(key);
                        }
                    }
                } else {
                    app.on_key(key);
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = std::time::Instant::now();
        }
    }

    main_player.quit();

    let state = State {
        last_visited_path: app.browser.current_directory().to_str().map(String::from),
        queue_items: Vec::from(queue.songs().clone()),
    };

    if let Err(err) = state.to_file() {
        log::error!("Could not save app state {err:?}");
    }

    Ok(())
}
