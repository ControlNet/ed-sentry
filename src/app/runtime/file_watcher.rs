use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use notify::{recommended_watcher, Event, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::mpsc;

use crate::text::line_safe;

mod helpers;

pub use helpers::{CompanionReadFailure, CompanionReadRetry, DebouncedWatcherEvents};

const EVENT_CHANNEL_CAPACITY: usize = 64;
const STATUS_FILE: &str = "Status.json";
const CARGO_FILE: &str = "Cargo.json";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AfkWatcherEvent {
    SelectedFile { path: PathBuf },
    StatusJson { path: PathBuf },
    CargoJson { path: PathBuf },
    WatcherWarning { message: String },
}

#[derive(Debug)]
pub enum AfkFileWatcherStart {
    Watching {
        watcher: AfkFileWatcher,
        events: mpsc::Receiver<AfkWatcherEvent>,
    },
    PollingFallback {
        warning: AfkWatcherFallback,
    },
}

#[derive(Debug)]
pub struct AfkFileWatcher {
    _watcher: RecommendedWatcher,
    paths: Arc<WatchedFileSet>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AfkWatcherFallback {
    message: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct WatcherInitError {
    reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WatchedFileSet {
    selected_file: PathBuf,
    status_file: PathBuf,
    cargo_file: PathBuf,
    watch_dir: PathBuf,
}

impl AfkFileWatcherStart {
    pub fn start(selected_file: impl AsRef<Path>) -> Self {
        match AfkFileWatcher::start(selected_file.as_ref()) {
            Ok((watcher, events)) => Self::Watching { watcher, events },
            Err(error) => Self::PollingFallback {
                warning: AfkWatcherFallback::from_init_error(&error),
            },
        }
    }
}

impl AfkFileWatcher {
    fn start(
        selected_file: &Path,
    ) -> Result<(Self, mpsc::Receiver<AfkWatcherEvent>), WatcherInitError> {
        let paths = Arc::new(WatchedFileSet::new(selected_file));
        let callback_paths = Arc::clone(&paths);
        let (sender, receiver) = mpsc::channel(EVENT_CHANNEL_CAPACITY);
        let callback_sender = sender.clone();
        let mut watcher = recommended_watcher(move |result: notify::Result<Event>| {
            dispatch_notify_result(&callback_paths, &callback_sender, result);
        })
        .map_err(WatcherInitError::from_notify)?;

        watcher
            .watch(paths.watch_dir(), RecursiveMode::NonRecursive)
            .map_err(WatcherInitError::from_notify)?;

        Ok((
            Self {
                _watcher: watcher,
                paths,
            },
            receiver,
        ))
    }

    pub fn watch_dir(&self) -> &Path {
        self.paths.watch_dir()
    }
}

impl AfkWatcherFallback {
    fn from_init_error(error: &WatcherInitError) -> Self {
        let message = line_safe(&format!(
            "file watcher unavailable; continuing with polling fallback ({})",
            error.reason
        ));
        Self { message }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl WatcherInitError {
    fn from_notify(error: notify::Error) -> Self {
        Self {
            reason: format!("{:?}", error.kind),
        }
    }
}

impl WatchedFileSet {
    pub fn new(selected_file: impl AsRef<Path>) -> Self {
        let selected_file = normalize_path(selected_file.as_ref());
        let watch_dir = selected_file
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));
        Self {
            status_file: watch_dir.join(STATUS_FILE),
            cargo_file: watch_dir.join(CARGO_FILE),
            selected_file,
            watch_dir,
        }
    }

    pub fn classify_path(&self, path: impl AsRef<Path>) -> Option<AfkWatcherEvent> {
        let path = normalize_path(path.as_ref());
        if path == self.selected_file {
            return Some(AfkWatcherEvent::SelectedFile { path });
        }
        if path == self.status_file {
            return Some(AfkWatcherEvent::StatusJson { path });
        }
        if path == self.cargo_file {
            return Some(AfkWatcherEvent::CargoJson { path });
        }
        None
    }

    fn normalize_event(&self, event: &Event) -> Vec<AfkWatcherEvent> {
        event
            .paths
            .iter()
            .filter_map(|path| self.classify_path(path))
            .collect()
    }

    fn watch_dir(&self) -> &Path {
        &self.watch_dir
    }
}

fn dispatch_notify_result(
    paths: &WatchedFileSet,
    sender: &mpsc::Sender<AfkWatcherEvent>,
    result: notify::Result<Event>,
) {
    match result {
        Ok(event) => {
            for normalized in paths.normalize_event(&event) {
                let _ = sender.try_send(normalized);
            }
        }
        Err(error) => {
            let message = line_safe(&format!(
                "file watcher event failed; polling fallback remains active ({:?})",
                error.kind
            ));
            let _ = sender.try_send(AfkWatcherEvent::WatcherWarning { message });
        }
    }
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    normalized.push(component.as_os_str());
                }
            }
            Component::Normal(part) => normalized.push(part),
        }
    }
    normalized
}

#[cfg(test)]
mod tests;
