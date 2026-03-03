//! Feishu event handler.

use moltis_channels::plugin::{ChannelEvent, ChannelReplyTarget, ChannelType};
use open_lark::service::im::v1::p2_im_message_receive_v1::P2ImMessageReceiveV1;
use tracing::{debug, warn};

use crate::{
    access::check_group_access,
    state::AccountStateMap,
};

/// Feishu event handler
pub struct Handler {
    pub account_id: String,
    pub accounts: AccountStateMap,
}

impl Handler {
    /// Handle incoming message event from Feishu
    pub async fn handle_event(
        &self,
        event: P2ImMessageReceiveV1,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        debug!(account_id = %self.account_id, "handling feishu message event");

        // Get account state
        let state = {
            let accounts = self.accounts.read().unwrap_or_else(|e| e.into_inner());
            accounts.get(&self.account_id).cloned()
        };

        let state = match state {
            Some(s) => s,
            None => {
                warn!(account_id = %self.account_id, "account not found");
                return Ok(());
            }
        };

        // Extract message info from event
        let message = &event.event.message;
        let sender = &event.event.sender;

        let message_id = message.message_id.clone();
        let chat_id = message.chat_id.clone();
        let sender_id = sender.sender_id.open_id.clone();
        let chat_type = message.chat_type.clone();
        let message_type = message.message_type.clone();
        let content = message.content.clone();

        // Check if it's a direct message
        let is_direct_message = chat_type == "p2p";

        // Check group access policy
        if !check_group_access(
            state.config.group_policy,
            &state.config.group_allowlist,
            &chat_id,
            is_direct_message,
        ) {
            debug!(chat_id = %chat_id, "message denied by group policy");
            return Ok(());
        }

        // Parse text content
        let text = self.parse_text_content(&message_type, &content);

        // Build reply target
        let reply_target = ChannelReplyTarget {
            channel_type: ChannelType::Feishu,
            account_id: self.account_id.clone(),
            chat_id,
            message_id: Some(message_id),
        };

        // Dispatch to chat via event_sink
        if let Some(ref event_sink) = state.event_sink {
            // Emit inbound message event for UI updates
            let event = ChannelEvent::InboundMessage {
                channel_type: ChannelType::Feishu,
                account_id: self.account_id.clone(),
                peer_id: sender_id,
                username: None,
                sender_name: None,
                message_count: None,
                access_granted: true,
            };

            event_sink.emit(event).await;

            // Dispatch the actual message text to chat
            event_sink
                .dispatch_to_chat(
                    &text,
                    reply_target,
                    moltis_channels::plugin::ChannelMessageMeta {
                        channel_type: ChannelType::Feishu,
                        sender_name: None,
                        username: None,
                        message_kind: Some(moltis_channels::plugin::ChannelMessageKind::Text),
                        model: None,
                        audio_filename: None,
                    },
                )
                .await;
        } else {
            warn!("no event sink configured");
        }

        Ok(())
    }

    /// Parse message content and extract text
    fn parse_text_content(&self, message_type: &str, content: &str) -> String {
        match message_type {
            "text" => {
                // Text content is a JSON string like {"text": "hello"}
                match serde_json::from_str::<serde_json::Value>(content) {
                    Ok(json) => json
                        .get("text")
                        .and_then(|t| t.as_str())
                        .unwrap_or(content)
                        .to_string(),
                    Err(_) => content.to_string(),
                }
            }
            "image" => "[Image]".to_string(),
            "file" => {
                // File content contains file_name
                match serde_json::from_str::<serde_json::Value>(content) {
                    Ok(json) => {
                        let file_name = json.get("file_name").and_then(|n| n.as_str());
                        format!("[File: {}]", file_name.unwrap_or("unknown"))
                    }
                    Err(_) => "[File]".to_string(),
                }
            }
            _ => format!("[{}: {}]", message_type, content),
        }
    }
}
