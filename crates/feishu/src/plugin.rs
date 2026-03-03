//! Feishu channel plugin.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use moltis_channels::{
    message_log::MessageLog,
    otp::OtpState,
    plugin::{
        ChannelEventSink, ChannelOutbound, ChannelPlugin, ChannelStatus,
    },
    Result as ChannelResult,
};
use open_lark::{
    client::ws_client::LarkWsClient,
    event::dispatcher::EventDispatcherHandler,
    service::im::v1::p2_im_message_receive_v1::P2ImMessageReceiveV1,
};
use secrecy::ExposeSecret;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

use crate::{
    config::FeishuAccountConfig,
    handler::Handler,
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

/// Run WebSocket connection for a Feishu account
async fn run_websocket(
    app_id: String,
    app_secret: String,
    account_id: String,
    accounts: AccountStateMap,
    cancel: CancellationToken,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Create event dispatcher with message handler
    let handler = {
        let account_id = account_id.clone();
        let accounts = Arc::clone(&accounts);

        EventDispatcherHandler::builder()
            .register_p2_im_message_receive_v1(move |event: P2ImMessageReceiveV1| {
                // Spawn a new task to handle the message
                let handler = Handler {
                    account_id: account_id.clone(),
                    accounts: Arc::clone(&accounts),
                };

                tokio::spawn(async move {
                    if let Err(e) = handler.handle_event(event).await {
                        error!(account_id = %handler.account_id, error = %e, "failed to handle message");
                    }
                });
            })
            .build()
    };

    info!(account_id = %account_id, "starting feishu websocket connection");

    // Start WebSocket connection with cancellation support
    tokio::select! {
        result = LarkWsClient::open(&app_id, &app_secret, handler) => {
            match result {
                Ok(()) => {
                    info!(account_id = %account_id, "websocket connection closed");
                }
                Err(e) => {
                    return Err(Box::new(e) as Box<dyn std::error::Error + Send + Sync>);
                }
            }
        }
        _ = cancel.cancelled() => {
            info!(account_id = %account_id, "websocket connection cancelled");
        }
    }

    Ok(())
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

        let cancel = CancellationToken::new();
        let accounts_clone = Arc::clone(&self.accounts);
        let account_id_owned = account_id.to_string();
        let app_id = cfg.app_id.clone();
        let app_secret = cfg.app_secret.expose_secret().clone();

        // Store account state
        {
            let otp_cooldown = cfg.otp_cooldown_secs;
            let mut accounts = self.accounts.write().unwrap_or_else(|e| e.into_inner());
            accounts.insert(account_id.to_string(), AccountState {
                account_id: account_id.to_string(),
                config: cfg,
                message_log: self.message_log.clone(),
                event_sink: self.event_sink.clone(),
                cancel: cancel.clone(),
                otp: Arc::new(std::sync::Mutex::new(OtpState::new(otp_cooldown))),
                tenant_key: None,
            });
        }

        // Spawn WebSocket connection task in a blocking thread
        // because open-lark's EventDispatcherHandler is not Send
        let cancel_for_task = cancel.clone();
        let account_id_for_task = account_id.to_string();
        std::thread::spawn(move || {
            let rt = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build() {
                Ok(rt) => rt,
                Err(e) => {
                    error!(account_id = %account_id_for_task, error = %e, "failed to create tokio runtime");
                    return;
                }
            };

            rt.block_on(async {
                if let Err(e) = run_websocket(
                    app_id,
                    app_secret,
                    account_id_owned,
                    accounts_clone,
                    cancel_for_task,
                ).await {
                    error!(account_id = %account_id_for_task, error = %e, "feishu websocket error");
                }
            });
        });

        Ok(())
    }

    async fn stop_account(&mut self, account_id: &str) -> ChannelResult<()> {
        let cancel = {
            let mut accounts = self.accounts.write().unwrap_or_else(|e| e.into_inner());
            accounts.remove(account_id).map(|s| s.cancel)
        };
        if let Some(cancel) = cancel {
            cancel.cancel();
            info!(account_id, "feishu account stopped");
        } else {
            warn!(account_id, "feishu account not found");
        }
        Ok(())
    }

    fn outbound(&self) -> Option<&dyn ChannelOutbound> {
        Some(&self.outbound)
    }

    fn status(&self) -> Option<&dyn ChannelStatus> {
        Some(&self.outbound)
    }
}
