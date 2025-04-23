# TODO

## Configurable Skip on Paused Behavior

- Mode 1: do nothing (normal) (Jolteon stays _paused_. Adding songs to the queue or skipping songs will not start playback automatically)
- Mode 2: un-pause when stopping/skipping current song.
- Mode 3: un-pause when stopping current song, only if there are no more songs in the queue. 

## Toggle Repeat

We can currently enable REPEAT ONE with Alt+R and disable it with Alt+T. 
Support for toggling instead.

## Jolteon Idle Animation

- Store a "start_frame". Start after 2 seconds of being idle.
- Support variable frame rate.

## User Message Log Screen

- Temporarily display user messages over the _currently playing_ area.
- After a time-out, just permanently hide that message from that area, going back to showing the usual _currently playing_ stuff.
- Store these messages permanently in a new, "log" screen.
- This log screen is unrelated to debugging logs (`log::debug`, etc). It's only for user-facing messages.

## AtomicDuration

An `AtomicDuration` struct may be more ergonomic than a `Mutex<Duration>`, and might be marginally faster.

```rs
AtomicDuration { 
    seconds: AtomicU64,  
    nanos: AtomicU32, 
}
```

It also may not, and it could be marginally slower. Atomics can spin-lock, while Futexes will suspend the thread.

Still, a Mutex, in Unix, uses a Futex, that has an AtomicU32 inside, so we'd just be adding an AtomicU64 and saving us a lock.

```rs
pub struct Mutex {
    futex: Atomic,
}

pub struct Mutex {
    futex: Atomic,
}

type Atomic = futex::SmallAtomic;

pub type SmallAtomic = AtomicU32;
```

And futexes do a syscall.

All in all, it probably won't do any difference at all for Jolteon. We only lock threads on shared resources between the rendering and the playing thread,
and that only happens up to FPS on the rendering side and up to once per song in the queue or once per keyboard input, which is pretty low. Always above
1ms.

So the only difference might be in ergonomics: 

```rs
// now, with mutex:
total_time: Arc::new(Mutex::new(total_time)),
self.total_time.lock().unwrap().clone()
*self.total_time.lock().unwrap() = song_list_to_duration(&songs);

// then, with atomics:
total_time: Arc::new(AtomicDuration::from_duration(total_time)), // or Into trait?
self.total_time.clone()
self.total_time = song_list_to_duration(&songs);
```

Alternatively, since we don't really need nanosecond precision (for display and seek), nor more than... a few hours? a whole day? (for total queue length) of seconds, 
we could just store the millis as an AtomicU64, and do `Duration::from_millis()`. 
There are 86_400_000 millis in a day. The u64 max is 18446744073709551615... that is 213_503_982_334 days? Should be fine lol 
Even a U32 should give us more than 40 days, in which case we'd be using a single AtomicU32, which is what a Mutex is already using inside,
so, at worst, we'd have the same performance of the Mutex.

## Thread-Safe Output Stream

OutputStream is !Send + !Sync, meaning we can't pass it around. We can't drop it either. We just have to hold on to it.
It's really pretty much a wrapper around a cpal Stream.

OutputStreamHandle, on the other hand, is thread-safe, because it doesn't contain the cpal Stream in it.

It'd be nice if we could have a thread-safe cpal Stream. It seems that it only is !Sync and !Send only for Android... and this is future-proofing.

> Streams cannot be `Send` or `Sync` if we plan to support Android's AAudio API. This is
> because the stream API is not thread-safe, and the API prohibits calling certain
> functions within the callback.

See https://github.com/RustAudio/cpal/blob/bbb58ab76787d090d32ed56964bfcf194b8f6a3d/src/platform/mod.rs#L67
