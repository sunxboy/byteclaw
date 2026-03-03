//! Feishu outbound messaging.

use async_trait::async_trait;
use moltis_channels::{
    plugin::{ChannelHealthSnapshot, ChannelOutbound, ChannelStatus},
    Result as ChannelResult,
};
use moltis_common::types::ReplyPayload;
use open_lark::{
    client::LarkClient,
    core::constants::AppType,
    service::im::v1::message::{CreateMessageRequest, CreateMessageRequestBody},
};
use secrecy::ExposeSecret;
#[cfg(feature = "tracing")]
use tracing::{debug, error, info};

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
        _reply_to: Option<&str>,
    ) -> ChannelResult<()> {
        #[cfg(feature = "tracing")]
        debug!(account_id, to, text_len = text.len(), "sending text message");

        // Get account state to retrieve credentials
        let (app_id, app_secret) = {
            let accounts = self.accounts.read().unwrap_or_else(|e| e.into_inner());
            match accounts.get(account_id) {
                Some(state) => (
                    state.config.app_id.clone(),
                    state.config.app_secret.expose_secret().clone(),
                ),
                None => {
                    return Err(moltis_channels::Error::invalid_input(format!(
                        "feishu account {} not found",
                        account_id
                    )));
                }
            }
        };

        // Create Lark client
        let client = LarkClient::builder(&app_id, &app_secret)
            .with_app_type(AppType::SelfBuild)
            .with_enable_token_cache(true)
            .build();

        // Build message request
        let message = CreateMessageRequestBody::builder()
            .receive_id(to)
            .msg_type("text")
            .content(format!("{{\"text\":\"{}\"}}", escape_json(text)))
            .build();

        let request = CreateMessageRequest::builder()
            .receive_id_type("open_id")
            .request_body(message)
            .build();

        // Send message
        match client.im.v1.message.create(request, None).await {
            Ok(_) => {
                #[cfg(feature = "tracing")]
                info!(account_id, to, "text message sent successfully");
                Ok(())
            }
            Err(e) => {
                #[cfg(feature = "tracing")]
                error!(account_id, to, error = %e, "failed to send text message");
                Err(moltis_channels::Error::external(
                    "feishu",
                    e,
                ))
            }
        }
    }

    async fn send_media(
        &self,
        account_id: &str,
        to: &str,
        payload: &ReplyPayload,
        _reply_to: Option<&str>,
    ) -> ChannelResult<()> {
        #[cfg(feature = "tracing")]
        debug!(account_id, to, "sending media");

        // Get account state to retrieve credentials
        let (app_id, app_secret) = {
            let accounts = self.accounts.read().unwrap_or_else(|e| e.into_inner());
            match accounts.get(account_id) {
                Some(state) => (
                    state.config.app_id.clone(),
                    state.config.app_secret.expose_secret().clone(),
                ),
                None => {
                    return Err(moltis_channels::Error::invalid_input(format!(
                        "feishu account {} not found",
                        account_id
                    )));
                }
            }
        };

        // Create Lark client
        let client = LarkClient::builder(&app_id, &app_secret)
            .with_app_type(AppType::SelfBuild)
            .with_enable_token_cache(true)
            .build();

        // Build message content based on payload
        let content = if let Some(ref media) = payload.media {
            format!(
                "{{\"text\":\"[Media: {} - {}]\"}}",
                escape_json(&media.mime_type),
                escape_json(&media.url)
            )
        } else {
            format!("{{\"text\":\"{}\"}}", escape_json(&payload.text))
        };

        // Build message request
        let message = CreateMessageRequestBody::builder()
            .receive_id(to)
            .msg_type("text")
            .content(content)
            .build();

        let request = CreateMessageRequest::builder()
            .receive_id_type("open_id")
            .request_body(message)
            .build();

        // Send message
        match client.im.v1.message.create(request, None).await {
            Ok(_) => {
                #[cfg(feature = "tracing")]
                info!(account_id, to, "media message sent successfully");
                Ok(())
            }
            Err(e) => {
                #[cfg(feature = "tracing")]
                error!(account_id, to, error = %e, "failed to send media message");
                Err(moltis_channels::Error::external(
                    "feishu",
                    e,
                ))
            }
        }
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
        if let Some(state) = accounts.get(account_id) {
            // Check if cancellation token is cancelled
            if state.cancel.is_cancelled() {
                Ok(ChannelHealthSnapshot {
                    connected: false,
                    account_id: account_id.to_string(),
                    details: Some("connection cancelled".into()),
                })
            } else {
                Ok(ChannelHealthSnapshot {
                    connected: true,
                    account_id: account_id.to_string(),
                    details: Some("connected".into()),
                })
            }
        } else {
            Ok(ChannelHealthSnapshot {
                connected: false,
                account_id: account_id.to_string(),
                details: Some("account not started".into()),
            })
        }
    }
}

/// Escape special characters for JSON string
fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}
