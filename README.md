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

*Tested only on Linux*

### Binaries

Binaries are automatically built and published here, in the GitHub repo, built for Linux and Mac, for every new commit to `main`.

🚧 They are currently being published as zipped artifacts uploaded to workflow runs, but not yet as releases.
They will be automatically published as releases in the future. For now, the only way to get the releases it to navigate to 
the [CD workflow](https://github.com/lautarodragan/jolteon/actions/workflows/cd.yml?query=is%3Asuccess+branch%3Amain++), select the most recent one,
and download one of the files listed in the _Artifacts_ section in the lower part of that page.

### Build From Source

See [Developing](#developing) down below.

## Features

- 🚧 Automatic updates
  - Not fully implemented yet. The app checks for new GitHub releases when launched, and detects new versions,
    but doesn't yet download the published binary, and we aren't publishing releases automatically just yet.
- The number keys `1` through `5` select the different screens. The top bar shows the available screens and highlights the active one.
- `Tab` cycles through focusable elements in the screen.
- `Ctrl+Space` toggles play/pause. 
- When paused, a blinking `PAUSED` indicator is displayed in the lower-right corner on the screen.
- A frame counter is show in the top-right corner of the screen.
  - 🚧 This is only there for debugging purposes, and will be configurable and disabled by default in the future. 
- Media library
  - Search/Filter in the artist/album tree. Just press any letter or number key to start filtering. 
    Matches will be displayed in red, and, while filtering, the navigation keys will jump between matches.
    Press `Esc` to exit filtering. Pressing `Enter` to play a song will also exit filtering mode.
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
  - Search/Filter in File Browser (Ctrl+F)
  - Press `j` to add the selected file or folder to the Library
  - Press `y` to add the selected file or folder to the Queue
  - Press `Enter` to play the selected file
  - To add a Cue sheet, rather than selecting the entire folder or a media file, select the cue sheet file and hit `j` or `y` on it instead.
  - 🚧 UX and keyboard shortcuts in the File Browser are inconsistent with the rest of the application. This will soon be fixed, and it'll
    use the same UX and keyboard shortcuts the Library and Playlists screen use.
  - 🚧 Soon, adding a folder to the library or queue will prioritize .cue files inside the folder. Right now, cue sheet files are ignored
    when adding an entire folder, so you'll have to open the folder and work on the individual .cue file instead.
  - 🚧 Currently, the file browser renders on the left half of the screen, and nothing renders on the right half.
    This will soon be addressed, displaying folders on the left half, and files on the right half.
- Playing Queue
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
    - 🚧 This feature isn't build into the binary conditionally, which I'm guessing is breaking Jolteon in the Mac builds.
      This should be straight-forward to fix and will be addressed soon.
- Gapless playback
  - 🚧 Cue sheet files still perform seeking when jumping from one song to the next in the Queue, which might introduce a noticeable delay.
    In the future, I'll improve this feature so, if a track in the queue is followed by the next track in the same cue sheet file,
    no seeking is done at all, which will enable 100% true gapless playback for cue sheet files.
- Persistent app state
  - The current directory of the browser
  - The queue
  - Modifications to playlists and library are saved immediately. This means application crashes or unexpected system shut-downs
    will not prevent changes from being saved.
  - 🚧 Current song, with playback position (coming soon)
- Focus on stability
  - Application crashes are handled safely, restoring the terminal to its normal state before exiting the process.
  - Thread hygiene: all threads joined on exit - no thread is brute-force-killed by the OS on process exit.
  - Minimal use of `unwrap`. Only true bugs in the application should crash Jolteon. Any external source of indeterministic should be
    handled accordingly.
  - 🚧 In the future, if any non-bug causes an issue, rather than just being ignored, proper UX will be implemented and feedback given.
- A clock on the top bar :)
  - 🚧 In the future, we'll be able to enable/disable this feature, as well as configuring the time format.
- 🚧 Configurable keyboard shortcuts
  - This is a work in progress. They are not really configurable yet, unless you're willing to modify `assets/actions.kv`, but 90% the code
    is already there, and this feature will be fully implemented soon.
  - Not every keyboard shortcut will be configurable.
  - Custom keyboard shortcuts will override default ones, but default ones will always be present.
  - Ctrl+Q is handled specially. The application must always be exit-able, no matter what. No "how do I exit Vim" situation.
- 🚧 Themes
  - Currently, there's only one theme. You can find it in `assets/theme.toml`. 90% of the code allowing customization and multiple
    out-of-the-box themes is already done, so this feature is likely to come soon.
  - There will be some way to switch themes programmatically from outside the application, for themes to be switchable by external scripts.
  - Integration with OS light/dark mode will be added. Which theme is associated with each mode will be configurable, but have a sensible default.

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

If you do experience any sort of performance issues — be it choppy UI, keyboard input response, choppy audio, or significantly higher CPU/RAM usage than `mpv` or any other well-known media player
for the same file, please open an issue reporting it. Being able to reproduce this with an audio file available in the public domain, or with a license that permits sharing it, would be ideal,
even if hard or very unlikely.

## Philosophy

- Support features, UI and UX similar to `cmus`
- Statically linked, dependency free, single file binary that anyone can just download, `chmod a+x` and run.

### History & Rant

See [HISTORY.md](./HISTORY.md).

## Developing

Rust is very friendly language —even to newcomers—, and has a very friendly community. 

If you're new to Rust, I encourage you to give it a try. [The Rust Book](https://doc.rust-lang.org/book/) is pretty awesome, [rust-lang.org/learn](https://www.rust-lang.org/learn) is generally a great starting point, and, for slightly more advanced topics, 
Mara Bos's [Rust Atomics and Locks](https://marabos.nl/atomics/) and Jon Gjengset's [Crust of Rust](https://www.youtube.com/playlist?list=PLqbS7AVVErFiWDOAVrPt7aYmnuuOLYvOa) series are great resources.

To install Rust and Cargo, see [rust-lang.org/tools/install](https://www.rust-lang.org/tools/install) or [rust-lang.org/cargo/getting-started](https://doc.rust-lang.org/cargo/getting-started/installation.html).

To get started with Jolteon, just clone the repo and then run the project:

```
git clone --depth=1 https://github.com/lautarodragan/jolteon.git
cd jolteon

cargo run
```

You may need to install `libasound2-dev`:

```
sudo apt-get update && sudo apt-get install libasound2-dev
```

Check out the GitHub workflows for CI/CD for more details on what to do to get Jolteon to run and build.

Regarding the code: I try to keep the source code as clean and intuitive as I can, so modifying it should be (hopefully) relatively easy.
I'll add an ARCHITECTURE.md soon-ish, which should make the source code friendlier to navigate.

Keep in mind I'm using my own fork of `cpal` right now. I have an open [PR for cpal](https://github.com/RustAudio/cpal/pull/909), with a small bugfix, that hasn't been merged yet.

## Bugs

See [BUGS.md](./BUGS.md).
