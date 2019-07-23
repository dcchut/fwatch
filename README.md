# fwatch

fwatch is a file watching library written in Rust.

* [API Documentation](https://docs.rs/fwatch/)
* Cargo package: [fwatch](https://crates.io/crates/actix)

---
## Installation

Add this to your `Cargo.toml':

```toml
[dependencies]
fwatch = "0.1.2"
```
## Basic Usage

The following example sets up a `Watcher` to observe any changes (called `Transition`'s in fwatch) to `foo.txt` and `bar.txt`.  The currently available transitions are `Created`, `Modified`, `Deleted`, and `None`.  

Instead of `BasicTarget`, any struct implementing the `Watchable` trait can be used below.  

```rust,ignore
use fwatch::{BasicTarget, Watcher, Transition};

fn main() {
    let mut watcher : Watcher<BasicTarget> = Watcher::new();

    // Add a couple of files to watch
    watcher.add_target(BasicTarget::new("foo.txt"));
    watcher.add_target(BasicTarget::new("bar.txt"));
    
    // Calling watcher.watch() returns a vector of Transitions, which tell us if the watched
    // files have undergone any state transitions since the previous invocation of watcher.watch()
    for (index, transition) in watcher.watch().into_iter().enumerate() {
        // Get the path and state of the current target
        let path = watcher.get_path(index).unwrap();
        let state = watcher.get_state(index).uwnrap();

        // Do something based on the observed transition.
        match transition {
            Transition::Created => { /* The watched file has been created */ },
            Transition::Modified => { /* The watched file has been modified */ },
            Transition::Deleted => { /* The watched file has been deleted */ },
            Transition::None => { /* None of the above transitions were observed */ },
        }
    }
}
```

## License

This project is licensed under the Apache License, Version 2.0.
