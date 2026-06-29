mod assets;
mod policy;
mod server;
pub(crate) mod tunnel_state;

pub use assets::resolve_assets_for_executable;
pub use policy::WebEndpointPolicy;
pub use server::{start, start_with_state, WebServer};

#[cfg(test)]
mod tests;
