use std::path::Path;
use std::process::Command;

fn main() {
    let manifest_dir = match std::env::var("CARGO_MANIFEST_DIR") {
        Ok(manifest_dir) => manifest_dir,
        Err(error) => panic!("CARGO_MANIFEST_DIR must be set while building ed-sentry: {error}"),
    };
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/logs/HEAD");
    let latest_commit_date = latest_commit_date(&manifest_dir);
    println!("cargo:rustc-env=ED_SENTRY_COMMIT_DATE={latest_commit_date}");

    let tauri_dir = Path::new(&manifest_dir).join("ui").join("src-tauri");

    std::env::set_current_dir(&tauri_dir)
        .unwrap_or_else(|error| panic!("ed-sentry GUI build must enter ui/src-tauri: {error}"));
    tauri_build::build();
}

fn latest_commit_date(manifest_dir: &str) -> String {
    let output = Command::new("git")
        .args(["log", "-1", "--format=%cd", "--date=format:%Y%m%d"])
        .current_dir(manifest_dir)
        .output()
        .unwrap_or_else(|error| {
            panic!("ed-sentry build must query the latest git commit date: {error}")
        });
    if !output.status.success() {
        panic!(
            "ed-sentry build failed to query the latest git commit date: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    match String::from_utf8(output.stdout) {
        Ok(date) => date.trim().to_string(),
        Err(error) => panic!("latest git commit date must be UTF-8: {error}"),
    }
}
