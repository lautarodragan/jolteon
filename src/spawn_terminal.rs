use std::{
    path::PathBuf,
    thread,
    io::BufRead,
};

#[allow(dead_code)]
fn spawn_terminal(cwd: PathBuf) {
    if let Err(err) = thread::Builder::new().name("term".to_string()).spawn(move || {
        log::debug!("spawning child process");

        let proc = std::process::Command::new("kitty")
            .current_dir(cwd)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn();

        if let Ok(mut proc) = proc {
            log::debug!("spawned child process");

            let stdout = proc.stdout.as_mut().unwrap();
            let stdout_reader = std::io::BufReader::new(stdout);

            for line in stdout_reader.lines() {
                log::debug!("stdout: {:?}", line);
            }

            log::debug!("child process exited");
        } else if let Err(err) = proc {
            log::error!("error spawning thread {:?}", err);
        }
    }) {
        log::error!("Error spawning thread! {:?}", err);
    }
}
