use std::time::{Duration, Instant};

use super::AfkWatcherEvent;

#[derive(Debug)]
pub struct DebouncedWatcherEvents {
    window: Duration,
    pending: Vec<PendingDebouncedEvent>,
}

#[derive(Debug)]
struct PendingDebouncedEvent {
    event: AfkWatcherEvent,
    last_seen: Instant,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompanionReadRetry {
    max_attempts: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompanionReadFailure<E> {
    Retryable(E),
    Final(E),
}

impl DebouncedWatcherEvents {
    pub fn new(window: Duration) -> Self {
        Self {
            window,
            pending: Vec::new(),
        }
    }

    pub fn push(&mut self, event: AfkWatcherEvent, now: Instant) {
        if let Some(pending) = self
            .pending
            .iter_mut()
            .find(|pending| pending.event.coalesces_with(&event))
        {
            pending.event = event;
            pending.last_seen = now;
            return;
        }
        self.pending.push(PendingDebouncedEvent {
            event,
            last_seen: now,
        });
    }

    pub fn drain_ready(&mut self, now: Instant) -> Vec<AfkWatcherEvent> {
        let mut ready = Vec::new();
        let mut pending = Vec::new();
        for event in self.pending.drain(..) {
            match now.checked_duration_since(event.last_seen) {
                Some(elapsed) if elapsed >= self.window => ready.push(event.event),
                Some(_) | None => pending.push(event),
            }
        }
        self.pending = pending;
        ready
    }

    pub fn next_ready_delay(&self, now: Instant) -> Option<Duration> {
        self.pending
            .iter()
            .map(|event| {
                event
                    .last_seen
                    .checked_add(self.window)
                    .unwrap_or(event.last_seen)
                    .saturating_duration_since(now)
            })
            .min()
    }
}

impl CompanionReadRetry {
    pub const fn new(max_attempts: usize) -> Self {
        Self { max_attempts }
    }

    pub fn read<T, E>(
        self,
        mut read_once: impl FnMut() -> Result<T, CompanionReadFailure<E>>,
    ) -> Result<T, E> {
        let max_attempts = self.max_attempts.max(1);
        let mut attempts = 0_usize;
        loop {
            attempts += 1;
            match read_once() {
                Ok(value) => return Ok(value),
                Err(CompanionReadFailure::Final(error)) => return Err(error),
                Err(CompanionReadFailure::Retryable(error)) if attempts >= max_attempts => {
                    return Err(error);
                }
                Err(CompanionReadFailure::Retryable(_)) => {}
            }
        }
    }

    pub async fn read_with_delay<T, E>(
        self,
        delay: Duration,
        mut read_once: impl FnMut() -> Result<T, CompanionReadFailure<E>>,
    ) -> Result<T, E> {
        let max_attempts = self.max_attempts.max(1);
        let mut attempts = 0_usize;
        loop {
            attempts += 1;
            match read_once() {
                Ok(value) => return Ok(value),
                Err(CompanionReadFailure::Final(error)) => return Err(error),
                Err(CompanionReadFailure::Retryable(error)) if attempts >= max_attempts => {
                    return Err(error);
                }
                Err(CompanionReadFailure::Retryable(_)) => {
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }
}

impl AfkWatcherEvent {
    fn coalesces_with(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::SelectedFile { path: left }, Self::SelectedFile { path: right })
            | (Self::StatusJson { path: left }, Self::StatusJson { path: right })
            | (Self::CargoJson { path: left }, Self::CargoJson { path: right }) => left == right,
            (Self::WatcherWarning { .. }, Self::WatcherWarning { .. }) => false,
            (Self::SelectedFile { .. }, _)
            | (Self::StatusJson { .. }, _)
            | (Self::CargoJson { .. }, _)
            | (Self::WatcherWarning { .. }, _) => false,
        }
    }
}
