# Jolteon 

The best music player for local files.

![File Browser - Light Theme](assets/file_browser_light.png?raw=true)
![File Browser - Dark Theme](assets/file_browser_dark.png?raw=true)
![File Browser - Search](assets/file_browser_search.png?raw=true)

# Table of contents
1. [About](#about)
1. [Installation](#installation)
1. [Features](#features)
1. [Supported Audio Formats](#supported-audio-formats)
1. [Customization](#customization)
1. [Performance](#performance)
1. [Philosophy](#philosophy)
1. [Contributing](#contributing)
1. [Bugs](#Bugs)

## About 

Jolteon, the best music player for local files.

Warning: this project is under heavy development.

## Installation

*Tested only on Linux*

### Build From Source

Right now, this is the only option.

You'll need rust nightly.

```
git clone --depth=1 https://github.com/lautarodragan/jolteon.git
cd jolteon
cargo run --release
```

### Upcoming

- Binaries automatically built and published here, in the GitHub repo
- Automatic updates

## Features

- Media library
  - Search/Filter in the artist list. Just start typing. Press `Esc` to clear the filter.
  - Soon, full-library search will be added. The only challenge here is coming up with great UI and UX for this feature — the code itself is simple and most of what's needed is already implemented.
- Playlists
- File Browser
  - Search/Filter in File Browser (Ctrl+F)
- Playing Queue
- `.cue` sheet file support
- Customizable color themes
- Controls
  - Play/Pause
  - Seek 5 seconds forward/backward
  - Media keys Play/Pause support via MPRIS in Linux
- Gapless playback
- Persist app state:
  - The current directory of the browser
  - The queue
  - Current song (coming soon)
- Safe handling of application crashes, restoring the terminal to its normal state, thread hygiene (all threads joined on exit) 
- A clock on the top bar :)

### Upcoming

#### Virtual directories in media library.

Displaying music by artist, album, track number and song title is generally more desirable than navigating the file system, but, sometimes, being able
to manually structure and organize music beyond its metadata is convenient.

Personally, I prefer having all soundtracks under an `OST` folder, rather than mixed with bands. Same goes for "classic" music, or _interpreted_ music in general (as opposed to original compositions).

There's no real reason not to support both approaches at once, by organizing music files by metadata but allowing grouping by _virtual directories_, which would enable things like:
- _Interpreted_
  - Bach
- _Modern_
  - The Doors
  - Pink Floyd
    - The Piper at the Gates of Dawn 
    - The Dark Side of the Moon
    - Wish You Were Here
- _Soundtracks_
  - _Cowboy Bebop_
  - _Ry Cooder - Crossroads_

#### Media metadata overrides

Have you ever experienced... this?:
- `Dark Side of the Moon` by `Pink Floyd` 
- `The Piper at the Gates of Dawn` by `PINK FLOYD`

Even though those are two albums by the same artist, they'll usually show as two different artists in media libraries, because of the different spelling.

Some cases are more complicated, particularly with soundtracks and classic music, in which cases you may want to organize albums under "Johann Sebastian Bach" rather than the names of the interpreters.

Modifying the files themselves is 100% not an acceptable solution to this issue. 

Jolteon will offer an option to "override" the metadata of these files, either by storing a `.jolt` file along the song files in the same folder, or directly in the UI.

This will also help in cases where a media tag or cue sheet entry may have invalid data (such as non-utf encoding), which Jolteon will usually default to an empty string.

### Album Covers

Kitty supports full-resolution, full-color images. It shouldn't be particularly hard to add this feature.

I'll have to figure out the best UI and UX for this, and probably make it optional/configurable.

## Supported Audio Formats

The following formats should work: `aac`, `flac`, `mp3`, `mp4`, `m4a`, `ogg`, `wav`.

Jolteon uses Rodeo to play music, with the symphonia backend. 

I mainly use `flac` files, and some `mp3`. Other formats aren't usually tested.

### Codec Issues

So far, I've only found _one_ issue with _one_ flac file, which fails to perform seeks, and, after a few seconds of playback, causes the cpal thread to panic, crashing Jolteon. 
This same file does run well with `mpv`. It does report errors reading it, but it still recovers well from them, and is able to seek without issues.

I tried switching the flac backend, but got even worse results. I looked into using [libmpv](https://github.com/mpv-player/mpv-examples/tree/master/libmpv) and [libavcodec](https://www.ffmpeg.org/libavcodec.html), which, in my mind, are pretty much guaranteed to be more stable, but switching to them is far from trivial.

Figuring out the specific bug in the flac codes built in pure Rust is probably an easier and more reasonable path forward.

And Jolteon shouldn't crash if the audio playback crashes, but that's a story for another day.

## Customization

### Theme

Right now, the theme is built into the binary (see assets/theme.toml).
Soon, more built-in themes will be added, as well as support for custom ones.

### Future Configuration Options

There aren't any other configuration options available right now, but I have a few in mind:
- Keyboard shortcuts
- Texts (translations)
- _Maybe_ some degree UI layout. 


## Performance

I don't bench-mark Jolteon, but I do use it many hours, every day, and the release build always stays at .5-2% of my CPU, and usually 0% RAM.
I manually compare this to `mpv` and the numbers seem to match, and my machine is 6+ years old, so I'm happy with it.
Specially considering RustRover and Chrome consume orders of magnitude more, permanently.

I haven't experienced any issues with the audio performance itself, but this is handled by symphonia and cpal, so there isn't a lot Jolteon can do to break it.
Same goes for the UI, which is managed by Ratatui.

If you do experience any sort of performance issues — be it choppy UI, keyboard input response, choppy audio, or significantly higher CPU/RAM usage than `mpv` or any other well-known media player
for the same file, please open an issue reporting it. Being able to reproduce this with an audio file available in the public domain, or with a license that permits sharing it, would be ideal,
even if hard or very unlikely.


## Philosophy

- Support features, UI and UX similar to `cmus`
- Statically linked, dependency free, single file binary that anyone can just download, `chmod a+x` and run.

### History & Rant

See [HISTORY.md](./HISTORY.md).

## Contributing

In general, I won't accept contributions, because I don't have enough time or Rust knowledge to properly do code review. 

You're free to have your own fork of Jolteon, though. Even if you're new to Rust, it's very friendly language with a very friendly community; and I try to keep the source code as clean and intuitive as I can, so modifying it should be relatively easy. 

To install Rust and Cargo, see https://www.rust-lang.org/tools/install or https://doc.rust-lang.org/cargo/getting-started/installation.html. Under macOS, Linux, etc, it's just copy-pasting a `curl ...` command. I won't copy it here for safety reasons.

Then, just clone the repo:

```
git clone --depth=1 https://github.com/lautarodragan/jolteon.git
cd jolteon
```

And `cargo run`, `cargo run --release` and `cargo build` are the commands you'll be running the most.

Keep in mind I'm using my own fork of `cpal` right now. I have an open [PR for cpal](https://github.com/RustAudio/cpal/pull/909), with a small bugfix, that hasn't been merged yet. 

## Bugs

See [BUGS.md](./BUGS.md).
