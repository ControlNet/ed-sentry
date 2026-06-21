use toml::Value;

use super::{read_bool, read_string, read_u16};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WebConfig {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub open_browser: bool,
}

impl Default for WebConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            host: "127.0.0.1".to_string(),
            port: 8765,
            open_browser: false,
        }
    }
}

pub(super) fn read_web_config(
    table: &toml::map::Map<String, Value>,
    web: &mut WebConfig,
    warnings: &mut Vec<String>,
) {
    read_bool(
        table.get("enabled"),
        "web.enabled",
        &mut web.enabled,
        warnings,
    );
    read_string(table.get("host"), "web.host", &mut web.host, warnings);
    read_u16(table.get("port"), "web.port", &mut web.port, warnings);
    read_bool(
        table.get("open_browser"),
        "web.open_browser",
        &mut web.open_browser,
        warnings,
    );
    if !is_localhost_bind(&web.host) {
        warnings.push(format!(
            "config key web.host binds to non-localhost address {}; WebUI remains local-first and future write endpoints must stay protected",
            web.host
        ));
    }
}

fn is_localhost_bind(host: &str) -> bool {
    matches!(host, "127.0.0.1" | "localhost" | "::1" | "[::1]")
}

#[cfg(test)]
pub(super) mod test_support {
    use super::*;
    use crate::config::{AppConfig, CliConfigOverrides};

    pub(in crate::config) fn assert_defaults_to_disabled_localhost() {
        let loaded = AppConfig::from_toml_str("").unwrap();

        assert!(loaded.warnings.is_empty());
        assert_eq!(
            loaded.config.web,
            WebConfig {
                enabled: false,
                host: "127.0.0.1".to_string(),
                port: 8765,
                open_browser: false,
            }
        );

        let runtime = loaded.config.into_runtime(&CliConfigOverrides::default());
        assert_eq!(runtime.web, WebConfig::default());
    }

    pub(in crate::config) fn assert_enabled_preserves_host_port_open_browser() {
        let loaded = AppConfig::from_toml_str(
            r#"
            [web]
            enabled = true
            host = "127.0.0.1"
            port = 9876
            open_browser = true
            "#,
        )
        .unwrap();

        assert!(loaded.warnings.is_empty());
        assert!(loaded.config.web.enabled);
        assert_eq!(loaded.config.web.host, "127.0.0.1");
        assert_eq!(loaded.config.web.port, 9876);
        assert!(loaded.config.web.open_browser);

        let web = loaded.config.web.clone();
        let runtime = loaded.config.into_runtime(&CliConfigOverrides::default());
        assert_eq!(runtime.web, web);
    }

    pub(in crate::config) fn assert_wrong_typed_keys_warn_and_keep_defaults() {
        let loaded = AppConfig::from_toml_str(
            r#"
            [web]
            enabled = "yes"
            host = 42
            port = "8765"
            open_browser = "please"
            "#,
        )
        .unwrap();

        assert_eq!(loaded.config.web, WebConfig::default());
        assert_eq!(loaded.warnings.len(), 4);
        assert!(loaded.warnings[0].contains("web.enabled"));
        assert!(loaded.warnings[1].contains("web.host"));
        assert!(loaded.warnings[2].contains("web.port"));
        assert!(loaded.warnings[3].contains("web.open_browser"));
    }

    pub(in crate::config) fn assert_non_localhost_warns_without_blocking() {
        let loaded = AppConfig::from_toml_str(
            r#"
            [web]
            enabled = true
            host = "0.0.0.0"
            "#,
        )
        .unwrap();

        assert!(loaded.config.web.enabled);
        assert_eq!(loaded.config.web.host, "0.0.0.0");
        assert_eq!(loaded.warnings.len(), 1);
        assert!(loaded.warnings[0].contains("web.host"));
        assert!(loaded.warnings[0].contains("non-localhost"));
    }
}

#[cfg(test)]
mod tests {
    use super::test_support;

    #[test]
    fn config_web_defaults_to_disabled_localhost() {
        test_support::assert_defaults_to_disabled_localhost();
    }

    #[test]
    fn config_web_enabled_preserves_host_port_open_browser() {
        test_support::assert_enabled_preserves_host_port_open_browser();
    }

    #[test]
    fn config_web_wrong_typed_keys_warn_and_keep_defaults() {
        test_support::assert_wrong_typed_keys_warn_and_keep_defaults();
    }

    #[test]
    fn config_web_non_localhost_warns_without_blocking() {
        test_support::assert_non_localhost_warns_without_blocking();
    }
}
