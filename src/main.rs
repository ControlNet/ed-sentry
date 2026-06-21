use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    ed_sentry::app::cli::run_from_env().await
}
