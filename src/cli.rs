use std::{borrow::Cow, io::stdout, path::PathBuf, sync::Arc, time::Duration};

use clap::{Parser, Subcommand, ValueEnum};
use crossterm::{
    event,
    event::Event,
    execute,
    queue,
    style::{Attribute, Color, Print, SetAttribute, SetForegroundColor},
    terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode},
    tty::IsTty,
};
use lofty::{
    file::TaggedFileExt,
    prelude::ItemKey,
    probe::Probe,
    tag::{ItemValue, Tag},
};

use crate::{
    actions::{Action, Actions, DEFAULT_ACTIONS_STR},
    auto_update::{CARGO_PKG_VERSION, RELEASE_VERSION_OVERRIDE},
    duration::duration_to_string,
    main_player::MainPlayer,
    settings::Settings,
    structs::Song,
};

#[derive(Parser, Debug)]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,

    #[arg(value_name = "FILE")]
    path: Option<PathBuf>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

#[derive(Subcommand, Debug)]
enum Command {
    PrintDefaultConfig,
    PrintDefaultKeyBindings,
    Version,
    About,
    Play {
        #[arg(value_name = "FILE")]
        path: PathBuf,

        #[arg(short, long, default_value_t = 0.5)]
        volume: f32,
    },
    Tags {
        #[arg(value_name = "FILE")]
        path: PathBuf,

        #[arg(value_enum, short, long, default_value_t = ColorOption::Auto)]
        color: ColorOption,

        #[arg(value_enum, short, long, default_value_t = OutputFormat::Text)]
        output: OutputFormat,
    },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum ColorOption {
    Auto,
    Always,
    Never,
}

/// Parses cli arguments. If a command is passed, this function will run it and exit the process.
///
/// This function returns no value, but it will directly exit the process if
/// either the arguments are invalid (which is done by Clap itself),
/// or if they are valid, and, by design, the TUI version of Jolteon isn't expected
/// to run after the command finishes running (such as `jolteon version`).
///
/// Jolteon uses the `aws` cli style (or `kubectl`): the first argument is always a jolteon command.
/// Commands are not prefixed with dashes (`jolteon play <file>`, not `jolteon --play <file>`).
/// This distinguishes commands from options (`jolteon play <file> --volume .2`)
pub fn cli() {
    let args = Args::parse();

    let command = match args.command {
        Some(command) => command,
        None => {
            if let Some(path) = args.path {
                Command::Play { path, volume: 0.5 }
            } else {
                return;
            }
        }
    };

    match command {
        Command::Play { path, volume } => {
            let volume = volume.clamp(0.0, 1.0);
            println!("Playing {path:?}");
            println!("Volume set to {volume}");
            println!();
            let song = match Song::from_file(&path) {
                Ok(song) => song,
                Err(err) => {
                    eprintln!("{err}");
                    std::process::exit(1);
                }
            };

            #[cfg(debug_assertions)]
            {
                println!("song {song:#?}");
                println!();
            }

            println!("Playing {title}", title = song.title);
            println!(
                "by artist {artist}",
                artist = song.artist.as_deref().unwrap_or("(missing artist name metadata)")
            );

            let song_length = song.length;
            let player = Arc::new(MainPlayer::spawn(None, vec![song]));

            player.on_error({
                move |error| {
                    log::error!("Error reported by multi_track_player: {error}");
                    eprintln!("Error reported by multi_track_player: {error}");
                }
            });

            player.on_queue_changed({
                || {
                    // println!("queue changed");
                }
            });

            player.single_track_player().set_volume(volume);

            println!();
            println!("Ctrl+C to exit");
            println!();

            let actions = Actions::from_file_or_default();
            let tick_rate = Duration::from_millis(100);
            let mut last_tick = std::time::Instant::now();

            enable_raw_mode().unwrap();

            execute!(stdout(), crossterm::cursor::Hide).unwrap();

            loop {
                let playing_position = player.playing_position();
                let time = format!(
                    "{time_played} / {current_song_length}",
                    time_played = duration_to_string(playing_position),
                    current_song_length = duration_to_string(song_length),
                );

                execute!(
                    stdout(),
                    crossterm::cursor::MoveToColumn(0),
                    Clear(ClearType::CurrentLine),
                    Print(time),
                )
                .unwrap();

                if !playing_position.is_zero() && player.playing_song().is_none() {
                    break;
                }

                let timeout = tick_rate.saturating_sub(last_tick.elapsed());

                if event::poll(timeout).unwrap()
                    && let Event::Key(key) = event::read().unwrap()
                    && let actions = actions.action_by_key(key)
                    && !actions.is_empty()
                    && actions.contains(&Action::Quit)
                {
                    break;
                }

                if last_tick.elapsed() >= tick_rate {
                    last_tick = std::time::Instant::now();
                }
            }

            execute!(
                stdout(),
                crossterm::cursor::MoveToNextLine(5),
                Print("\n"),
                Print("Bye")
            )
            .unwrap();

            disable_raw_mode().unwrap_or_else(|e| {
                log::error!("tried to disable_raw_mode but couldn't :( {e}");
            });
        }
        Command::PrintDefaultConfig => {
            println!("# default {} configuration:", env!("CARGO_PKG_NAME"));
            println!("{}", Settings::default());
        }
        Command::PrintDefaultKeyBindings => {
            println!("# default {} key bindings:", env!("CARGO_PKG_NAME"));
            println!("{DEFAULT_ACTIONS_STR}");
        }
        Command::Version => {
            println!("Jolteon {}", RELEASE_VERSION_OVERRIDE.unwrap_or(CARGO_PKG_VERSION));
        }
        Command::About => {
            println!("{}", env!("CARGO_PKG_NAME"));
            println!("{}", env!("CARGO_PKG_DESCRIPTION"));
            println!("{}", env!("CARGO_PKG_REPOSITORY"));
        }
        Command::Tags { path, color, output } => {
            let color = output == OutputFormat::Text
                && (color == ColorOption::Always || (color == ColorOption::Auto && stdout().is_tty()));

            macro_rules! styled {
                    ($text:expr $(, $command:expr)* $(,)?) => {{
                        if color {
                            queue!(stdout() $(, $command)*).unwrap();
                        }

                        queue!(stdout(), Print($text)).unwrap();

                        if color {
                            queue!(stdout(), SetAttribute(Attribute::Reset)).unwrap();
                        }
                    }}
                }

            styled!("Tags", SetForegroundColor(Color::Green), SetAttribute(Attribute::Bold));
            println!(" in {path:?}:");
            println!();

            let tagged_file = Probe::open(path).unwrap().read().unwrap();
            let tags = tagged_file.tags();

            fn tag_value_to_string(value: &ItemValue) -> String {
                match value {
                    ItemValue::Text(s) => s.to_string(),
                    ItemValue::Locator(l) => format!("locator: {l}"),
                    ItemValue::Binary(b) => {
                        if b.len() > 8 {
                            format!("binary: {:?}... (total length: {})", b.iter().take(8), b.len())
                        } else {
                            format!("binary: {b:?}")
                        }
                    }
                }
            }

            let print_tag = |longest_key: usize| {
                move |(key, value): (Cow<str>, &ItemValue)| {
                    let key = format!("  {key: <padding$}", padding = longest_key + 2);
                    styled!(key, SetForegroundColor(Color::Green), SetAttribute(Attribute::Bold));
                    let value = tag_value_to_string(value);
                    println!("    {value:?}");
                }
            };

            let print_tag_items = |tag: &Tag, known: bool| {
                let tags = tag.items().filter_map(|item| match item.key() {
                    ItemKey::Unknown(_) if known => None,
                    key if known => Some((Cow::from(format!("{key:?}")), item.value())),
                    ItemKey::Unknown(key) if !known => Some((Cow::from(key), item.value())),
                    _ => None,
                });

                if let Some(longest_key) = tags.clone().map(|(k, _)| k.len()).max() {
                    let word = if known { "Standard" } else { "Unknown" };
                    println!("{word} {tag_type:?} tags:", tag_type = tag.tag_type());
                    tags.for_each(print_tag(longest_key));
                    println!();
                }
            };

            for tag in tags {
                print_tag_items(tag, true);
                print_tag_items(tag, false);
            }
        }
    }

    execute!(stdout(), SetAttribute(Attribute::Reset), crossterm::cursor::Show,).unwrap();
    println!();

    std::process::exit(0);
}
