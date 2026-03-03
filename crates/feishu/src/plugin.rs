//! Feishu channel plugin.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use moltis_channels::{
    message_log::MessageLog,
    plugin::{
        ChannelEventSink, ChannelOutbound, ChannelPlugin, ChannelStatus,
    },
    Result as ChannelResult,
};
use secrecy::ExposeSecret;
use tracing::{info, warn};

use crate::{
    config::FeishuAccountConfig,
    outbound::FeishuOutbound,
    state::{AccountState, AccountStateMap},
};

/// Feishu channel plugin.
pub struct FeishuPlugin {
    accounts: AccountStateMap,
    outbound: FeishuOutbound,
    message_log: Option<Arc<dyn MessageLog>>,
    event_sink: Option<Arc<dyn ChannelEventSink>>,
}

impl FeishuPlugin {
    /// Creates a new plugin instance.
    pub fn new() -> Self {
        let accounts: AccountStateMap = Arc::new(RwLock::new(HashMap::new()));
        let outbound = FeishuOutbound {
            accounts: Arc::clone(&accounts),
        };
        Self {
            accounts,
            outbound,
            message_log: None,
            event_sink: None,
        }
    }

    /// Set message log.
    pub fn with_message_log(mut self, log: Arc<dyn MessageLog>) -> Self {
        self.message_log = Some(log);
        self
    }

    /// Set event sink.
    pub fn with_event_sink(mut self, sink: Arc<dyn ChannelEventSink>) -> Self {
        self.event_sink = Some(sink);
        self
    }

    /// Get shared outbound adapter.
    pub fn shared_outbound(&self) -> Arc<dyn ChannelOutbound> {
        Arc::new(FeishuOutbound {
            accounts: Arc::clone(&self.accounts),
        })
    }

    /// Get all account IDs.
    pub fn account_ids(&self) -> Vec<String> {
        let accounts = self.accounts.read().unwrap_or_else(|e| e.into_inner());
        accounts.keys().cloned().collect()
    }

    /// Check if account exists.
    pub fn has_account(&self, account_id: &str) -> bool {
        let accounts = self.accounts.read().unwrap_or_else(|e| e.into_inner());
        accounts.contains_key(account_id)
    }
}

impl Default for FeishuPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ChannelPlugin for FeishuPlugin {
    fn id(&self) -> &str {
        "feishu"
    }

    fn name(&self) -> &str {
        "Feishu"
    }

    async fn start_account(
        &mut self,
        account_id: &str,
        config: serde_json::Value,
    ) -> ChannelResult<()> {
        let cfg: FeishuAccountConfig = serde_json::from_value(config)?;
        if cfg.app_id.is_empty() || cfg.app_secret.expose_secret().is_empty() {
            return Err(moltis_channels::Error::invalid_input(
                "Feishu app_id and app_secret are required",
            ));
        }

        info!(account_id, "starting feishu account");

        // TODO: Initialize open-lark client and start WebSocket connection

        Ok(())
    }

    async fn stop_account(&mut self, account_id: &str) -> ChannelResult<()> {
        let _ = account_id;
        // TODO: Stop the account
        Ok(())
    }

    fn outbound(&self) -> Option<&dyn ChannelOutbound> {
        Some(&self.outbound)
    }

    fn status(&self) -> Option<&dyn ChannelStatus> {
        Some(&self.outbound)
    }
}
