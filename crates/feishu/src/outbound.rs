//! Feishu outbound messaging.

use tracing::debug;

/// Outbound message sender for Feishu.
#[derive(Debug, Clone)]
pub struct FeishuOutbound;

impl FeishuOutbound {
    /// Creates a new outbound sender.
    pub fn new() -> Self {
        Self
    }
}

impl Default for FeishuOutbound {
    fn default() -> Self {
        Self::new()
    }
}
