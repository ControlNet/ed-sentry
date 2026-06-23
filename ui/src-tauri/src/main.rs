#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

fn main() -> ExitCode {
    match launch_gui() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Error: failed to launch ed-sentry.exe --gui: {error}");
            ExitCode::from(1)
        }
    }
}

fn launch_gui() -> std::io::Result<()> {
    let current_exe = env::current_exe()?;
    let mut command = Command::new(backend_exe_path(&current_exe));
    command.arg("--gui");
    configure_gui_command(&mut command);
    command.spawn()?;
    Ok(())
}

#[cfg(windows)]
fn configure_gui_command(command: &mut Command) {
    command.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(windows))]
fn configure_gui_command(_command: &mut Command) {}

fn backend_exe_path(current_exe: &Path) -> PathBuf {
    current_exe
        .parent()
        .map_or_else(|| PathBuf::from(backend_exe_name()), |dir| {
            dir.join(backend_exe_name())
        })
}

#[cfg(windows)]
const fn backend_exe_name() -> &'static str {
    "ed-sentry.exe"
}

#[cfg(not(windows))]
const fn backend_exe_name() -> &'static str {
    "ed-sentry"
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::backend_exe_path;

    #[test]
    fn backend_exe_path_uses_launcher_sibling() {
        let launcher = Path::new("C:/Users/user/Downloads/ed-sentry/ed-sentry-gui.exe");

        assert_eq!(
            backend_exe_path(launcher),
            PathBuf::from("C:/Users/user/Downloads/ed-sentry").join(super::backend_exe_name())
        );
    }

    #[cfg(windows)]
    #[test]
    fn launcher_uses_create_no_window_for_backend_process() {
        assert_eq!(super::CREATE_NO_WINDOW, 0x0800_0000);
    }
}
