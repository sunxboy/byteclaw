//! Feishu channel state management.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use moltis_channels::message_log::MessageLog;
use moltis_channels::otp::OtpState;
use moltis_channels::plugin::ChannelEventSink;
use tokio_util::sync::CancellationToken;

use crate::config::FeishuAccountConfig;

/// Per-account state
pub struct AccountState {
    pub account_id: String,
    pub config: FeishuAccountConfig,
    pub message_log: Option<Arc<dyn MessageLog>>,
    pub event_sink: Option<Arc<dyn ChannelEventSink>>,
    pub cancel: CancellationToken,
    pub otp: Mutex<OtpState>,
    // Feishu-specific fields
    pub tenant_key: Option<String>,
}

impl std::fmt::Debug for AccountState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AccountState")
            .field("account_id", &self.account_id)
            .field("config", &self.config)
            .field("message_log", &self.message_log.is_some())
            .field("event_sink", &self.event_sink.is_some())
            .field("tenant_key", &self.tenant_key)
            .finish_non_exhaustive()
    }
}

pub type AccountStateMap = Arc<std::sync::RwLock<HashMap<String, AccountState>>>;
