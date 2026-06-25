pub fn binary_command() -> std::process::Command {
    std::process::Command::new(assert_cmd::cargo::cargo_bin("ed-sentry-core"))
}
