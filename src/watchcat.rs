use notify::{DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;

pub struct Watchcat {
    rx: Receiver<DebouncedEvent>,

    // TODO: can the lifetime of the watcher be bound to the resulting struct?
    #[allow(dead_code)]
    watcher: RecommendedWatcher,
}

impl Watchcat {
    pub fn new(folder: &str) -> Result<Self, String> {
        let (tx, rx) = channel();

        // Create a watcher object, delivering debounced events.
        let mut watcher = RecommendedWatcher::new(tx, Duration::from_secs(10))
            .map_err(|err| format!("failed to create watcher Error: {}", err))?;

        watcher
            .watch(folder, RecursiveMode::NonRecursive)
            .map_err(|err| format!("failed to watch {} Error: {}", folder, err))?;

        Ok(Self { watcher, rx })
    }

    pub fn get_changed_filenames(&self) -> Vec<String> {
        let mut filenames: Vec<String> = Vec::new();
        for event in self.rx.try_iter() {
            match event {
                DebouncedEvent::Create(path) | DebouncedEvent::Write(path) => {
                    if let Some(filename) = get_filename_as_string(&path) {
                        filenames.push(filename);
                    }
                }
                _ => {}
            }
        }

        filenames
    }
}

fn get_filename_as_string(path: &Path) -> Option<String> {
    if let Some(filename) = path.file_name() {
        if let Some(string) = filename.to_str() {
            return Some(string.to_owned());
        }
    }

    None
}
