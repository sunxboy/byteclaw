//! Feishu channel plugin.

use crate::config::FeishuConfig;
use crate::handler::FeishuHandler;
use crate::outbound::FeishuOutbound;
use tracing::info;

/// Feishu channel plugin.
#[derive(Debug, Clone)]
pub struct FeishuPlugin {
    config: FeishuConfig,
    handler: FeishuHandler,
    outbound: FeishuOutbound,
}

impl FeishuPlugin {
    /// Creates a new plugin instance with the given configuration.
    pub fn new(config: FeishuConfig) -> Self {
        info!("Initializing Feishu plugin");
        Self {
            config,
            handler: FeishuHandler::new(),
            outbound: FeishuOutbound::new(),
        }
    }
}
