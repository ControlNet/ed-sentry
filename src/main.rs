use std::process::ExitCode;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LaunchMode {
    Cli,
    Gui,
}

fn main() -> ExitCode {
    match launch_mode_from_args(std::env::args_os()) {
        LaunchMode::Cli => run_cli(),
        LaunchMode::Gui => run_gui(),
    }
}

fn launch_mode_from_args(args: impl IntoIterator<Item = std::ffi::OsString>) -> LaunchMode {
    if args.into_iter().skip(1).any(|arg| arg == "--gui") {
        LaunchMode::Gui
    } else {
        LaunchMode::Cli
    }
}

fn run_cli() -> ExitCode {
    match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime.block_on(ed_sentry::app::cli::run_from_env()),
        Err(error) => {
            eprintln!("Error: failed to start async runtime: {error}");
            ExitCode::from(1)
        }
    }
}

fn run_gui() -> ExitCode {
    match ed_sentry::desktop_gui::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Error: failed to start desktop GUI: {error}");
            ExitCode::from(1)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use super::{launch_mode_from_args, LaunchMode};

    #[test]
    fn launch_mode_uses_cli_when_gui_flag_is_absent() {
        let args = [OsString::from("ed-sentry"), OsString::from("--help")];

        assert_eq!(launch_mode_from_args(args), LaunchMode::Cli);
    }

    #[test]
    fn launch_mode_uses_gui_when_gui_flag_is_present() {
        let args = [OsString::from("ed-sentry"), OsString::from("--gui")];

        assert_eq!(launch_mode_from_args(args), LaunchMode::Gui);
    }
}
