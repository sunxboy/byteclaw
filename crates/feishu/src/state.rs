//! Feishu channel state management.

use std::sync::Arc;
use tokio::sync::RwLock;

/// Shared state for the Feishu channel.
#[derive(Debug, Clone)]
pub struct FeishuState {
    inner: Arc<RwLock<FeishuStateInner>>,
}

#[derive(Debug, Default)]
struct FeishuStateInner {
    connected: bool,
}

impl FeishuState {
    /// Creates a new state instance.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(FeishuStateInner::default())),
        }
    }
}

impl Default for FeishuState {
    fn default() -> Self {
        Self::new()
    }
}
