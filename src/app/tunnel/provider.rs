use chrono::{DateTime, Utc};

use super::{TunnelProvider, TunnelStatus};

mod cloudflare;

pub use cloudflare::{cloudflare_trycloudflare_url, CloudflareQuickTunnelProvider};

pub trait TunnelProviderController {
    fn provider(&self) -> TunnelProvider;
    fn status(&self, now: DateTime<Utc>) -> TunnelStatus;
    fn start(&mut self, now: DateTime<Utc>) -> TunnelStatus;
    fn stop(&mut self, now: DateTime<Utc>) -> TunnelStatus;
}

pub struct TunnelManager<P> {
    provider: P,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SshTunnelProvider;

impl<P> TunnelManager<P>
where
    P: TunnelProviderController,
{
    pub const fn new(provider: P) -> Self {
        Self { provider }
    }

    pub fn provider(&self) -> TunnelProvider {
        self.provider.provider()
    }

    pub fn status(&self, now: DateTime<Utc>) -> TunnelStatus {
        self.provider.status(now)
    }

    pub fn start(&mut self, now: DateTime<Utc>) -> TunnelStatus {
        self.provider.start(now)
    }

    pub fn stop(&mut self, now: DateTime<Utc>) -> TunnelStatus {
        self.provider.stop(now)
    }
}

impl TunnelProviderController for SshTunnelProvider {
    fn provider(&self) -> TunnelProvider {
        TunnelProvider::Ssh
    }

    fn status(&self, now: DateTime<Utc>) -> TunnelStatus {
        TunnelStatus::unsupported(TunnelProvider::Ssh, now)
    }

    fn start(&mut self, now: DateTime<Utc>) -> TunnelStatus {
        TunnelStatus::unsupported(TunnelProvider::Ssh, now)
    }

    fn stop(&mut self, now: DateTime<Utc>) -> TunnelStatus {
        TunnelStatus::unsupported(TunnelProvider::Ssh, now)
    }
}
