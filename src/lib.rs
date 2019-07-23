use std::path::PathBuf;
use std::time::SystemTime;

/// The base watchable trait.
pub trait Watchable {
    /// The path associated with the watchable object
    fn path(&self) -> &PathBuf;
}

/// A standalone implementation of the watchable trait.
#[derive(Debug, Clone)]
pub struct BasicTarget {
    /// The path we want to watch
    path: PathBuf,
}

impl BasicTarget {
    pub fn new<T: Into<PathBuf>>(path: T) -> Self {
        Self { path: path.into() }
    }
}

impl Watchable for BasicTarget {
    fn path(&self) -> &PathBuf {
        &self.path
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
/// State transitions that a watchable may undergo.
pub enum Transition {
    Created,
    Modified,
    Deleted,
    None,
}

#[derive(Debug, Eq, PartialEq)]
/// The current state of the watchable.
pub enum WatchState {
    DoesNotExist,
    Exists(Option<SystemTime>),
}

#[derive(Debug, Default)]
/// A watcher instance.
///
/// An instance of Watcher keeps track of a vector of watchables and their corresponding states.
pub struct Watcher<W: Watchable> {
    targets: Vec<W>,
    states: Vec<WatchState>,
}

fn compute_state<W: Watchable>(target: &W) -> WatchState {
    // Does the specified path exist
    let file_exists = target.path().exists();

    // Compute the last modification date of this file, if possible
    let mut last_modified_date = None;

    if file_exists {
        // Determine the last modification time of this file
        let metadata = std::fs::metadata(target.path());

        if let Ok(metadata) = metadata {
            if let Ok(modified) = metadata.modified() {
                last_modified_date = Some(modified);
            }
        }
    }

    if file_exists {
        WatchState::Exists(last_modified_date)
    } else {
        WatchState::DoesNotExist
    }
}

impl<W: Watchable> Watcher<W> {
    /// Create a new watcher instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use fwatch::{BasicTarget, Watcher};
    ///
    /// fn main() {
    ///     let mut watcher: Watcher<BasicTarget> = Watcher::new();
    /// }
    /// ```
    pub fn new() -> Self {
        Watcher {
            targets: Vec::new(),
            states: Vec::new(),
        }
    }

    /// Adds a target to the watcher.
    ///
    /// # Examples
    ///
    /// ```
    /// use fwatch::{BasicTarget, Watcher};
    ///
    /// fn main() {
    ///     let mut watcher : Watcher<BasicTarget> = Watcher::new();
    ///
    ///     // Watch the "foo.txt" path
    ///     watcher.add_target(BasicTarget::new("foo.txt"));
    /// }
    /// ```
    pub fn add_target(&mut self, target: W) {
        self.states.push(compute_state(&target));
        self.targets.push(target);
    }

    /// Remove a target from the watcher.
    ///
    /// This function will panic if index is greater than the size of self.states() / self.targets().
    ///
    /// # Examples
    ///
    /// ```
    /// use fwatch::{BasicTarget, Watcher};
    ///
    /// fn main() {
    ///     let mut watcher : Watcher<BasicTarget> = Watcher::new();
    ///
    ///     // Inserts "foo.txt" at index 0
    ///     watcher.add_target(BasicTarget::new("foo.txt"));
    ///
    ///     // Remove "foo.txt" from the watcher
    ///     assert!(watcher.remove_target(0));
    /// }
    /// ```
    pub fn remove_target(&mut self, index: usize) -> bool {
        if index > self.states.len() {
            false
        } else {
            self.states.remove(index);
            self.targets.remove(index);

            true
        }
    }

    /// Attempt to get the state corresponding to the given target index.
    ///
    /// Note that this doesn't update the current state.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use fwatch::{BasicTarget, Watcher, WatchState};
    ///
    /// fn main() {
    ///     let mut watcher : Watcher<BasicTarget> = Watcher::new();
    ///
    ///     // Watch a file that doesn't exist
    ///     watcher.add_target(BasicTarget::new("does_not_exist.txt"));
    ///     assert_eq!(watcher.get_state(0).unwrap(), &WatchState::DoesNotExist);
    ///
    ///     // Watch a file that does exist
    ///     watcher.add_target(BasicTarget::new("exists.txt"));
    ///     assert_ne!(watcher.get_state(1).unwrap(), &WatchState::DoesNotExist);
    /// }
    /// ```
    pub fn get_state(&self, index: usize) -> Option<&WatchState> {
        self.states.get(index)
    }

    /// Attempt to get the path corresponding to the given target index.
    ///
    /// # Examples
    ///
    /// ```
    /// use fwatch::{BasicTarget, Watcher, WatchState};
    ///
    /// fn main() {
    ///     let mut watcher : Watcher<BasicTarget> = Watcher::new();
    ///     watcher.add_target(BasicTarget::new("foo.txt"));
    ///
    ///     let path = watcher.get_path(0).unwrap();
    ///     assert_eq!(path.to_str().unwrap(), "foo.txt");
    /// }
    /// ```
    pub fn get_path(&self, index: usize) -> Option<&PathBuf> {
        self.targets.get(index).and_then(|v| Some(v.path()))
    }

    /// Observe any state transitions in our targets.
    ///
    /// Returns a vector containing the observed state transition for each target.
    ///
    /// # Examples
    ///
    /// ```
    /// use fwatch::{BasicTarget, Watcher, Transition};
    ///
    /// fn main() {
    ///     let mut watcher : Watcher<BasicTarget> = Watcher::new();
    ///
    ///     // Watch a file that doesn't exist
    ///     watcher.add_target(BasicTarget::new("does_not_exist.txt"));
    ///
    ///     let results = watcher.watch();
    ///
    ///     for (index, transition) in watcher.watch().into_iter().enumerate() {
    ///         // Get a reference to the path and state of the current target
    ///         let path = watcher.get_path(index).unwrap();
    ///         let state = watcher.get_state(index).unwrap();
    ///
    ///         match transition {
    ///             Transition::Created => { /* The watched file has been created */ },
    ///             Transition::Modified => { /* The watched file has been modified */ },
    ///             Transition::Deleted => { /* The watched file has been deleted */ },
    ///             Transition::None => { /* None of the above transitions were observed */ },
    ///         }
    ///     }
    /// }
    /// ```
    pub fn watch(&mut self) -> Vec<Transition> {
        let mut result = Vec::new();

        for (index, target) in self.targets.iter().enumerate() {
            let previous_state = self.states.get(index).unwrap();
            let current_state = compute_state(target);
            let mut transition = Transition::None;

            // Check for state transitions
            match (previous_state, &current_state) {
                // The file was created
                (WatchState::DoesNotExist, WatchState::Exists(_)) => {
                    transition = Transition::Created;
                }
                // The file was deleted
                (WatchState::Exists(_), WatchState::DoesNotExist) => {
                    transition = Transition::Deleted;
                }
                // The file was modified
                (WatchState::Exists(Some(t1)), WatchState::Exists(Some(t2))) if t1 != t2 => {
                    transition = Transition::Modified;
                }
                _ => {}
            };

            // now update our state vector
            *self.states.get_mut(index).unwrap() = current_state;

            result.push(transition);
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use crate::{BasicTarget, Transition, Watcher};
    use std::io::{Error, Write};
    use std::thread::sleep;
    use std::time::Duration;
    use tempfile::NamedTempFile;

    #[test]
    /// Creates a temporary file and tests the modification + deletion transitions
    fn transitions() -> Result<(), Error> {
        let mut watcher: Watcher<BasicTarget> = Watcher::new();

        // Open a named temporary file & add it to our watcher
        let tmp = NamedTempFile::new()?;
        watcher.add_target(BasicTarget::new(tmp.path()));

        // We're going to modify our temporary file - to ensure the modification time
        // ono our temporary file changes, we wait a bit over a second before modifying
        sleep(Duration::from_millis(1500));

        {
            let mut handle = tmp.reopen()?;
            write!(handle, "test")?;
        }

        // The watcher should notice the modification transition
        assert_eq!(watcher.watch(), vec![Transition::Modified]);

        // Delete the temporary file
        tmp.close()?;

        // The watcher should observe the deletion transition
        assert_eq!(watcher.watch(), vec![Transition::Deleted]);

        Ok(())
    }
}
