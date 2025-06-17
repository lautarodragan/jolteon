use std::{borrow::Cow, io::Write, path::PathBuf, sync::Arc, thread, time::Duration};

use clap::{Parser, Subcommand, ValueEnum};
use crossterm::{
    execute,
    queue,
    style::{Attribute, Color, Print, SetAttribute, SetForegroundColor},
    terminal::{Clear, ClearType},
};
use lofty::{
    file::TaggedFileExt,
    prelude::ItemKey,
    probe::Probe,
    tag::{ItemValue, Tag},
};
use rodio::OutputStream;

use crate::{
    auto_update::RELEASE_VERSION,
    duration::duration_to_string,
    main_player::MainPlayer,
    settings::Settings,
    structs::Song,
};

#[derive(Parser, Debug)]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    PrintDefaultConfig,
    Version,
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

    if let Some(command) = args.command {
        match command {
            Command::PrintDefaultConfig => {
                println!("# default Jolteon configuration:");
                println!("{}", Settings::default());
            }
            Command::Version => {
                if let Some(version) = RELEASE_VERSION {
                    println!("Jolteon {version}");
                } else {
                    println!(
                        "Version unknown. This is an error. Make sure JOLTEON_RELEASE_VERSION is set at compile time."
                    );
                }
            }
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

                let (_output_stream, output_stream_handle) = OutputStream::try_default().unwrap();
                let song_length = song.length;
                let player = Arc::new(MainPlayer::spawn(output_stream_handle, None, vec![song]));

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

                loop {
                    let playing_position = player.playing_position();

                    execute!(std::io::stdout(), Clear(ClearType::CurrentLine)).unwrap();
                    print!(
                        "{time_played} / {current_song_length}\r",
                        time_played = duration_to_string(playing_position),
                        current_song_length = duration_to_string(song_length),
                    );
                    std::io::stdout().flush().unwrap();

                    if !playing_position.is_zero() && player.playing_song().is_none() {
                        println!();
                        break;
                    }

                    thread::sleep(Duration::from_secs(1));
                }
            }
            Command::Tags { path, color } => {
                let color = color == ColorOption::Always || color == ColorOption::Auto; // TODO: Color == Auto && Settings.CliColor = Always
                macro_rules! styled {
                    ($text:expr $(, $command:expr)* $(,)?) => {{
                        if color {
                            queue!(std::io::stdout() $(, $command)*).unwrap();
                        }

                        queue!(std::io::stdout(), Print($text)).unwrap();

                        if color {
                            queue!(std::io::stdout(), SetAttribute(Attribute::Reset)).unwrap();
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
                        println!("Standard {tag_type:?} tags:", tag_type = tag.tag_type());
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
        execute!(std::io::stdout(), SetAttribute(Attribute::Reset),).unwrap();
        println!();
        println!("Bye :)");
        std::process::exit(0);
    }
}
