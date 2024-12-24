#![allow(clippy::field_reassign_with_default)]
#![warn(clippy::uninlined_format_args)]
#![warn(clippy::uninlined_format_args)]
#![warn(clippy::string_add_assign)]
#![warn(clippy::string_add_assign)]
#![warn(clippy::ref_option_ref)]
#![warn(clippy::option_as_ref_cloned)]
#![warn(clippy::option_as_ref_cloned)]
#![warn(clippy::assigning_clones)]
#![warn(clippy::inefficient_to_string)]
#![allow(clippy::enum_variant_names)]
extern crate alloc;

mod app;
mod auto_update;
mod bye;
mod components;
mod config;
mod constants;
mod cue;
mod duration;
mod extensions;
mod files;
mod mpris;
mod player;
mod source;
mod spawn_terminal;
mod state;
mod structs;
mod term;
mod toml;
mod ui;
mod main_player;

use std::error::Error;
use std::io::stdout;
use std::thread;

use async_std::task;
use colored::{Color, Colorize};
use flexi_logger::{style, DeferredNow, FileSpec, Logger, WriteMode};
use log::{debug, info, Record};

use crate::{app::App, auto_update::auto_update, bye::bye, mpris::Mpris, term::reset_terminal};

pub fn log_format(w: &mut dyn std::io::Write, now: &mut DeferredNow, record: &Record) -> Result<(), std::io::Error> {
    write!(w, "{}   ", now.format("%-l:%M:%S%P"))?;

    let level = format!("{: <8}", record.level());
    write!(w, "{}", style(record.level()).paint(level))?;

    write!(w, "{: <16}", thread::current().name().unwrap_or("<unnamed>"),)?;

    let target = record.target().to_string();

    let color = if target.starts_with("jolteon") {
        Color::Green
    } else if target.starts_with("::") {
        Color::Blue
    } else {
        Color::Black
    };

    write!(w, "{:28}", target[..target.len().min(25)].color(color))?;

    write!(w, "{}", record.args())?;
    Ok(())
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    set_panic_hook();

    let _logger = Logger::try_with_str("jolteon=trace,::=trace, warn")?
        .format(log_format)
        .log_to_file(FileSpec::default().suppress_timestamp())
        .write_mode(WriteMode::Direct)
        .use_utc()
        .start()?;

    info!("Starting");

    let _auto_update = auto_update().await;

    debug!("Starting mpris and player");

    let mpris = Mpris::new().await?;

    let task_player = task::spawn_blocking(move || {
        let mut app = App::new(mpris);
        app.start()
            .unwrap_or_else(|err| log::error!("app.start error :( \n{:#?}", err));
        log::trace!("Player.start() finished");
    });

    debug!("Awaiting player task");
    task_player.await;
    debug!("Quitting Jolteon");

    debug!("Resetting terminal");
    reset_terminal(&mut stdout());

    info!("{}", bye());
    Ok(())
}

fn set_panic_hook() {
    debug!("set_panic_hook");
    let original_hook = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |panic_info| {
        // intentionally ignore errors here since we're already in a panic
        reset_terminal(&mut stdout());
        original_hook(panic_info);
    }));
}
