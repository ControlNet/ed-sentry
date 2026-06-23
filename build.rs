use std::path::Path;

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR must be set while building ed-sentry");
    let tauri_dir = Path::new(&manifest_dir).join("ui").join("src-tauri");

    std::env::set_current_dir(&tauri_dir)
        .expect("ed-sentry GUI build must be able to enter ui/src-tauri");
    tauri_build::build();
}
