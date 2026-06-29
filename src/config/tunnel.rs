use std::fmt;

use toml::Value;

use super::{read_bool, read_string};

#[derive(Clone, PartialEq, Eq)]
pub struct TunnelConfig {
    pub provider: String,
    pub auto_start: bool,
    pub config_password: String,
}

impl Default for TunnelConfig {
    fn default() -> Self {
        Self {
            provider: "cloudflare_quick".to_string(),
            auto_start: false,
            config_password: String::new(),
        }
    }
}

impl fmt::Debug for TunnelConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let password = if self.config_password.is_empty() {
            ""
        } else {
            "<redacted>"
        };

        formatter
            .debug_struct("TunnelConfig")
            .field("provider", &self.provider)
            .field("auto_start", &self.auto_start)
            .field("config_password", &password)
            .finish()
    }
}

pub(super) fn read_tunnel_config(
    table: &toml::map::Map<String, Value>,
    tunnel: &mut TunnelConfig,
    warnings: &mut Vec<String>,
) {
    read_string(
        table.get("provider"),
        "tunnel.provider",
        &mut tunnel.provider,
        warnings,
    );
    read_bool(
        table.get("auto_start"),
        "tunnel.auto_start",
        &mut tunnel.auto_start,
        warnings,
    );
    read_string(
        table.get("config_password"),
        "tunnel.config_password",
        &mut tunnel.config_password,
        warnings,
    );
}
