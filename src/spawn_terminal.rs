use std::{io::BufRead, path::PathBuf, thread};

pub fn spawn_terminal(cwd: PathBuf) {
    thread::Builder::new()
        .name("term".to_string())
        .spawn(move || {
            log::debug!("spawning child process");

            let proc = std::process::Command::new("kitty")
                .current_dir(cwd)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn();

            if let Ok(mut proc) = proc {
                log::debug!("spawned child process");


                // TODO: this keeps Jolteon's process open, even after closing jolteon,
                //  until this child process ends.
                //  Not good UX, and there's no real need to do this, other than debugging.
                //  Maybe make it configurable, or skip in release compilation.
                let stdout = proc.stdout.as_mut().unwrap();
                let stdout_reader = std::io::BufReader::new(stdout);

                for line in stdout_reader.lines() {
                    log::debug!("stdout: {:?}", line);
                }

                log::debug!("child process exited");
            } else if let Err(err) = proc {
                log::error!("error spawning thread {:?}", err);
            }
        })
        .unwrap();
}
