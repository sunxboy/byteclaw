//! Feishu access control and permissions.

use tracing::debug;

/// Access control for Feishu channel.
#[derive(Debug, Clone)]
pub struct FeishuAccess;

impl FeishuAccess {
    /// Creates a new access control instance.
    pub fn new() -> Self {
        Self
    }
}

impl Default for FeishuAccess {
    fn default() -> Self {
        Self::new()
    }
}
