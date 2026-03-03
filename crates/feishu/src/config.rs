//! Feishu channel configuration.

use serde::{Deserialize, Serialize};

/// Feishu channel configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FeishuConfig {
    /// App ID for Feishu/Lark application.
    pub app_id: String,
    /// App secret for Feishu/Lark application.
    pub app_secret: String,
    /// Verification token for webhook validation.
    pub verification_token: Option<String>,
    /// Encrypt key for message encryption.
    pub encrypt_key: Option<String>,
}
