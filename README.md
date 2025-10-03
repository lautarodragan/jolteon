# Jolteon 

The best music player.

# Table of contents
1. [Installation](#installation)
1. [Features](#features)
1. [Supported Audio Formats](#supported-audio-formats)
1. [Customization](#customization)
1. [Performance](#performance)
1. [Philosophy](#philosophy)
1. [Developing](#developing)
1. [Bugs](#Bugs)

## Installation

### Script

```
curl -s -o- https://raw.githubusercontent.com/lautarodragan/jolteon/refs/heads/main/get.sh | bash
```

### Binaries

Jolteon is available for Linux and MacOS (both Apple Silicon and Intel).

You should be able to just download the binary, `chmod +x jolteon`, and run it.

- See [releases](https://github.com/lautarodragan/jolteon/releases) for manually published releases and nightly builds.
- See the [release workflow](https://github.com/lautarodragan/jolteon/actions/workflows/cd.yml) run history for binaries built automatically for every commit to `main`.

### Cargo

```
cargo +nightly install jolteon
```

### From Source

```
git clone --depth=1 https://github.com/lautarodragan/jolteon.git
cd jolteon

cargo install --path .
```

## Features

- The number keys `1` through `5` select the different screens. The top bar shows the available screens and highlights the active one.
- `Tab` cycles through focusable elements in the screen.
- `Ctrl+Space` toggles play/pause. 
- When paused, a blinking `PAUSED` indicator is displayed in the lower-right corner on the screen. The animation can be disabled via configuration. 
- Media library
  - Search/Filter in the artist/album tree. Just press any letter or number key to start filtering. 
    Matches will be displayed in red, and, while filtering, the navigation keys will jump between matches.
    Press `Esc` to exit filtering. Pressing `Enter` to play a song will also exit filtering mode.
  - `Space` toggles expanding/collapsing (or opening/closing) the selected artist.
  - `(`, `Alt9`, and `AltC` collapses all artists.
  - `)`, `Alt0`, and `AltE` expands all artists.
  - The entire library is just one big json file. This makes it easy to back it up, and you can even use `git` to track changes to it, etc.
  - Modifications to the library are saved instantly, not when the application closes.
  - 🚧 Upcoming: automatic sorting
  - 🚧 Soon, support to search individual songs will be added. UI and UX for this feature TBD.
- Playlists
  - `Enter` will add the selected song or playlist to the queue. `Alt+Enter` will play it immediately. 
  - Deleted playlist are soft-deleted, not truly deleted. The playlist graveyard can be opened and closed with F8.
  - 🚧 The playlist graveyard cannot be focused or used at all, right now. Just opened to see it, and closed. In the future, it'll be properly
    interactive, allowing un-deleting playlists, and maybe even hard-deleting.
  - ⚠️ There's currently a bug that prevents selection of tracks in the song list in some cases. There's a cheap, temporary work-around:
    hitting enter to play the selected song seems to "fix" it temporarily.
  - 🚧 Currently, files can _only_ be added to playlists from the file browser. Support to do so from the Library is coming soon.
- File Browser
  - Explore files and folders on the left, files in the selected folder on the top-right, and details of the selected file on the 
    bottom-right.
  - Play (add to queue) music files right there in the browser, or add them to the selected playlist or library.
  - Key Bindings are shown on the screen.
  - The current directory is persisted when the application closes. You can close Jolteon, come back, and pick up where you left off. 
  - 🚧 Soon, adding a folder to the library or queue will prioritize .cue files inside the folder. Right now, cue sheet files are ignored
    when adding an entire folder, so you'll have to open the folder and work on the individual .cue file instead.
  - 🚧 Upcoming: Bookmarks
- Playing Queue
  - The queue is persisted when the application closes. If you close Jolteon with tracks in the queue, when you come back, it'll
    start playing the next automatically.
  - 🚧 In the future, the currently playing song and its position will be persisted too, so, rather than to start playing 
    the next song, it'll start playing the same song at the position it was when Jolteon was closed.
- `.cue` sheet file support
  - Metadata missing for the `.cue` file will be grabbed from the media file itself 
- `.jolt` files to override audio metadata non-destructively
  - 🚧 This works perfectly, but isn't properly documented. The format is straight forward. I's a plaintext, key-value file,
    which allows overriding the `artist`, `album` and `disc_number`. Entries in the `.jolt` file take priority over metadata 
    in media files and cue sheet.
- Controls
  - Play/Pause
  - Seek 5 seconds forward/backward
  - Media keys Play/Pause support via MPRIS in Linux
    - 🚧 There's a bug in this feature at the moment: ownership of the media keys is lost when some other application overtakes it,
      and not regained afterwards. The UX for this case is generally weird in all applications, though. I'm not sure what UX
      I'd prefer myself, even. But, at the very least, it should be regained when the overtaking application is closed (such as a Google Chrome
      tab being opened to YouTube overtaking it, but it being regained when the tab is closed).
- Help Screen
  - 🚧 This is pretty raw, right now. The goal is for Jolteon to require no guesswork, no external documentation, and to feel 100% friendly and risk-free.
    (no destructive actions, Ctrl+Z for everything, no confusing behavior, etc).
- Focus on stability
  - Application crashes are handled safely, restoring the terminal to its normal state before exiting the process.
  - Thread hygiene: all threads joined on exit - no thread is brute-force-killed by the OS on process exit.
  - Minimal use of `unwrap`. Only true bugs in the application should crash Jolteon. Any external source of indeterminism should be
    handled accordingly.
  - 🚧 In the future, if any non-bug causes an issue, rather than just being ignored, proper UX will be implemented and feedback given.
- A clock on the top bar. Can be turned off via configuration.
- Configurable key bindings
  - The Help screen has a bit of info on this, but the UX will improve in the future, like support to change the key bindings inside the application.
- 🚧 Themes
  - Currently, there's only one theme. You can find it in `assets/theme.toml`. 90% of the code allowing customization and multiple
    out-of-the-box themes is already done, so this feature is likely to come soon.
  - There will be some way to switch themes programmatically from outside the application, for themes to be switchable by external scripts.
  - Integration with OS light/dark mode will be added. Which theme is associated with each mode will be configurable, but have a sensible default.
- 🚧 Gapless playback
  - 🚧 Cue sheet tracks are handled as if they were individual files. If playing 2 consecutive tracks from a single Cue sheet,
    when track A finishes playing, Jolteon will still close the file, open it again, and seek to the starting time of track B.
    This basically defeats the gapless playback we get for free from Cue sheet files. In the future, this case will be handled specifically,
    to take advantage of it and just keep playing the same audio file.
  - 🚧 True gapless playback between different files is a different challenge. Latency is addressed by buffering two files at once
    rather than just one, and, roughly speaking, chaining the decoding iterators. We may still wind up with undesired pauses or audible artifacts
    between songs. Ideally, for gapless playback, we should always try to have a single, big audio file for the entire album, with its Cue sheet.
- 🚧 Automatic updates
  - Not fully implemented yet. The app checks for new GitHub releases when launched, and detects new versions,
    but doesn't yet download the published binary.

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

#### Album Covers

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

## Performance

I don't bench-mark Jolteon, but I do use it many hours, every day, and the release build always stays at .5-2% of my CPU, and usually 0% RAM (yes, that's a zero).
I manually compare this to `mpv` and the numbers seem to match, and my machine is 6+ years old, so I'm happy with it.
Specially considering RustRover and Chrome consume orders of magnitude more, permanently.

I haven't experienced any issues with the audio performance itself, but this is handled by symphonia and cpal, so there isn't a lot Jolteon can do to break it.
Same goes for the UI, which is managed by Ratatui.

I keep it open for days at a time — sometimes, even over a week, and haven't
seen it crash or increase memory usage.

If you do experience any sort of performance issues — be it choppy UI, keyboard input response, choppy audio, or significantly higher CPU/RAM usage than `mpv` or any other well-known media player
for the same file, please open an issue reporting it. Being able to reproduce this with an audio file available in the public domain, or with a license that permits sharing it, would be ideal,
even if hard or very unlikely.

## Philosophy

- Support features, UI and UX similar to `cmus`
- Statically linked, dependency free, single file binary that anyone can just download, `chmod a+x` and run.

### History & Rant

See [HISTORY.md](docs/HISTORY.md).

## Developing

See [DEVELOPING.md](docs/DEVELOPING.md)

## Bugs

See [BUGS.md](docs/BUGS.md).
