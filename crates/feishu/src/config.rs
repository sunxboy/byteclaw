//! Feishu channel configuration.

use secrecy::Secret;
use serde::{Deserialize, Serialize};

/// Feishu account configuration
#[derive(Debug, Clone, Deserialize)]
pub struct FeishuAccountConfig {
    /// Feishu app ID
    pub app_id: String,
    /// Feishu app secret
    pub app_secret: Secret<String>,
    /// Connection mode (only websocket supported)
    #[serde(default)]
    pub connection_mode: ConnectionMode,
    /// Group chat policy
    #[serde(default)]
    pub group_policy: GroupPolicy,
    /// Allowlist of group chat IDs (for allowlist policy)
    #[serde(default)]
    pub group_allowlist: Vec<String>,
    /// OTP cooldown in seconds
    #[serde(default = "default_otp_cooldown")]
    pub otp_cooldown_secs: u64,
    /// Optional encryption key for message verification
    #[serde(default)]
    pub encrypt_key: Option<Secret<String>>,
}

impl FeishuAccountConfig {
    /// Returns a sanitized JSON value for status reporting (excludes secrets)
    pub fn to_status_json(&self) -> serde_json::Value {
        serde_json::json!({
            "app_id": self.app_id,
            "connection_mode": self.connection_mode,
            "group_policy": self.group_policy,
            "group_allowlist": self.group_allowlist,
            "otp_cooldown_secs": self.otp_cooldown_secs,
            "has_encrypt_key": self.encrypt_key.is_some(),
        })
    }
}

impl Default for FeishuAccountConfig {
    fn default() -> Self {
        Self {
            app_id: String::new(),
            app_secret: Secret::new(String::new()),
            connection_mode: ConnectionMode::default(),
            group_policy: GroupPolicy::default(),
            group_allowlist: Vec::new(),
            otp_cooldown_secs: default_otp_cooldown(),
            encrypt_key: None,
        }
    }
}

fn default_otp_cooldown() -> u64 {
    3600 // 1 hour
}

/// Connection mode for Feishu
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionMode {
    /// WebSocket long connection (default and only supported mode)
    #[default]
    Websocket,
}

/// Group chat policy
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum GroupPolicy {
    /// Only respond in allowlisted groups
    Allowlist,
    /// Respond in all groups
    Open,
    /// Only respond in direct messages (no groups)
    #[default]
    Closed,
}

#[cfg(test)]
mod tests {
    use super::*;
    use secrecy::ExposeSecret;

    #[test]
    fn config_deserialize() {
        let json = r#"{
            "app_id": "cli_xxx",
            "app_secret": "secret_xxx",
            "group_policy": "allowlist",
            "group_allowlist": ["oc_xxx"]
        }"#;
        let cfg: FeishuAccountConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.app_id, "cli_xxx");
        assert_eq!(cfg.app_secret.expose_secret(), "secret_xxx");
        assert_eq!(cfg.group_policy, GroupPolicy::Allowlist);
        assert_eq!(cfg.group_allowlist, vec!["oc_xxx"]);
    }

    #[test]
    fn config_default() {
        let cfg = FeishuAccountConfig::default();
        assert_eq!(cfg.connection_mode, ConnectionMode::Websocket);
        assert_eq!(cfg.group_policy, GroupPolicy::Closed);
        assert_eq!(cfg.otp_cooldown_secs, 3600);
    }

    #[test]
    fn config_to_status_json_excludes_secrets() {
        let cfg = FeishuAccountConfig {
            app_id: "cli_test".to_string(),
            app_secret: Secret::new("secret_value".to_string()),
            connection_mode: ConnectionMode::Websocket,
            group_policy: GroupPolicy::Allowlist,
            group_allowlist: vec!["oc_123".to_string()],
            otp_cooldown_secs: 1800,
            encrypt_key: Some(Secret::new("encrypt_key".to_string())),
        };

        let status = cfg.to_status_json();

        // Verify app_id is present
        assert_eq!(status["app_id"], "cli_test");

        // Verify secrets are NOT included
        assert!(!status.as_object().unwrap().contains_key("app_secret"));
        assert!(!status.as_object().unwrap().contains_key("encrypt_key"));

        // Verify has_encrypt_key boolean is present instead
        assert_eq!(status["has_encrypt_key"], true);

        // Verify other fields are present
        assert_eq!(status["connection_mode"], "websocket");
        assert_eq!(status["group_policy"], "allowlist");
        assert_eq!(status["otp_cooldown_secs"], 1800);
        assert_eq!(status["group_allowlist"].as_array().unwrap().len(), 1);
        assert_eq!(status["group_allowlist"][0], "oc_123");
    }

    #[test]
    fn config_deserialize_all_policies() {
        for policy in ["allowlist", "open", "closed"] {
            let json = format!(
                r#"{{"app_id": "cli_xxx", "app_secret": "secret", "group_policy": "{}"}}"#,
                policy
            );
            let cfg: FeishuAccountConfig = serde_json::from_str(&json).unwrap();
            let expected = match policy {
                "allowlist" => GroupPolicy::Allowlist,
                "open" => GroupPolicy::Open,
                "closed" => GroupPolicy::Closed,
                _ => unreachable!(),
            };
            assert_eq!(cfg.group_policy, expected, "Failed for policy: {}", policy);
        }
    }

    #[test]
    fn connection_mode_default_is_websocket() {
        assert_eq!(ConnectionMode::default(), ConnectionMode::Websocket);
    }

    #[test]
    fn group_policy_default_is_closed() {
        assert_eq!(GroupPolicy::default(), GroupPolicy::Closed);
    }
}
