use std::{
    error::Error,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use async_std::task;
use crossterm::{event, event::Event};
use rodio::OutputStream;

use crate::{
    actions::{Action, Actions, OnAction, OnActionMut},
    components::*,
    config::Theme,
    main_player::MainPlayer,
    mpris::Mpris,
    settings::Settings,
    state::State,
    term::set_terminal,
};

pub async fn run() -> Result<(), Box<dyn Error>> {
    #[cfg(target_os = "linux")]
    let mpris = match Mpris::new().await {
        Ok(mpris) => Some(mpris),
        Err(err) => {
            log::warn!("Could not create MPRIS instance. Error was: {err:?}");
            None
        }
    };

    #[cfg(not(target_os = "linux"))]
    let mpris = None;

    task::spawn_blocking(move || {
        if let Err(err) = run_sync(mpris) {
            log::error!("{err:?}");
        }
    })
    .await;

    Ok(())
}

fn run_sync(mpris: Option<Mpris>) -> Result<(), Box<dyn Error>> {
    let actions = Actions::from_file_or_default();
    assert!(
        actions.contains(Action::Quit),
        "No key binding for Action::Quit! User would not be able to exit Jolteon. This is 100% a bug."
    );

    let mut terminal = set_terminal()?;

    let settings = include_str!("../assets/settings.toml");
    let settings: Settings = toml::from_str(settings).unwrap();

    let theme = include_str!("../assets/theme.toml");
    let theme: Theme = toml::from_str(theme).unwrap();

    let state = State::from_file();

    // if _output_stream is dropped playback will end & attached `OutputStreamHandle`s will no longer work.
    // Creating the output_stream indirectly spawns the cpal_alsa_out thread, and creates the mixer tied to it.
    let (_output_stream, output_stream_handle) = OutputStream::try_default()?;

    let player = Arc::new(MainPlayer::spawn(output_stream_handle, mpris, state.queue_items));
    let queue_changed = Arc::new(AtomicBool::default());

    player.on_queue_changed({
        // See src/README.md to make sense of this
        let queue_changed = queue_changed.clone();
        move || {
            queue_changed.store(true, Ordering::Release);
        }
    });

    player.on_error({
        move |error| {
            log::error!("Error reported by multi_track_player: {error}");
            // TODO: communicate to root component
        }
    });

    let focus_stolen = Arc::new(AtomicBool::default());
    let mut root_component = Root::new(&actions, settings, theme, Arc::downgrade(&player));

    root_component.on_queue_changed({
        let player = player.clone();
        move |change| {
            log::debug!("root_component.on_queue_changed {change:?}");

            match change {
                QueueChange::AddFront(song) => {
                    player.add_front(song);
                }
                QueueChange::AddBack(song) => {
                    player.add_back(song);
                }
                QueueChange::Append(songs) => {
                    player.append(&mut songs.into());
                }
                QueueChange::Remove(index) => {
                    player.remove(index);
                }
            }
        }
    });

    let tick_rate = Duration::from_millis(100);
    let mut last_tick = std::time::Instant::now();

    loop {
        if queue_changed.swap(false, Ordering::AcqRel) {
            player.queue().with_items(|songs| {
                root_component.set_queue(songs.clone().into());
            });
        }

        terminal.draw(|frame| {
            frame.render_widget(&mut root_component, frame.area());
        })?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());

        if event::poll(timeout)?
            && let Event::Key(key) = event::read()?
            && let actions = actions.action_by_key(key)
            && !actions.is_empty()
        {
            if actions.contains(&Action::Quit) {
                break;
            } else if let Some(action) = actions.iter().find_map(|action| {
                if let Action::Player(action) = action {
                    Some(action)
                } else {
                    None
                }
            }) && !focus_stolen.load(Ordering::Relaxed)
            {
                player.on_action(vec![*action]);
                player.single_track_player().on_action(vec![*action]);
            } else {
                // log::debug!("app actions {actions:?}");
                root_component.on_action(actions);
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = std::time::Instant::now();
        }
    }

    let state = State {
        last_visited_path: root_component.browser_directory().to_str().map(String::from),
        queue_items: Vec::from(player.queue().songs().clone()),
    };

    if let Err(err) = state.to_file() {
        log::error!("Could not save app state {err:?}");
    }

    log::trace!("Dropping root_component...");
    drop(root_component);
    log::trace!("root_component dropped");

    log::debug!(
        "main_player strong_count: {}. weak_count: {}",
        Arc::strong_count(&player),
        Arc::weak_count(&player)
    );
    if let Some(main_player) = Arc::into_inner(player) {
        log::debug!("main_player.quit()");
        main_player.quit();
    } else {
        log::error!("Could not gracefully quit main_player. There are some outstanding references somewhere.");
    }

    Ok(())
}
