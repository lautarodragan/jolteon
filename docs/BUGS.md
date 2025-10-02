# Known Bugs

Jolteon is a work-in-progress, and currently undergoing heavy development.

I use Jolteon every day, almost all day, to listen to music, so I generally find bugs quickly, and fix them quickly, if I can.

There are probably many bugs there yet, and I'm probably going to introduce new ones as I keep adding features and refactoring code.

Having said that, the rate at which I find bugs has lowered considerably over time, and the speed with which I fix them, gone up.

Some known issues:
- I still use `unwrap` in some places I'd prefer not to. This will panic if something unexpected happens.
- Deadlocks. There may be some out there that I haven't caught.

## Metadata: Track Number

Jolteon tries to parse the track number as a number (which is a string), and discards non-number inputs.
Some track contain things like `A1`, `A2` and then `B1` in the track number, as a way to distinguish disc numbers.
There is no standard Disc Number media tag, either.

## File Browser: Performance

Jolteon currently reads an entire directory and parses most files in it! It loads CUE sheets, Jolt files and music metadata.
This would be excellent UX if HDs had infinite speed and life, but is generally a bad idea in the real world. 

It's specially problematic when mixing mechanical drives (which is where you'll usually want to store high-quality media files) 
and large directories (which is what music lovers usually have).

Even worse, in rare cases, some files cause the media tag reading code to take a long time.

~~All of this freezes the UI completely while processing.~~ this now runs in a separate thread, with debouncing, so it's no longer such a big issue.

The best UX may be:
- Simply show the list of directory entries — just the file names
- Replace list elements, one by one, with their parsed counterparts, as they come in from the thread
- Store the file metadata somewhere (a second media library, basically!), as a cache, including file.date_modified 
- Next time, if dir_entry.date_modified != cache_entry.date_modified { read from disc } otherwise just cache'd data

It'd be nice to reuse most of the existing library code for the cache. It's practically the same thing.

## List Component: Search Scroll Bug

When using search, the view isn't scrolled to show the selected element after modifying the filter (meaning: pressing letter or number keys).
It is correctly scrolled when navigating the search matches with the arrow keys.

This happens in the Artist/Album Tree in the Library, too, but this component hasn't been replaced by the generic List component yet,
and it lacks some features it has, so there's no sense in fixing it there. It just needs to be replaced by the generic list component,
but the list component needs to be enhanced before that can happen, to support a "tree" view.

## High CPU Usage (in rare case)

On a fresh Arch Linux install, without pipewire, Jolteon consumes a crazy amount of CPU.
Many things do not work well without a proper audio setup, and using the system without pipewire (or something equivalent) is not a normal thing,
but, still, `mpv` doesn't do this — it's CPU usage is normal, even in this situation.

Profiling with 
