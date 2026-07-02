use std::fs;
use std::path::{Path, PathBuf};

use ed_sentry::app::runtime::DesktopRuntime;
use ed_sentry::app::{TunnelProvider, TunnelStatusKind};
use ed_sentry::config::{AppConfig, CliConfigOverrides};

#[tokio::test]
async fn desktop_runtime_manual_start_keeps_ssh_provider_unsupported() {
    // Given: desktop runtime config selects SSH and cloudflared would log if invoked.
    let temp_dir = tempfile::tempdir().unwrap();
    let dist = tempfile::tempdir().unwrap();
    write_dist(dist.path());
    std::env::set_var("ED_SENTRY_WEBUI_DIST", dist.path());
    let args_log = temp_dir.path().join("args.log");
    let fake = fake_cloudflared(temp_dir.path(), args_log.clone());
    std::env::set_var("ED_SENTRY_CLOUDFLARED_PATH", &fake);
    let journal_path = temp_dir.path().join("Journal.2035-01-08T124500.01.log");
    fs::write(
        &journal_path,
        r#"{"timestamp":"2035-01-08T12:45:00Z","event":"LoadGame","Commander":"Cmdr Fixture","Odyssey":true}"#,
    )
    .unwrap();
    let mut config = AppConfig::default().into_runtime(&CliConfigOverrides {
        set_file: Some(journal_path),
        no_status_line: true,
        poll_interval_ms: Some(60_000),
        ..CliConfigOverrides::default()
    });
    config.web.enabled = true;
    config.web.port = 0;
    config.tunnel.provider = "ssh".to_string();
    let desktop = DesktopRuntime::start(config).await.unwrap();

    // When: the public desktop tunnel start method is called.
    let started = desktop.start_tunnel().await;

    // Then: SSH stays unsupported and the Cloudflare executable is never invoked.
    assert_eq!(started.kind, TunnelStatusKind::Unsupported);
    assert_eq!(started.provider, TunnelProvider::Ssh);
    assert!(!args_log.exists());
    std::env::remove_var("ED_SENTRY_CLOUDFLARED_PATH");
    std::env::remove_var("ED_SENTRY_WEBUI_DIST");
}

fn write_dist(path: &Path) {
    fs::create_dir_all(path).unwrap();
    fs::write(
        path.join("index.html"),
        "<!doctype html><title>ed-sentry</title><main>desktop tunnel</main>",
    )
    .unwrap();
}

fn fake_cloudflared(dir: &Path, args_log: PathBuf) -> PathBuf {
    let path = dir.join(fake_cloudflared_name());
    fs::write(&path, fake_cloudflared_script(&args_log)).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
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

fn fake_cloudflared_script(args_log: &Path) -> String {
    if cfg!(windows) {
        return format!(
            "@echo off\r\necho started>> {}\r\necho https://fixture.trycloudflare.com\r\nping -n 31 127.0.0.1 >NUL\r\n",
            batch_quote(args_log)
        );
    }
    format!(
        "#!/bin/sh\nprintf 'started\n' >> {}; printf '%s\n' 'https://fixture.trycloudflare.com'; while :; do sleep 1; done\n",
        shell_quote(args_log)
    )
}

fn batch_quote(path: &Path) -> String {
    format!("\"{}\"", path.display())
}

fn shell_quote(path: &Path) -> String {
    format!("'{}'", path.display().to_string().replace('\'', "'\\''"))
}
