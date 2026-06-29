use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

const CLOUDFLARED_OVERRIDE_ENV: &str = "ED_SENTRY_CLOUDFLARED_PATH";

pub(super) fn resolve_cloudflared_executable() -> PathBuf {
    let current_exe_parent = current_exe_parent();
    resolve_cloudflared_executable_from(
        env::var_os(CLOUDFLARED_OVERRIDE_ENV),
        current_exe_parent.as_deref(),
    )
}

fn resolve_cloudflared_executable_from(
    override_path: Option<OsString>,
    packaged_sibling_parent: Option<&Path>,
) -> PathBuf {
    if let Some(path) = override_path {
        return PathBuf::from(path);
    }
    #[cfg(windows)]
    {
        if let Some(parent) = packaged_sibling_parent {
            return packaged_cloudflared_path(parent);
        }
        PathBuf::from("tools/cloudflared/cloudflared.exe")
    }
    #[cfg(not(windows))]
    {
        let _ = packaged_sibling_parent;
        PathBuf::from("cloudflared")
    }
}

fn current_exe_parent() -> Option<PathBuf> {
    env::current_exe()
        .ok()
        .and_then(|current_exe| current_exe.parent().map(Path::to_path_buf))
}

#[cfg(any(windows, test))]
fn packaged_cloudflared_path(parent: &Path) -> PathBuf {
    parent
        .join("tools")
        .join("cloudflared")
        .join("cloudflared.exe")
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::path::{Path, PathBuf};

    use super::{packaged_cloudflared_path, resolve_cloudflared_executable_from};

    #[test]
    fn cloudflared_resolver_uses_explicit_override_before_packaged_sibling() {
        // Given: a dev/test override and a packaged executable directory.
        let override_path = OsString::from(r"C:\fixture\cloudflared.exe");
        let package_dir = Path::new(r"C:\Program Files\ed-sentry");

        // When: the resolver chooses the executable path.
        let resolved = resolve_cloudflared_executable_from(Some(override_path), Some(package_dir));

        // Then: the override wins so tests and development can supply a fake executable.
        assert_eq!(resolved, PathBuf::from(r"C:\fixture\cloudflared.exe"));
    }

    #[test]
    fn cloudflared_packaged_path_is_tools_sibling() {
        // Given: the packaged ed-sentry executable directory.
        let package_dir = Path::new(r"C:\Program Files\ed-sentry");

        // When: the packaged sibling path is built.
        let resolved = packaged_cloudflared_path(package_dir);

        // Then: cloudflared is looked up in the package tools subdirectory.
        assert_eq!(
            resolved,
            package_dir
                .join("tools")
                .join("cloudflared")
                .join("cloudflared.exe")
        );
    }
}
