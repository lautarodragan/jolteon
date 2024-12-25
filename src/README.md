# Double Queue: UI and "abstract"

Jolteon uses a few, long-lived threads:
- A "main" one, that handles input and renders the screen
- A "player" one, that grabs songs from the song queue and other sources and plays them when it should
- A "playback" one, focused on a single song, that doesn't have a concept of "playlist", "queue", "folder", "cue sheet", etc --- it just focused on playing one song
- The "mpris" thread, used to communicate with the MPRIS server to support media keys (Linux only)

To keep the code simple, all UI is !Send. UI "components" use `Rc`, not `Arc`, and `Cell` (and siblings), not `Mutex` or `Atomic*`.

We need to show the song queue in the UI, in its dedicated Queue Screen, and allow users to do things with it, 
which has to happen in the "main" input+rendering thread, but we also need the "play" thread to be able to access the queue. 

To achieve this, we duplicate the data:
we keep two lists in memory: one "abstract" that is UI and input agnostic,
and can be sent between threads, and a "UI" one, which does the rendering and input handling.

We need to keep both lists in sync. Conceptually, the "abstract" queue is the source of truth, 
but, in practice, both queues can be changed directly, meaning the sync has to be bidirectional.
For example: the "player" thread will automatically play the next item in the queue when the current song finishes playing,
but the user could also manually remove entries from the queue using the UI.

Changes that happen in the UI queue are _pushed_ into the "abstract" queue immediately, as they happen. 
This is done via `queue_ui.on_enter` etc.

Changes that happen in the "abstract" queue are _pulled_ into the UI queue in the main loop,
blocking this "main" thread — thus blocking rendering and input handling.

At the time of writing, the relevant code portions are:

```rs
// app.rs:158
queue_ui.on_enter({
    let player = player.clone();
    let queue = queue.clone();

    move |song| {
        queue.add_front(song);
        player.stop();
    }
});
queue_ui.on_delete({
    let queue = queue.clone();

    move |_song, index| {
        queue.remove(index);
    }
});
```

```rs
// app.rs:251
if self.queue.length() != self.queue_ui.len() {
    self.queue.with_items(|songs| {
        self.queue_ui.set_items(Vec::from(songs.clone()));
    });
}
```
