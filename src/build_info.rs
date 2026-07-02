pub const APP_NAME: &str = "ed-sentry";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const APP_COMMIT_DATE: &str = env!("ED_SENTRY_COMMIT_DATE");
pub const APP_BUILD_VERSION: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    "-",
    env!("ED_SENTRY_COMMIT_DATE")
);

pub fn app_title() -> String {
    format!("{APP_NAME} {APP_BUILD_VERSION}")
}
