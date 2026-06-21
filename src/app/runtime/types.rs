use std::fmt;
use std::path::PathBuf;

use crate::config::RuntimeConfig;
use crate::journal::{select_configured_journal_file, JournalError};
use crate::notifier::Notification;

use crate::app::{AppSnapshot, EventFeedItem, NotificationView};

#[derive(Debug)]
pub struct RuntimeError {
    message: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MonitorStartup {
    pub journal_file: PathBuf,
    pub commander: Option<String>,
    pub preload_line_count: usize,
    pub preload_eof_offset: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeNotificationDelivery {
    All,
    TerminalOnly,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeNotification {
    pub notification: Notification,
    pub view: NotificationView,
    pub feed_item: EventFeedItem,
    pub delivery: RuntimeNotificationDelivery,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeBatch {
    pub warnings: Vec<String>,
    pub notifications: Vec<RuntimeNotification>,
    pub snapshot: AppSnapshot,
}

impl RuntimeBatch {
    pub fn empty(snapshot: AppSnapshot) -> Self {
        Self {
            warnings: Vec::new(),
            notifications: Vec::new(),
            snapshot,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeStatusSnapshot {
    pub status_line: Option<String>,
    pub dynamic_title: Option<String>,
    pub force_publish: bool,
    pub snapshot: AppSnapshot,
}

pub trait JournalSelector {
    fn select(&mut self, config: &RuntimeConfig) -> Result<PathBuf, RuntimeError>;
}

pub struct ConfiguredJournalSelector;

impl fmt::Display for RuntimeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for RuntimeError {}

impl RuntimeError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl From<JournalError> for RuntimeError {
    fn from(error: JournalError) -> Self {
        Self::new(error.to_string())
    }
}

impl JournalSelector for ConfiguredJournalSelector {
    fn select(&mut self, config: &RuntimeConfig) -> Result<PathBuf, RuntimeError> {
        select_configured_journal_file(config).map_err(RuntimeError::from)
    }
}
