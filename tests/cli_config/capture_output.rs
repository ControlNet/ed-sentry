use std::io::{BufRead, BufReader, Read};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

const POLL_INTERVAL: Duration = Duration::from_millis(20);

#[derive(Debug)]
pub struct CapturedWatchOutput {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

#[derive(Clone, Copy, Debug)]
enum CapturedStream {
    Stdout,
    Stderr,
}

pub struct RunningWatch {
    child: Child,
    receiver: Receiver<(CapturedStream, Vec<u8>)>,
    stdout_thread: Option<JoinHandle<()>>,
    stderr_thread: Option<JoinHandle<()>>,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

impl RunningWatch {
    pub fn spawn(mut command: Command) -> Self {
        command.stdout(Stdio::piped()).stderr(Stdio::piped());
        let child = command.spawn().unwrap();
        Self::from_child(child)
    }

    pub fn from_child(mut child: Child) -> Self {
        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();
        let (sender, receiver) = mpsc::channel();
        let stdout_thread = spawn_reader(CapturedStream::Stdout, stdout, sender.clone());
        let stderr_thread = spawn_reader(CapturedStream::Stderr, stderr, sender);

        Self {
            child,
            receiver,
            stdout_thread: Some(stdout_thread),
            stderr_thread: Some(stderr_thread),
            stdout: Vec::new(),
            stderr: Vec::new(),
        }
    }

    pub fn wait_for_output(
        &mut self,
        stdout_needles: &[&str],
        stderr_needles: &[&str],
        deadline: Duration,
    ) {
        let started = Instant::now();
        while started.elapsed() < deadline {
            self.read_available(deadline.saturating_sub(started.elapsed()));
            if contains_all(&self.stdout, stdout_needles)
                && contains_all(&self.stderr, stderr_needles)
            {
                assert!(
                    self.child.try_wait().unwrap().is_none(),
                    "watch exited after producing expected readiness output"
                );
                return;
            }
            if self.child.try_wait().unwrap().is_some() {
                self.drain_after_exit();
                panic!(
                    "watch exited before expected readiness output; expected stdout {:?}, stderr {:?}; captured stdout: {}; captured stderr: {}",
                    stdout_needles,
                    stderr_needles,
                    String::from_utf8_lossy(&self.stdout),
                    String::from_utf8_lossy(&self.stderr)
                );
            }
        }

        self.child.kill().ok();
        self.drain_after_exit();
        panic!(
            "timed out waiting for watch readiness output; expected stdout {:?}, stderr {:?}; captured stdout: {}; captured stderr: {}",
            stdout_needles,
            stderr_needles,
            String::from_utf8_lossy(&self.stdout),
            String::from_utf8_lossy(&self.stderr)
        );
    }

    pub fn stop(mut self) -> CapturedWatchOutput {
        assert!(
            self.child.try_wait().unwrap().is_none(),
            "watch exited before the test stopped it"
        );
        self.child.kill().ok();
        self.drain_after_exit();
        CapturedWatchOutput {
            stdout: self.stdout,
            stderr: self.stderr,
        }
    }

    fn read_available(&mut self, remaining: Duration) {
        let wait = remaining.min(POLL_INTERVAL);
        match self.receiver.recv_timeout(wait) {
            Ok((stream, chunk)) => self.push_chunk(stream, chunk),
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => {}
        }
        while let Ok((stream, chunk)) = self.receiver.try_recv() {
            self.push_chunk(stream, chunk);
        }
    }

    fn drain_after_exit(&mut self) {
        let _ = self.child.wait();
        self.join_readers();
        while let Ok((stream, chunk)) = self.receiver.try_recv() {
            self.push_chunk(stream, chunk);
        }
    }

    fn join_readers(&mut self) {
        if let Some(stdout_thread) = self.stdout_thread.take() {
            let _ = stdout_thread.join();
        }
        if let Some(stderr_thread) = self.stderr_thread.take() {
            let _ = stderr_thread.join();
        }
    }

    fn push_chunk(&mut self, stream: CapturedStream, chunk: Vec<u8>) {
        match stream {
            CapturedStream::Stdout => self.stdout.extend(chunk),
            CapturedStream::Stderr => self.stderr.extend(chunk),
        }
    }
}

pub fn capture_watch_output_until(
    command: Command,
    stdout_needles: &[&str],
    stderr_needles: &[&str],
    deadline: Duration,
) -> CapturedWatchOutput {
    let mut watch = RunningWatch::spawn(command);
    watch.wait_for_output(stdout_needles, stderr_needles, deadline);
    watch.stop()
}

fn spawn_reader<R>(
    stream: CapturedStream,
    reader: R,
    sender: Sender<(CapturedStream, Vec<u8>)>,
) -> JoinHandle<()>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut reader = BufReader::new(reader);
        loop {
            let mut chunk = Vec::new();
            match reader.read_until(b'\n', &mut chunk) {
                Ok(0) => return,
                Ok(_) => {
                    if sender.send((stream, chunk)).is_err() {
                        return;
                    }
                }
                Err(_) => return,
            }
        }
    })
}

fn contains_all(output: &[u8], needles: &[&str]) -> bool {
    let output = String::from_utf8_lossy(output);
    needles.iter().all(|needle| output.contains(needle))
}
