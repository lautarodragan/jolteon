#![cfg(target_os = "linux")]
#![forbid(unsafe_code)]

use std::{
    fs::{self, File},
    io,
    path::Path,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

const RATE: usize = 48_000;
const MAX_CAPTURE_BYTES: u64 = 8 * 1024 * 1024;

#[test]
#[ignore = "requires Linux ALSA, ffmpeg, ffprobe and timeout; run with --ignored"]
fn audio_cli_playback() {
    with_artifacts(|work| {
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tone-440hz.wav");
        check_signal(&capture(work, &fixture)?)
    });
}

#[test]
#[ignore = "requires Linux ALSA, ffmpeg, ffprobe and timeout; run with --ignored"]
fn audio_cli_preserves_short_silence() {
    with_artifacts(|work| {
        run_tool(work, "ffmpeg", "fixture", 20, &[
            "-v",
            "error",
            "-nostdin",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:sample_rate=48000:duration=2[a];anullsrc=r=48000:cl=mono:d=1[b];sine=frequency=880:sample_rate=48000:duration=2[c];[a][b][c]concat=n=3:v=0:a=1",
            "-c:a",
            "pcm_s16le",
            "gap.wav",
        ])?;
        let bytes = capture(work, &work.join("gap.wav"))?;
        if bytes.len() % 4 != 0 {
            return Err(io::Error::other("incomplete decoded PCM sample"));
        }
        let samples: Vec<f64> = bytes
            .chunks_exact(4)
            .map(|b| f32::from_le_bytes(b.try_into().unwrap()) as f64)
            .collect();
        let duration = samples.len() as f64 / RATE as f64;
        if !(4.8..=8.25).contains(&duration) || !samples.iter().all(|s| s.is_finite()) {
            return Err(io::Error::other(format!("invalid gap capture: {duration:.3}s")));
        }
        // Interior windows tolerate filter edges but detect a removed/shortened gap.
        for (start, end, hz) in [(0.2, 1.8, 440.0), (2.1, 2.8, 0.0), (3.2, 4.8, 880.0)] {
            let window = &samples[(start * RATE as f64) as usize..(end * RATE as f64) as usize];
            let rms = (window.iter().map(|s| s * s).sum::<f64>() / window.len() as f64).sqrt();
            let frequency = window.windows(2).filter(|s| s[0] <= 0.0 && s[1] > 0.0).count() as f64 / (end - start);
            if (hz == 0.0 && rms > 0.0001) || (hz != 0.0 && (rms <= 0.01 || (frequency - hz).abs() > 2.0)) {
                return Err(io::Error::other(format!(
                    "unexpected {start}–{end}s window: RMS={rms:.5}, frequency={frequency:.1} Hz"
                )));
            }
        }
        eprintln!("Preserved 440 Hz -> 1s silence -> 880 Hz ({duration:.3}s capture)");
        Ok(())
    });
}

fn with_artifacts(test: impl FnOnce(&Path) -> io::Result<()>) {
    // A fixed parent and UUID make the generated ALSA path safe to quote.
    let work = Path::new("/tmp").join(format!("jolteon-audio-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&work).unwrap();
    let result = test(&work);
    if result.is_err() || std::env::var_os("JOLTEON_AUDIO_KEEP").is_some() {
        eprintln!("Audio test artifacts: {}", work.display());
    } else {
        fs::remove_dir_all(&work).unwrap();
    }
    result.unwrap();
}

fn capture(work: &Path, fixture: &Path) -> io::Result<Vec<u8>> {
    fs::copy(fixture, work.join("tone.wav"))?;
    // Regression guard: loading host/user ALSA hooks must never replace our PCM.
    // This isolated user config deliberately names an unavailable plugin.
    let user_config = work.join("xdg/alsa");
    fs::create_dir_all(&user_config)?;
    fs::write(
        user_config.join("asoundrc"),
        "pcm.!default { type jolteon_test_override_must_not_be_loaded }\n",
    )?;
    // ALSA owns the anonymous pipe. FFmpeg drops startup silence, then stops
    // emitting after 3 consecutive seconds of silence, but drains input to EOF.
    // The shell publishes a status only after FFmpeg exits and finalizes its WAV.
    // All shell text is fixed; paths/fixture names are not interpolated into it.
    fs::write(
        work.join("alsa.conf"),
        format!(
            // Do not include alsa.conf: its lazy hooks can override default with
            // PipeWire, PulseAudio, or ~/.asoundrc after this file is parsed.
            r#"pcm.!default {{
    type plug
    slave {{
        pcm {{
            type file
            slave.pcm {{ type null }}
            file "|timeout --kill-after=1s 15s ffmpeg -v error -nostdin -f s16le -ar 48000 -ac 2 -i pipe:0 -af silenceremove=start_periods=1:start_threshold=0.0001:stop_periods=1:stop_duration=3:stop_threshold=0.0001:detection=peak -fs {MAX_CAPTURE_BYTES} -c:a pcm_s16le capture.wav >capture.stdout 2>capture.stderr; echo $? >capture.status.tmp; mv capture.status.tmp capture.status"
            format "raw"
        }}
        format S16_LE
        rate {RATE}
        channels 2
    }}
}}
"#,
        ),
    )?;
    run_tool(work, env!("CARGO_BIN_EXE_jolteon"), "playback", 10, &[
        "play", "tone.wav", "--volume", "1.0",
    ])?;
    wait_for_capture(work)?;
    let output = fs::read_to_string(work.join("playback.stdout"))?;
    if output.contains('\x1b') || !output.contains("Bye") {
        return Err(io::Error::other(
            "non-interactive playback did not finish with plain-text output",
        ));
    }
    let size = fs::metadata(work.join("capture.wav"))?.len();
    if size == 0 || size >= MAX_CAPTURE_BYTES {
        return Err(io::Error::other("capture is empty or reached the size limit"));
    }
    run_tool(work, "ffprobe", "probe", 20, &[
        "-v",
        "error",
        "-show_streams",
        "-show_format",
        "-of",
        "json",
        "capture.wav",
    ])?;
    let metadata = fs::read(work.join("probe.stdout"))?;
    serde_json::from_slice::<serde_json::Value>(&metadata)?;
    fs::rename(work.join("probe.stdout"), work.join("probe.json"))?;
    run_tool(work, "ffmpeg", "decode", 20, &[
        "-v",
        "error",
        "-nostdin",
        "-i",
        "capture.wav",
        "-f",
        "f32le",
        "-ac",
        "1",
        "-ar",
        "48000",
        "pipe:1",
    ])?;
    fs::read(work.join("decode.stdout"))
}

fn wait_for_capture(work: &Path) -> io::Result<()> {
    // The shell opens this diagnostic file before starting FFmpeg. Successful
    // playback with no such file means the test's capture PCM was never used.
    if !work.join("capture.stderr").try_exists()? {
        return Err(io::Error::other(
            "ALSA capture command never started; playback did not use the test PCM (see alsa.conf and playback.stderr)",
        ));
    }
    let deadline = Instant::now() + Duration::from_secs(17);
    loop {
        match fs::read_to_string(work.join("capture.status")) {
            Ok(status) if status.trim() == "0" => return Ok(()),
            Ok(status) => {
                return Err(io::Error::other(format!(
                    "audio capture exited with {}; see capture.stderr",
                    status.trim()
                )));
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
        if Instant::now() >= deadline {
            return Err(io::Error::other("audio capture did not report completion within 17s"));
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn check_signal(bytes: &[u8]) -> io::Result<()> {
    if bytes.is_empty() || bytes.len() % 4 != 0 {
        return Err(io::Error::other("decoded capture is empty or incomplete"));
    }
    let mut energy = 0.0;
    let mut active = 0;
    for bytes in bytes.chunks_exact(4) {
        let sample = f32::from_le_bytes(bytes.try_into().unwrap()) as f64;
        if !sample.is_finite() {
            return Err(io::Error::other("decoded capture contains a non-finite sample"));
        }
        energy += sample * sample;
        active += usize::from(sample.abs() > 0.01);
    }
    let samples = bytes.len() / 4;
    let duration = samples as f64 / RATE as f64;
    let rms = (energy / samples as f64).sqrt();
    let active = active as f64 / RATE as f64;
    // Two seconds of tone, up to three of trailing silence, and filter tolerance.
    if !(1.5..=5.25).contains(&duration) || rms <= 0.01 || active < 1.0 {
        return Err(io::Error::other(format!(
            "unexpected signal: duration={duration:.3}s, RMS={rms:.5}, active={active:.3}s"
        )));
    }
    eprintln!("CLI playback: {duration:.3}s captured, RMS={rms:.5}, {active:.3}s non-silent samples");
    Ok(())
}

fn run_tool(work: &Path, tool: &str, label: &str, seconds: u32, args: &[&str]) -> io::Result<()> {
    // Coreutils timeout bounds each command and terminates it on timeout.
    // ALSA's capture command has its own deadline in case it outlives Jolteon.
    let status = Command::new("timeout")
        .args(["--kill-after=1s", &format!("{seconds}s"), tool])
        .args(args)
        .current_dir(work)
        .env("ALSA_CONFIG_PATH", work.join("alsa.conf"))
        .env("XDG_CONFIG_HOME", work.join("xdg"))
        .stdin(Stdio::null())
        .stdout(File::create(work.join(format!("{label}.stdout")))?)
        .stderr(File::create(work.join(format!("{label}.stderr")))?)
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "{tool} exited with {status}; see {label}.stderr"
        )))
    }
}
