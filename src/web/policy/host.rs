use crate::app::ActiveTunnel;

use super::{is_loopback_host, is_wildcard_bind_host};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RequestHost {
    LocalLoopback,
    Bind,
    Tunnel,
}

pub(super) fn normalize_host(host: &str) -> String {
    host.trim().trim_end_matches('.').to_ascii_lowercase()
}

pub(super) fn classify_host(
    host: &str,
    bind_host: &str,
    active_tunnel: Option<&ActiveTunnel>,
) -> Option<RequestHost> {
    let host = normalize_host(host);
    let bind_host = normalize_host(bind_host);
    if is_loopback_host(&host) {
        return Some(RequestHost::LocalLoopback);
    }
    if !host.is_empty() && host == bind_host {
        return Some(RequestHost::Bind);
    }
    if active_tunnel.is_some_and(|active| host == normalize_host(active.host())) {
        return Some(RequestHost::Tunnel);
    }
    if wildcard_host_allowed(&host, &bind_host) {
        return Some(RequestHost::Bind);
    }
    None
}

fn wildcard_host_allowed(host: &str, bind_host: &str) -> bool {
    is_wildcard_bind_host(bind_host) && !host.is_empty() && !is_trycloudflare_host(host)
}

fn is_trycloudflare_host(host: &str) -> bool {
    let host = normalize_host(host);
    host == "trycloudflare.com" || host.ends_with(".trycloudflare.com")
}

#[cfg(test)]
mod tests {
    use crate::app::{ActiveTunnel, TunnelSessionId};

    use super::{classify_host, RequestHost};

    #[test]
    fn classify_host_normalizes_mixed_case_and_trailing_dot_tunnel_host_on_wildcard_bind() {
        let active = ActiveTunnel::new(
            "abc-xyz.trycloudflare.com",
            TunnelSessionId::new("cloudflare-test-1"),
        );

        assert_eq!(
            classify_host("ABC-XYZ.TRYCLOUDFLARE.COM", "0.0.0.0", Some(&active)),
            Some(RequestHost::Tunnel)
        );
        assert_eq!(
            classify_host("abc-xyz.trycloudflare.com.", "0.0.0.0", Some(&active)),
            Some(RequestHost::Tunnel)
        );
        assert_eq!(
            classify_host("Abc-Xyz.TryCloudflare.Com.", "::", Some(&active)),
            Some(RequestHost::Tunnel)
        );
        assert_eq!(
            classify_host("OTHER.TRYCLOUDFLARE.COM", "0.0.0.0", Some(&active)),
            None
        );
        assert_eq!(
            classify_host("other.trycloudflare.com.", "0.0.0.0", Some(&active)),
            None
        );
        assert_eq!(
            classify_host("192.168.1.50", "0.0.0.0", Some(&active)),
            Some(RequestHost::Bind)
        );
        assert_eq!(
            classify_host("Localhost", "127.0.0.1", None),
            Some(RequestHost::LocalLoopback)
        );
        assert_eq!(
            classify_host("LOCALHOST.", "127.0.0.1", None),
            Some(RequestHost::LocalLoopback)
        );
    }
}
