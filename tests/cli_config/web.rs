pub fn write_webui_dist(path: &std::path::Path, marker: &str) {
    std::fs::create_dir_all(path).unwrap();
    std::fs::write(
        path.join("index.html"),
        format!("<!doctype html><title>ed-sentry</title><main>{marker}</main>"),
    )
    .unwrap();
}

pub fn write_web_config(path: &std::path::Path, journal_folder: &std::path::Path, web_port: u16) {
    std::fs::write(
        path,
        format!(
            r#"
            [journal]
            folder = {:?}

            [web]
            enabled = true
            host = "127.0.0.1"
            port = {}
            open_browser = false
            "#,
            journal_folder.display().to_string(),
            web_port
        ),
    )
    .unwrap();
}
