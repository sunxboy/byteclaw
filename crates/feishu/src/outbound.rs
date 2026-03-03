//! Feishu outbound messaging.

use async_trait::async_trait;
use moltis_channels::{
    plugin::{ChannelHealthSnapshot, ChannelOutbound, ChannelStatus},
    Result as ChannelResult,
};
use moltis_common::types::ReplyPayload;
use tracing::debug;

use crate::state::AccountStateMap;

/// Feishu outbound adapter
#[derive(Debug)]
pub struct FeishuOutbound {
    pub accounts: AccountStateMap,
}

#[async_trait]
impl ChannelOutbound for FeishuOutbound {
    async fn send_text(
        &self,
        account_id: &str,
        to: &str,
        text: &str,
        reply_to: Option<&str>,
    ) -> ChannelResult<()> {
        debug!(account_id, to, text_len = text.len(), "sending text message");
        // TODO: Implement using open-lark API
        let _ = (account_id, to, text, reply_to);
        Ok(())
    }

    async fn send_media(
        &self,
        account_id: &str,
        to: &str,
        payload: &ReplyPayload,
        reply_to: Option<&str>,
    ) -> ChannelResult<()> {
        debug!(account_id, to, "sending media");
        // TODO: Implement
        let _ = (account_id, to, payload, reply_to);
        Ok(())
    }

    async fn send_typing(&self, account_id: &str, to: &str) -> ChannelResult<()> {
        // Feishu doesn't have a typing indicator API
        let _ = (account_id, to);
        Ok(())
    }
}

#[async_trait]
impl ChannelStatus for FeishuOutbound {
    async fn probe(&self, account_id: &str) -> ChannelResult<ChannelHealthSnapshot> {
        let accounts = self.accounts.read().unwrap_or_else(|e| e.into_inner());
        if let Some(_state) = accounts.get(account_id) {
            // TODO: Check actual connection status
            Ok(ChannelHealthSnapshot {
                connected: true,
                account_id: account_id.to_string(),
                details: Some("connected".into()),
            })
        } else {
            Ok(ChannelHealthSnapshot {
                connected: false,
                account_id: account_id.to_string(),
                details: Some("account not started".into()),
            })
        }
    }
}
