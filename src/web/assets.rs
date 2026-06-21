use std::env;
use std::path::{Path, PathBuf};

const WEBUI_DIST_ENV: &str = "ED_SENTRY_WEBUI_DIST";

pub fn resolve_assets_for_executable(exe_path: &Path) -> Option<PathBuf> {
    if let Some(path) = env_asset_root() {
        return Some(path);
    }
    if let Some(path) = packaged_asset_root(exe_path) {
        return Some(path);
    }
    repo_asset_root()
}

pub(crate) fn resolve_assets() -> Option<PathBuf> {
    if let Some(path) = env_asset_root() {
        return Some(path);
    }
    if let Ok(exe_path) = env::current_exe() {
        if let Some(path) = packaged_asset_root(&exe_path) {
            return Some(path);
        }
    }
    repo_asset_root()
}

fn env_asset_root() -> Option<PathBuf> {
    let path = env::var_os(WEBUI_DIST_ENV).map(PathBuf::from)?;
    asset_root_if_valid(path)
}

fn packaged_asset_root(exe_path: &Path) -> Option<PathBuf> {
    let dir = exe_path.parent()?;
    asset_root_if_valid(dir.join("webui"))
}

fn repo_asset_root() -> Option<PathBuf> {
    asset_root_if_valid(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("ui")
            .join("dist"),
    )
}

fn asset_root_if_valid(path: PathBuf) -> Option<PathBuf> {
    path.join("index.html").is_file().then_some(path)
}
