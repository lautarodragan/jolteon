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

You may need to install `libasound2-dev` or `alsa-lib`:

```
# debian, ubuntu, etc:
sudo apt-get update && sudo apt-get install libasound2-dev

# arch:
pacman -S alsa-lib
```

Check out the GitHub workflows for CI/CD for more details on what to do to get Jolteon to run and build.

Regarding the code: I try to keep the source code as clean and intuitive as I can, so modifying it should be (hopefully) relatively easy.
I'll add an ARCHITECTURE.md soon-ish, which should make the source code friendlier to navigate.
