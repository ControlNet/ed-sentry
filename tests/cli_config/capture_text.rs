use std::io::{BufRead, BufReader};
use std::process::Stdio;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

const WATCH_STARTUP_DEADLINE: Duration = Duration::from_secs(10);
const POLL_INTERVAL: Duration = Duration::from_millis(20);
const STARTUP_MARKERS: [&str; 2] = ["Starting... (Press Ctrl+C to stop)", "Scan:"];

pub fn capture_watch_startup(mut command: std::process::Command) -> String {
    command.stdout(Stdio::piped()).stderr(Stdio::null());
    let mut child = command.spawn().unwrap();
    let stdout = child.stdout.take().unwrap();
    let (sender, receiver) = mpsc::channel();
    let reader = thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        loop {
            let mut chunk = Vec::new();
            match reader.read_until(b'\n', &mut chunk) {
                Ok(0) => return,
                Ok(_) => {
                    if sender.send(chunk).is_err() {
                        return;
                    }
                }
                Err(_) => return,
            }
        }
    });

    let mut stdout = Vec::new();
    let started = Instant::now();
    while started.elapsed() < WATCH_STARTUP_DEADLINE {
        read_available(
            &receiver,
            &mut stdout,
            WATCH_STARTUP_DEADLINE.saturating_sub(started.elapsed()),
        );
        if contains_all(&stdout, &STARTUP_MARKERS) {
            assert!(
                child.try_wait().unwrap().is_none(),
                "watch exited after producing expected startup output"
            );
            child.kill().ok();
            drain_after_exit(&mut child, reader, &receiver, &mut stdout);
            return String::from_utf8(stdout).unwrap();
        }
        if child.try_wait().unwrap().is_some() {
            drain_after_exit(&mut child, reader, &receiver, &mut stdout);
            panic!(
                "watch exited before expected startup output; expected stdout {:?}; captured stdout: {}",
                STARTUP_MARKERS,
                String::from_utf8_lossy(&stdout)
            );
        }
    }

    child.kill().ok();
    drain_after_exit(&mut child, reader, &receiver, &mut stdout);
    panic!(
        "timed out waiting for watch startup output; expected stdout {:?}; captured stdout: {}",
        STARTUP_MARKERS,
        String::from_utf8_lossy(&stdout)
    );
}

fn read_available(receiver: &Receiver<Vec<u8>>, stdout: &mut Vec<u8>, remaining: Duration) {
    match receiver.recv_timeout(remaining.min(POLL_INTERVAL)) {
        Ok(chunk) => stdout.extend(chunk),
        Err(RecvTimeoutError::Timeout) => {}
        Err(RecvTimeoutError::Disconnected) => {}
    }
    while let Ok(chunk) = receiver.try_recv() {
        stdout.extend(chunk);
    }
}

fn drain_after_exit(
    child: &mut std::process::Child,
    reader: JoinHandle<()>,
    receiver: &Receiver<Vec<u8>>,
    stdout: &mut Vec<u8>,
) {
    let _ = child.wait();
    let _ = reader.join();
    while let Ok(chunk) = receiver.try_recv() {
        stdout.extend(chunk);
    }
}

fn contains_all(output: &[u8], needles: &[&str]) -> bool {
    let output = String::from_utf8_lossy(output);
    needles.iter().all(|needle| output.contains(needle))
}
