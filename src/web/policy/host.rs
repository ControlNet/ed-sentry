use crate::app::ActiveTunnel;

use super::{is_loopback_host, is_wildcard_bind_host};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RequestHost {
    LocalOrBind,
    Tunnel,
}

pub(super) fn classify_host(
    host: &str,
    bind_host: &str,
    active_tunnel: Option<&ActiveTunnel>,
) -> Option<RequestHost> {
    if bind_host_allowed(host, bind_host) {
        return Some(RequestHost::LocalOrBind);
    }
    if active_tunnel.is_some_and(|active| host == active.host()) {
        return Some(RequestHost::Tunnel);
    }
    if wildcard_host_allowed(host, bind_host) {
        return Some(RequestHost::LocalOrBind);
    }
    None
}

fn bind_host_allowed(host: &str, bind_host: &str) -> bool {
    host == bind_host || (is_loopback_host(bind_host) && is_loopback_host(host))
}

fn wildcard_host_allowed(host: &str, bind_host: &str) -> bool {
    is_wildcard_bind_host(bind_host) && !host.is_empty() && !is_trycloudflare_host(host)
}

fn is_trycloudflare_host(host: &str) -> bool {
    host == "trycloudflare.com" || host.ends_with(".trycloudflare.com")
}
