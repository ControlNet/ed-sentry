use std::path::Path;
use std::process::Stdio;
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::text::line_safe;

use super::parser::cloudflare_trycloudflare_url;

const CHILD_EXIT_POLL: Duration = Duration::from_millis(25);
const CHILD_STOP_TIMEOUT: Duration = Duration::from_secs(1);

pub(super) async fn spawn_cloudflared(
    executable: &Path,
    local_url: &str,
    url_timeout: Duration,
) -> Result<(Child, Vec<JoinHandle<()>>, String), String> {
    let mut command = Command::new(executable);
    command
        .args(["tunnel", "--url", local_url, "--metrics", "127.0.0.1:0"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    let mut child = command.spawn().map_err(|error| {
        format!(
            "cloudflared failed to start: {}",
            line_safe(&error.to_string())
        )
    })?;
    let (sender, mut receiver) = mpsc::channel(4);
    let readers = spawn_output_readers(&mut child, sender);
    match tokio::time::timeout(url_timeout, wait_for_tunnel_url(&mut child, &mut receiver)).await {
        Ok(Ok(public_url)) => Ok((child, readers, public_url)),
        Ok(Err(message)) => {
            terminate_child(child).await;
            abort_tasks(readers);
            Err(message)
        }
        Err(_elapsed) => match child.try_wait() {
            Ok(Some(exit_status)) => {
                abort_tasks(readers);
                Err(format!(
                    "cloudflared exited before URL was reported: {exit_status}"
                ))
            }
            Ok(None) => {
                terminate_child(child).await;
                abort_tasks(readers);
                Err("cloudflared did not report a tunnel URL before timeout".to_string())
            }
            Err(error) => {
                terminate_child(child).await;
                abort_tasks(readers);
                Err(format!(
                    "cloudflared status check failed: {}",
                    line_safe(&error.to_string())
                ))
            }
        },
    }
}

pub(super) async fn terminate_child(mut child: Child) {
    let _ = child.start_kill();
    let _ = tokio::time::timeout(CHILD_STOP_TIMEOUT, child.wait()).await;
}

pub(super) fn abort_tasks(tasks: Vec<JoinHandle<()>>) {
    for task in tasks {
        task.abort();
    }
}

fn spawn_output_readers(child: &mut Child, sender: mpsc::Sender<String>) -> Vec<JoinHandle<()>> {
    let mut readers = Vec::new();
    if let Some(stdout) = child.stdout.take() {
        readers.push(spawn_output_reader(stdout, sender.clone()));
    }
    if let Some(stderr) = child.stderr.take() {
        readers.push(spawn_output_reader(stderr, sender));
    }
    readers
}

fn spawn_output_reader<R>(reader: R, sender: mpsc::Sender<String>) -> JoinHandle<()>
where
    R: AsyncRead + Unpin + Send + 'static,
{
    tokio::spawn(async move {
        let mut reported_url = false;
        let mut lines = BufReader::new(reader).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if !reported_url {
                if let Some(public_url) = cloudflare_trycloudflare_url(&line) {
                    let _ = sender.send(public_url).await;
                    reported_url = true;
                }
            }
        }
    })
}

async fn wait_for_tunnel_url(
    child: &mut Child,
    receiver: &mut mpsc::Receiver<String>,
) -> Result<String, String> {
    loop {
        tokio::select! {
            received = receiver.recv() => {
                match received {
                    Some(public_url) => return Ok(public_url),
                    None => return Err(child_exit_message(child).await),
                }
            }
            _ = tokio::time::sleep(CHILD_EXIT_POLL) => {
                match child.try_wait() {
                    Ok(Some(exit_status)) => {
                        return Err(format!("cloudflared exited before URL was reported: {exit_status}"));
                    }
                    Ok(None) => {}
                    Err(error) => {
                        return Err(format!("cloudflared status check failed: {}", line_safe(&error.to_string())));
                    }
                }
            }
        }
    }
}

async fn child_exit_message(child: &mut Child) -> String {
    match tokio::time::timeout(CHILD_EXIT_POLL, child.wait()).await {
        Ok(Ok(exit_status)) => format!("cloudflared exited before URL was reported: {exit_status}"),
        Ok(Err(error)) => format!(
            "cloudflared status check failed: {}",
            line_safe(&error.to_string())
        ),
        Err(_elapsed) => "cloudflared output closed before URL was reported".to_string(),
    }
}
