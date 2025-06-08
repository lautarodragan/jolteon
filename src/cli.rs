use clap::{Parser, Subcommand};

use crate::{auto_update::RELEASE_VERSION, settings::Settings};

#[derive(Parser, Debug)]
#[command(about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    PrintDefaultConfig,
    Version,
}

/// Parses cli arguments.
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
        }
        std::process::exit(0);
    }
}
