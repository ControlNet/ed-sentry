use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

pub(super) fn write_document_atomically(
    path: &PathBuf,
    contents: &str,
) -> Result<(), super::ConfigWriteError> {
    let temp_path = temp_write_path(path);
    {
        let mut file =
            fs::File::create(&temp_path).map_err(|error| super::ConfigWriteError::Io {
                path: path.clone(),
                source: error,
            })?;
        file.write_all(contents.as_bytes())
            .and_then(|()| file.sync_all())
            .map_err(|error| super::ConfigWriteError::Io {
                path: path.clone(),
                source: error,
            })?;
    }
    fs::rename(&temp_path, path).map_err(|error| {
        let _ = fs::remove_file(&temp_path);
        super::ConfigWriteError::Io {
            path: path.clone(),
            source: error,
        }
    })
}

fn temp_write_path(path: &Path) -> PathBuf {
    let filename = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("config.toml");
    path.with_file_name(format!(
        ".{filename}.tmp-{}-{}",
        std::process::id(),
        unique_temp_suffix()
    ))
}

fn unique_temp_suffix() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos())
}
