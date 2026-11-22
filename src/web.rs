mod assets;
mod policy;
mod server;
pub(crate) mod tunnel_state;

pub use assets::resolve_assets_for_executable;
pub use policy::{RuntimeConfigApplier, WebEndpointPolicy};
pub use server::{start, start_with_applier, start_with_state, WebServer};

#[cfg(test)]
mod tests;

#[cfg(test)]
pub(crate) async fn test_env_lock() -> tokio::sync::MutexGuard<'static, ()> {
    tests::env_lock().await
}
