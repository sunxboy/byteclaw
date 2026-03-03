//! Feishu webhook handler.

use tracing::debug;

/// Handles incoming Feishu webhook events.
#[derive(Debug, Clone)]
pub struct FeishuHandler;

impl FeishuHandler {
    /// Creates a new handler instance.
    pub fn new() -> Self {
        Self
    }
}

impl Default for FeishuHandler {
    fn default() -> Self {
        Self::new()
    }
}
