//! Feishu webhook handler.

use std::sync::Arc;

use moltis_channels::plugin::{
    ChannelEventSink, ChannelMessageMeta, ChannelReplyTarget, ChannelType,
};
use tracing::{debug, warn};

use crate::access::check_group_access;
use crate::state::AccountStateMap;

/// Feishu event handler
pub struct Handler {
    pub account_id: String,
    pub accounts: AccountStateMap,
}

impl Handler {
    /// Handle incoming message from Feishu
    pub async fn handle_message(
        &self,
        _event: &serde_json::Value,  // Feishu message event
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        debug!(account_id = %self.account_id, "handling feishu message");

        // Check if account exists
        let account_exists = {
            let accounts = self.accounts.read().unwrap_or_else(|e| e.into_inner());
            accounts.contains_key(&self.account_id)
        };

        if !account_exists {
            warn!(account_id = %self.account_id, "account not found");
            return Ok(());
        };

        // TODO: Parse message and extract relevant fields
        // - message_id (for deduplication)
        // - sender_id
        // - chat_id
        // - message_type (text/image/file)
        // - content

        // TODO: Check group access using check_group_access()

        // TODO: Dispatch to chat via event_sink

        Ok(())
    }
}
