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

    let queue = Arc::new(Queue::new(state.queue_items));
    let main_player = Arc::new(MainPlayer::spawn(output_stream_handle, queue.clone(), mpris));

    let mut root_component = Root::new(theme, queue.clone(), Arc::downgrade(&main_player));

    root_component.on_queue_changed(|songs| {
        log::debug!("root_component.on_queue_changed {songs:?}");
    });

    let tick_rate = Duration::from_millis(100);
    let mut last_tick = std::time::Instant::now();

    loop {
        terminal.draw(|frame| {
            frame.render_widget_ref(&root_component, frame.area());
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
                            main_player.single_track_player().on_action(action);
                        }
                        Action::MainPlayer(action) => {
                            main_player.on_action(action);
                        }
                        Action::Screen(action) => {
                            root_component.on_action(action);
                        }
                        _ => {
                            root_component.on_key(key);
                        }
                    }
                } else {
                    root_component.on_key(key);
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = std::time::Instant::now();
        }
    }

    let state = State {
        last_visited_path: root_component.browser_directory().to_str().map(String::from),
        queue_items: Vec::from(queue.songs().clone()),
    };

    if let Err(err) = state.to_file() {
        log::error!("Could not save app state {err:?}");
    }

    drop(root_component);

    log::debug!(
        "main_player strong_count: {}. weak_count: {}",
        Arc::strong_count(&main_player),
        Arc::weak_count(&main_player)
    );
    if let Some(main_player) = Arc::into_inner(main_player) {
        log::debug!("main_player.quit()");
        main_player.quit();
    } else {
        log::error!("Could not gracefully quit main_player. There are some outstanding references somewhere.");
    }

    Ok(())
}
