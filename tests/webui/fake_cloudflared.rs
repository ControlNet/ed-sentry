use std::path::{Path, PathBuf};

pub(super) enum FakeCloudflared {
    EmitUrlThenWait,
    LogStartedThenWait(PathBuf),
    EmitFirstUrlThenExit(PathBuf),
}

pub(super) fn fake_cloudflared(dir: &Path, behavior: FakeCloudflared) -> PathBuf {
    let path = dir.join(fake_cloudflared_name());
    std::fs::write(&path, fake_cloudflared_script(&behavior)).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    path
}

fn fake_cloudflared_name() -> &'static str {
    if cfg!(windows) {
        "cloudflared-fixture.cmd"
    } else {
        "cloudflared-fixture"
    }
}

fn fake_cloudflared_script(behavior: &FakeCloudflared) -> String {
    if cfg!(windows) {
        return fake_cloudflared_batch(behavior);
    }
    fake_cloudflared_shell(behavior)
}

fn fake_cloudflared_shell(behavior: &FakeCloudflared) -> String {
    let body = match behavior {
        FakeCloudflared::EmitUrlThenWait => {
            "printf '%s\n' 'https://fixture.trycloudflare.com'; while :; do sleep 1; done"
                .to_string()
        }
        FakeCloudflared::LogStartedThenWait(path) => format!(
            "printf 'started\n' >> {}; printf '%s\n' 'https://fixture.trycloudflare.com'; while :; do sleep 1; done",
            shell_quote(path)
        ),
        FakeCloudflared::EmitFirstUrlThenExit(counter) => format!(
            "count=$(cat '{}' 2>/dev/null || printf 0); count=$((count + 1)); printf '%s' \"$count\" > '{}'; if [ \"$count\" = 1 ]; then printf '%s\n' 'https://fixture.trycloudflare.com'; exit 0; fi; printf '%s\n' 'https://rotated.trycloudflare.com'; while :; do sleep 1; done",
            counter.display(),
            counter.display()
        ),
    };
    format!("#!/bin/sh\n{body}\n")
}

fn fake_cloudflared_batch(behavior: &FakeCloudflared) -> String {
    let body = match behavior {
        FakeCloudflared::EmitUrlThenWait => {
            "echo https://fixture.trycloudflare.com\r\nping -n 31 127.0.0.1 >NUL".to_string()
        }
        FakeCloudflared::LogStartedThenWait(path) => format!(
            "echo started>> {}\r\necho https://fixture.trycloudflare.com\r\nping -n 31 127.0.0.1 >NUL",
            batch_quote(path)
        ),
        FakeCloudflared::EmitFirstUrlThenExit(counter) => {
            let quoted_counter = batch_quote(counter);
            format!(
                "set /P count=< {quoted_counter} 2>NUL\r\nif \"%count%\"==\"\" set count=0\r\nset /A count=%count%+1\r\necho %count%> {quoted_counter}\r\nif \"%count%\"==\"1\" (\r\n  echo https://fixture.trycloudflare.com\r\n  exit /B 0\r\n)\r\necho https://rotated.trycloudflare.com\r\nping -n 31 127.0.0.1 >NUL"
            )
        }
    };
    format!("@echo off\r\n{body}\r\n")
}

fn batch_quote(path: &Path) -> String {
    format!("\"{}\"", path.display())
}

fn shell_quote(path: &Path) -> String {
    format!("'{}'", path.display().to_string().replace('\'', "'\\''"))
}
