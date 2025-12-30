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

### Binaries

Jolteon is available for Linux and MacOS (both Apple Silicon and Intel).

You should be able to just download the binary, `chmod +x jolteon`, and run it.

Run the following script to download the latest release for your platform:

```
curl -s -o- https://raw.githubusercontent.com/lautarodragan/jolteon/refs/heads/main/get.sh | bash
```

Or…
- See [releases](https://github.com/lautarodragan/jolteon/releases) for published releases and nightly builds.
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

On Linux, you may need to install the ALSA lib dev package.

```
# Arch
sudo pacman -S alsa-lib

# Ubuntu
sudo apt-get install libasound2-dev
```

## Features

These are the main actions and their default key bindings:

| Key                                         | Action                                             |
|---------------------------------------------|----------------------------------------------------|
| <kbd>Tab</kbd>                              | Cycles through focusable elements in the screen.   |
| <kbd>1</kbd>                                | Select Library                                     |
| <kbd>2</kbd>                                | Select Playlists                                   |
| <kbd>3</kbd>                                | Select Queue                                       |
| <kbd>4</kbd>                                | Select FileBrowser                                 |
| <kbd>5</kbd>                                | Select Help                                        |
| <kbd>Ctrl</kbd> + <kbd>Space</kbd>          | Toggle play/pause                                  |
| <kbd>+</kbd>                                | Increase volume                                    |
| <kbd>-</kbd>                                | Decrease volume                                    |
| <kbd>Ctrl</kbd> + <kbd>Left</kbd>           | Seek backwards 5s                                  |
| <kbd>Ctrl</kbd> + <kbd>Right</kbd>          | Seek forwards 5s                                   |
| <kbd>Alt</kbd> + <kbd>Q</kbd>               | Repeat: None (Turn Off)                            |
| <kbd>Alt</kbd> + <kbd>W</kbd>               | Repeat: One Song                                   |
| <kbd>Alt</kbd> + <kbd>E</kbd>               | Repeat: Entire Queue                               |
| <kbd>Alt</kbd> + <kbd>R</kbd>               | Repeat: Toggle                                     |
|                                             |                                                    |
|                                             | **Library Screen**                                 |
| Any letter key                              | Search                                             |
| <kbd>↓</kbd>                                | While Searching: Select Next Result                |
| <kbd>↑</kbd>                                | While Searching: Select Previous Result            |
| <kbd>Esc</kbd>                              | While Searching: Exit search mode                  |
| <kbd>Enter</kbd>                            | While Searching: Exit search mode + play selection |
| <kbd>Space</kbd>                            | Collapse or expand selected artist                 |
| <kbd>(</kbd>, <kbd>Alt</kbd> + <kbd>9</kbd> | Collapse all artists                               |
| <kbd>)</kbd>, <kbd>Alt</kbd> + <kbd>0</kbd> | Expand all artists                                 |
|                                             |                                                    |
|                                             | **Playlist Screen**                                |
| <kbd>F8</kbd>                               | Open Playlist Graveyard                            |
| <kbd>Enter</kbd>                            | Add Selected Song / Playlist to Queue              |
| <kbd>Alt</kbd> + <kbd>Enter</kbd>           | Play Selected Song / Playlist                      |

The key bindings are configurable.

To see the full list of default key bindings, you can run `jolteon print-default-key-bindings`
or see [actions.ini](assets/actions.ini).

Custom key bindings are read from `~/.config/jolteon/actions.ini`.
You can `jolteon print-default-key-bindings > ~/.config/jolteon/actions.ini` to get started.

Inside Jolteon, the Help screen shows the active key bindings (default + custom overrides).

> [!TIP]
> Some Jolteon key bindings may conflict with the terminal's ones.
> If you want every key binding available to Jolteon, Kitty supports `clear_all_shortcuts`:
> `kitty -o "clear_all_shortcuts yes" jolteon`

### Configuration Options

| Setting             | Type                                      | Default       | Description                                              |
|---------------------|-------------------------------------------|---------------|----------------------------------------------------------|
| clock_display       | boolean                                   | true          | Debugging option. Displays a frame counter on the screen |
| paused_animation    | boolean                                   | true          | Debugging option. Displays a frame counter on the screen |
| theme               | "GruvboxDark" or "GruvboxDarkTransparent" | "GruvboxDark" | Choose one of the two built-in themes                    |
| debug_frame_counter | boolean                                   | false         | Debugging option. Displays a frame counter on the screen |

Note about `theme`: `GruvboxDarkTransparent` is literally `GruvboxDark` with a transparent background.
This is particularly useful if you set the opacity of the terminal to anything other than fully opaque,
since it'll allow seeing your wallpaper behind Jolteon.

In Hyprland and Kitty, try the following:

```
# Hyprland
decoration {
  blur {
    enabled = true
    size = 3
    passes = 1
    vibrancy = 0.1696
  }
}

# Kitty
background_opacity 0.6
map f5 set_background_opacity -0.1
map f6 set_background_opacity +0.1
```

### Other Features

Status:
- When paused, a blinking `PAUSED` indicator is displayed in the lower-right corner on the screen. The animation can be disabled via configuration. 
- A clock on the top bar. Can be turned off via configuration.

- Media library
  - The entire library is just one big json file. This makes it easy to back it up, and you can even use `git` to track changes to it, etc.
  - Modifications to the library are saved instantly, not when the application closes.
  - 🚧 Upcoming: automatic sorting
  - 🚧 Soon, support to search individual songs will be added. UI and UX for this feature TBD.
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
- `.cue` sheet file support
  - Metadata missing for the `.cue` file will be read from the media file 
- `.jolt` files to override audio metadata non-destructively
  - The format is straight forward. I's a plaintext, key-value file,
    which allows overriding the `artist`, `album` and `disc_number`. Entries in the `.jolt` file take priority over metadata 
    in media files and cue sheet.
- Media keys Play/Pause support via MPRIS in Linux
- Focus on stability
  - Application crashes are handled safely, restoring the terminal to its normal state before exiting the process.
  - Thread hygiene: all threads joined on exit - no thread is brute-force-killed by the OS on process exit.
  - Minimal use of `unwrap`. Only true bugs in the application should crash Jolteon. Any external source of indeterminism should be
    handled accordingly.

### Upcoming

<details>
<summary><strong>Playlist from Library</strong></summary>

Currently, files can _only_ be added to playlists from the file browser. Support to do so from the Library is coming soon.

</details>

<details>
<summary><strong>Communication, Feedback & Notifications</strong></summary>

If any non-bug causes an issue, rather than just being ignored, proper UX wunication, Feedback & Noill be implemented and feedback given.
For example: a file that cannot be played correctly, for whatever reason.

</details>

<details>
<summary><strong>Theme Customization</strong></summary>

Status: partially implemented; almost complete.

- There will be some way to switch themes programmatically from outside the application, for themes to be switchable by external scripts.
- Integration with OS light/dark mode will be added. Which theme is associated with each mode will be configurable, but have a sensible default.
- See `assets/gruvbox_dark.toml`

</details>

<details>
<summary><strong>Playlist Graveyard</strong></summary>

Status: partially implemented

- Deleted playlist are soft-deleted, not truly deleted. 
- The playlist graveyard can be opened and closed with <kbd>F8</kbd>.
- The playlist graveyard cannot be focused or used at all, right now. Just opened to see it, and closed. In the future, it'll be properly
  interactive, allowing un-deleting playlists, and maybe hard-deleting.

</details>

<details>
<summary><strong>Persist Currently Playing Track</strong></summary>

The currently playing song and its position will be persisted too, so, rather than to start playing 
the next song, it'll start playing the same song at the position it was when Jolteon was closed.

</details>

<details>
<summary><strong>Gapless playback</strong></summary>

Cue sheet tracks are handled as if they were individual files. If playing 2 consecutive tracks from a single Cue sheet,
when track A finishes playing, Jolteon will still close the file, open it again, and seek to the starting time of track B.
This basically defeats the gapless playback we get for free from Cue sheet files. In the future, this case will be handled specifically,
to take advantage of it and just keep playing the same audio file.

True gapless playback between different files is a different challenge. Latency is addressed by buffering two files at once
rather than just one, and, roughly speaking, chaining the decoding iterators. We may still wind up with undesired pauses or audible artifacts
between songs. Ideally, for gapless playback, we should always try to have a single, big audio file for the entire album, with its Cue sheet.

</details>

<details>
<summary><strong>Automatic updates</strong></summary>

Partially implemented. The app checks for new GitHub releases when launched, and detects new versions,
but doesn't yet download the published binary.

</details>

<details>
<summary><strong>Virtual directories in media library</strong></summary>

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

</details>

<details>
<summary><strong>Album Covers</strong></summary>

Kitty supports full-resolution, full-color images. It shouldn't be particularly hard to add this feature.

I'll have to figure out the best UI and UX for this, and probably make it optional/configurable.

</details>

## Supported Audio Formats

The following formats should work: `aac`, `flac`, `mp3`, `mp4`, `m4a`, `ogg`, `wav`.

<details>
<summary>Codec Issues?</summary>

Jolteon uses Rodeo to play music, with the symphonia backend. 

I mainly use `flac` files, and some `mp3`. Other formats aren't usually tested.

So far, I've only found _one_ issue with _one_ flac file, which fails to perform seeks, and, after a few seconds of playback, causes the cpal thread to panic, crashing Jolteon. 
This same file does run well with `mpv`. It does report errors reading it, but it still recovers well from them, and is able to seek without issues.

I tried switching the flac backend, but got even worse results. I looked into using [libmpv](https://github.com/mpv-player/mpv-examples/tree/master/libmpv) and [libavcodec](https://www.ffmpeg.org/libavcodec.html), which, in my mind, are pretty much guaranteed to be more stable, but switching to them is far from trivial.

Figuring out the specific bug in the flac codes built in pure Rust is probably an easier and more reasonable path forward.

And Jolteon shouldn't crash if the audio playback crashes, but that's a story for another day.

</details>

## Performance

tl;dr: is good

<details>
<summary>longer explanation</summary>

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

</details>

## Philosophy

- Support features, UI and UX similar to `cmus`
- Statically linked, dependency free, single file binary that anyone can just download, `chmod a+x` and run.

### History & Rant

See [HISTORY.md](docs/HISTORY.md).

## Developing

See [DEVELOPING.md](docs/DEVELOPING.md)

## Bugs

See [BUGS.md](docs/BUGS.md).
