use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, channel};
use std::time::Duration;

use notify_debouncer_full::notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher as _};
use notify_debouncer_full::{DebounceEventResult, Debouncer, FileIdMap, new_debouncer};

pub struct Watchcat {
    rx: Receiver<PathBuf>,

    // TODO: can the lifetime of the watcher be bound to the resulting struct?
    #[expect(dead_code)]
    watcher: Debouncer<RecommendedWatcher, FileIdMap>,
}

impl Watchcat {
    pub fn new(folder: &str) -> Self {
        let (tx, rx) = channel();

        let mut watcher = new_debouncer(
            Duration::from_secs(10),
            None,
            move |result: DebounceEventResult| {
                let events = result.expect("file system watcher error");
                let mut paths = events
                    .into_iter()
                    .filter(|event| {
                        matches!(event.kind, EventKind::Create(_) | EventKind::Modify(_))
                    })
                    .flat_map(|debounced_event| debounced_event.event.paths)
                    .collect::<Vec<_>>();
                paths.sort();
                paths.dedup();
                for path in paths {
                    tx.send(path).expect("receiver should still exist");
                }
            },
        )
        .expect("Failed to create file system watcher");

        let path = Path::new(folder);
        watcher
            .watcher()
            .watch(path, RecursiveMode::NonRecursive)
            .expect("failed to watch folder");
        watcher.cache().add_root(path, RecursiveMode::NonRecursive);

        Self { rx, watcher }
    }

    pub fn get_changed_filenames(&self) -> Vec<String> {
        let mut filenames: Vec<String> = Vec::new();
        for path in self.rx.try_iter() {
            if let Some(filename) = get_filename_as_string(&path) {
                filenames.push(filename);
            }
        }

        filenames
    }
}

fn get_filename_as_string(path: &Path) -> Option<String> {
    path.file_name()
        .and_then(std::ffi::OsStr::to_str)
        .map(ToOwned::to_owned)
}
