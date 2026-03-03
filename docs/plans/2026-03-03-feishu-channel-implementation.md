# Feishu Channel 实现计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 实现飞书 channel 支持，允许用户通过飞书机器人与 moltis Agent 交互

**Architecture:** 新增 `moltis-feishu` crate，使用 `open-lark` SDK 处理 WebSocket 连接和飞书 API，实现 ChannelPlugin trait 集成到现有 channel 架构

**Tech Stack:** Rust, open-lark, tokio, serde, secrecy

---

## 前置检查

### Task 0: 评估 open-lark crate

**Files:**
- Check: `https://docs.rs/open-lark/latest/open_lark/`
- Check: `https://crates.io/crates/open-lark`

**步骤:**
1. 查看 open-lark crate 的最新版本和文档
2. 确认支持的功能：WebSocket、事件分发、消息收发
3. 记录关键 API 使用方式

**验证:** 确认 crate 版本和功能满足需求

---

## Phase 1: 基础 Crate 结构

### Task 1: 创建 feishu crate 目录和 Cargo.toml

**Files:**
- Create: `crates/feishu/Cargo.toml`
- Create: `crates/feishu/src/lib.rs`

**步骤 1: 创建目录结构**

```bash
mkdir -p crates/feishu/src
```

**步骤 2: 编写 Cargo.toml**

参考 `crates/discord/Cargo.toml` 结构：

```toml
[package]
name = "moltis-feishu"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true

[dependencies]
# 飞书 SDK
open-lark = "0.3"

# Workspace deps
async-trait = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
secrecy = { workspace = true }
tokio = { workspace = true }
tracing = { workspace = true }
thiserror = { workspace = true }

# Internal crates
moltis-channels = { workspace = true }
moltis-common = { workspace = true }

[features]
default = []
tracing = ["moltis-channels/tracing"]
metrics = ["moltis-channels/metrics"]
```

**步骤 3: 编写基础 lib.rs**

```rust
//! Feishu (Lark) channel plugin.

pub mod config;
pub mod error;
pub mod handler;
pub mod outbound;
pub mod plugin;
pub mod state;
pub mod access;

pub use plugin::FeishuPlugin;
```

**步骤 4: 添加到 workspace**

修改根目录 `Cargo.toml`：

```toml
[workspace]
members = [
    # ... existing crates
    "crates/feishu",  # ADD THIS
]
```

**步骤 5: 验证编译**

```bash
cargo check -p moltis-feishu
```

Expected: 编译成功（可能有未使用代码警告）

**步骤 6: Commit**

```bash
git add crates/feishu/ Cargo.toml
git commit -m "feat(feishu): create crate structure"
```

---

### Task 2: 实现 Error 类型

**Files:**
- Create: `crates/feishu/src/error.rs`

**步骤 1: 编写 error.rs**

参考 `crates/discord/src/error.rs`：

```rust
use moltis_channels::Error as ChannelError;

#[derive(Debug, thiserror::Error)]
pub enum FeishuError {
    #[error("invalid configuration: {0}")]
    InvalidConfig(String),
    #[error("API error: {0}")]
    ApiError(String),
    #[error("WebSocket error: {0}")]
    WebSocketError(String),
    #[error("authentication failed: {0}")]
    AuthFailed(String),
}

impl From<FeishuError> for ChannelError {
    fn from(err: FeishuError) -> Self {
        ChannelError::plugin_error("feishu", err.to_string())
    }
}
```

**步骤 2: 验证编译**

```bash
cargo check -p moltis-feishu
```

**步骤 3: Commit**

```bash
git add crates/feishu/src/error.rs
git commit -m "feat(feishu): add error types"
```

---

### Task 3: 实现 Config 结构

**Files:**
- Create: `crates/feishu/src/config.rs`

**步骤 1: 编写 config.rs**

```rust
use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize};

/// Feishu account configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeishuAccountConfig {
    /// Feishu app ID
    pub app_id: String,
    /// Feishu app secret
    pub app_secret: Secret<String>,
    /// Connection mode (only websocket supported)
    #[serde(default)]
    pub connection_mode: ConnectionMode,
    /// Group chat policy
    #[serde(default)]
    pub group_policy: GroupPolicy,
    /// Allowlist of group chat IDs (for allowlist policy)
    #[serde(default)]
    pub group_allowlist: Vec<String>,
    /// OTP cooldown in seconds
    #[serde(default = "default_otp_cooldown")]
    pub otp_cooldown_secs: u64,
    /// Optional encryption key for message verification
    #[serde(default)]
    pub encrypt_key: Option<Secret<String>>,
}

impl Default for FeishuAccountConfig {
    fn default() -> Self {
        Self {
            app_id: String::new(),
            app_secret: Secret::new(String::new()),
            connection_mode: ConnectionMode::default(),
            group_policy: GroupPolicy::default(),
            group_allowlist: Vec::new(),
            otp_cooldown_secs: default_otp_cooldown(),
            encrypt_key: None,
        }
    }
}

fn default_otp_cooldown() -> u64 {
    3600 // 1 hour
}

/// Connection mode for Feishu
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionMode {
    /// WebSocket long connection (default and only supported mode)
    #[default]
    Websocket,
}

/// Group chat policy
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum GroupPolicy {
    /// Only respond in allowlisted groups
    Allowlist,
    /// Respond in all groups
    Open,
    /// Only respond in direct messages (no groups)
    #[default]
    Closed,
}

#[cfg(test)]
mod tests {
    use super::*;
    use secrecy::ExposeSecret;

    #[test]
    fn config_deserialize() {
        let json = r#"{
            "app_id": "cli_xxx",
            "app_secret": "secret_xxx",
            "group_policy": "allowlist",
            "group_allowlist": ["oc_xxx"]
        }"#;
        let cfg: FeishuAccountConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.app_id, "cli_xxx");
        assert_eq!(cfg.app_secret.expose_secret(), "secret_xxx");
        assert_eq!(cfg.group_policy, GroupPolicy::Allowlist);
        assert_eq!(cfg.group_allowlist, vec!["oc_xxx"]);
    }

    #[test]
    fn config_default() {
        let cfg = FeishuAccountConfig::default();
        assert_eq!(cfg.connection_mode, ConnectionMode::Websocket);
        assert_eq!(cfg.group_policy, GroupPolicy::Closed);
        assert_eq!(cfg.otp_cooldown_secs, 3600);
    }
}
```

**步骤 2: 运行测试**

```bash
cargo test -p moltis-feishu
```

Expected: 两个测试都通过

**步骤 3: Commit**

```bash
git add crates/feishu/src/config.rs
git commit -m "feat(feishu): add config types with tests"
```

---

## Phase 2: ChannelType 集成

### Task 4: 扩展 ChannelType 枚举

**Files:**
- Modify: `crates/channels/src/plugin.rs:10-16`

**步骤 1: 修改 ChannelType 枚举**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChannelType {
    Telegram,
    Whatsapp,
    #[serde(rename = "msteams")]
    MsTeams,
    Discord,
    Feishu,  // ADD THIS
}
```

**步骤 2: 更新 as_str() 方法**

```rust
impl ChannelType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Telegram => "telegram",
            Self::Whatsapp => "whatsapp",
            Self::MsTeams => "msteams",
            Self::Discord => "discord",
            Self::Feishu => "feishu",  // ADD THIS
        }
    }
    // ...
}
```

**步骤 3: 更新 display_name() 方法**

```rust
pub fn display_name(&self) -> &'static str {
    match self {
        Self::Telegram => "Telegram",
        Self::Whatsapp => "WhatsApp",
        Self::MsTeams => "Microsoft Teams",
        Self::Discord => "Discord",
        Self::Feishu => "Feishu",  // ADD THIS
    }
}
```

**步骤 4: 更新 FromStr 实现**

```rust
impl std::str::FromStr for ChannelType {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "telegram" => Ok(Self::Telegram),
            "whatsapp" => Ok(Self::Whatsapp),
            "msteams" | "microsoft_teams" | "microsoft-teams" | "teams" => Ok(Self::MsTeams),
            "discord" => Ok(Self::Discord),
            "feishu" | "lark" => Ok(Self::Feishu),  // ADD THIS
            other => Err(Error::invalid_input(format!(
                "unknown channel type: {other}"
            ))),
        }
    }
}
```

**步骤 5: 添加测试**

在 `mod tests` 中添加：

```rust
#[test]
fn channel_type_feishu_roundtrip() {
    let ct = ChannelType::Feishu;
    assert_eq!(ct.as_str(), "feishu");
    assert_eq!(ct.to_string(), "feishu");
    assert_eq!("feishu".parse::<ChannelType>().unwrap(), ct);
    assert_eq!("lark".parse::<ChannelType>().unwrap(), ct);
}
```

**步骤 6: 运行测试**

```bash
cargo test -p moltis-channels channel_type
```

Expected: 所有测试通过

**步骤 7: Commit**

```bash
git add crates/channels/src/plugin.rs
git commit -m "feat(channels): add Feishu to ChannelType"
```

---

### Task 5: 扩展 ChannelsConfig

**Files:**
- Modify: `crates/config/src/schema.rs:1136-1160`

**步骤 1: 添加 feishu 字段到 ChannelsConfig**

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChannelsConfig {
    #[serde(
        default = "default_channels_offered",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub offered: Vec<String>,
    #[serde(default)]
    pub telegram: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub whatsapp: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub msteams: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub discord: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub feishu: HashMap<String, serde_json::Value>,  // ADD THIS
}
```

**步骤 2: 更新 Default 实现**

```rust
impl Default for ChannelsConfig {
    fn default() -> Self {
        Self {
            offered: default_channels_offered(),
            telegram: HashMap::new(),
            whatsapp: HashMap::new(),
            msteams: HashMap::new(),
            discord: HashMap::new(),
            feishu: HashMap::new(),  // ADD THIS
        }
    }
}
```

**步骤 3: 更新 validate.rs**

检查 `crates/config/src/validate.rs` 中是否有 channels 相关验证，添加 feishu 支持：

```bash
grep -n "telegram\|discord\|msteams" crates/config/src/validate.rs | head -20
```

如果需要，在 `build_schema_map()` 中添加 feishu 字段映射。

**步骤 4: 验证编译**

```bash
cargo check -p moltis-config
```

**步骤 5: Commit**

```bash
git add crates/config/src/schema.rs
git commit -m "feat(config): add feishu field to ChannelsConfig"
```

---

## Phase 3: FeishuPlugin 核心实现

### Task 6: 实现 State 管理

**Files:**
- Create: `crates/feishu/src/state.rs`

**步骤 1: 编写 state.rs**

参考 `crates/discord/src/state.rs`：

```rust
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

pub type AccountStateMap = Arc<std::sync::RwLock<HashMap<String, AccountState>>>;
```

**步骤 2: 验证编译**

```bash
cargo check -p moltis-feishu
```

**步骤 3: Commit**

```bash
git add crates/feishu/src/state.rs
git commit -m "feat(feishu): add account state management"
```

---

### Task 7: 实现 Access 控制

**Files:**
- Create: `crates/feishu/src/access.rs`

**步骤 1: 编写 access.rs**

```rust
use crate::config::GroupPolicy;

/// Check if a group chat is allowed to interact with the bot
pub fn check_group_access(
    policy: GroupPolicy,
    allowlist: &[String],
    chat_id: &str,
    is_direct_message: bool,
) -> bool {
    match policy {
        // In closed policy, only direct messages are allowed
        GroupPolicy::Closed => is_direct_message,
        // In open policy, all chats are allowed
        GroupPolicy::Open => true,
        // In allowlist policy, only allowlisted groups and DMs are allowed
        GroupPolicy::Allowlist => {
            is_direct_message || allowlist.contains(&chat_id.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closed_policy_allows_only_dm() {
        assert!(check_group_access(GroupPolicy::Closed, &[], "chat1", true));
        assert!(!check_group_access(GroupPolicy::Closed, &[], "chat1", false));
    }

    #[test]
    fn open_policy_allows_all() {
        assert!(check_group_access(GroupPolicy::Open, &[], "chat1", true));
        assert!(check_group_access(GroupPolicy::Open, &[], "chat1", false));
        assert!(check_group_access(GroupPolicy::Open, &[], "any", false));
    }

    #[test]
    fn allowlist_policy() {
        let allowlist = vec!["chat1".to_string(), "chat2".to_string()];
        // DMs always allowed
        assert!(check_group_access(GroupPolicy::Allowlist, &allowlist, "chat1", true));
        // Allowlisted groups allowed
        assert!(check_group_access(GroupPolicy::Allowlist, &allowlist, "chat1", false));
        assert!(check_group_access(GroupPolicy::Allowlist, &allowlist, "chat2", false));
        // Non-allowlisted groups denied
        assert!(!check_group_access(GroupPolicy::Allowlist, &allowlist, "chat3", false));
    }
}
```

**步骤 2: 运行测试**

```bash
cargo test -p moltis-feishu access
```

Expected: 所有测试通过

**步骤 3: Commit**

```bash
git add crates/feishu/src/access.rs
git commit -m "feat(feishu): add group access control with tests"
```

---

### Task 8: 实现 Handler（事件处理）

**Files:**
- Create: `crates/feishu/src/handler.rs`

**步骤 1: 编写 handler.rs（基础结构）**

```rust
use std::sync::Arc;

use moltis_channels::plugin::{
    ChannelAttachment, ChannelEvent, ChannelEventSink, ChannelMessageKind, ChannelMessageMeta,
    ChannelReplyTarget, ChannelType,
};
use tracing::{debug, warn};

use crate::access::check_group_access;
use crate::config::GroupPolicy;
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
        message: &serde_json::Value,  // Feishu message event
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        debug!(account_id = %self.account_id, "handling feishu message");

        // TODO: Parse message and extract relevant fields
        // - message_id (for deduplication)
        // - sender_id
        // - chat_id
        // - message_type (text/image/file)
        // - content

        // Get account state
        let state = {
            let accounts = self.accounts.read().unwrap_or_else(|e| e.into_inner());
            accounts.get(&self.account_id).cloned()
        };

        let Some(state) = state else {
            warn!(account_id = %self.account_id, "account not found");
            return Ok(());
        };

        // TODO: Check group access
        // let is_dm = ...;
        // if !check_group_access(state.config.group_policy, &state.config.group_allowlist, chat_id, is_dm) {
        //     return Ok(());
        // }

        // TODO: Dispatch to chat
        // if let Some(ref sink) = state.event_sink {
        //     sink.dispatch_to_chat(...).await;
        // }

        Ok(())
    }
}
```

**步骤 2: 验证编译**

```bash
cargo check -p moltis-feishu
```

**步骤 3: Commit**

```bash
git add crates/feishu/src/handler.rs
git commit -m "feat(feishu): add handler skeleton"
```

---

### Task 9: 实现 Outbound（消息发送）

**Files:**
- Create: `crates/feishu/src/outbound.rs`

**步骤 1: 编写 outbound.rs（基础结构）**

```rust
use async_trait::async_trait;
use moltis_channels::{
    plugin::{ChannelHealthSnapshot, ChannelOutbound, ChannelStatus, ChannelStreamOutbound},
    Result as ChannelResult,
};
use moltis_common::types::ReplyPayload;
use tracing::debug;

use crate::state::AccountStateMap;

/// Feishu outbound adapter
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
        debug!(account_id, to, "sending text message");
        // TODO: Implement using open-lark API
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
```

**步骤 2: 验证编译**

```bash
cargo check -p moltis-feishu
```

**步骤 3: Commit**

```bash
git add crates/feishu/src/outbound.rs
git commit -m "feat(feishu): add outbound adapter skeleton"
```

---

### Task 10: 实现 Plugin（核心 ChannelPlugin trait）

**Files:**
- Create: `crates/feishu/src/plugin.rs`

**步骤 1: 编写 plugin.rs（基础结构）**

```rust
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use moltis_channels::{
    message_log::MessageLog,
    plugin::{
        ChannelEventSink, ChannelHealthSnapshot, ChannelOutbound, ChannelPlugin, ChannelStatus,
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

/// Feishu channel plugin
pub struct FeishuPlugin {
    accounts: AccountStateMap,
    outbound: FeishuOutbound,
    message_log: Option<Arc<dyn MessageLog>>,
    event_sink: Option<Arc<dyn ChannelEventSink>>,
}

impl FeishuPlugin {
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

    pub fn with_message_log(mut self, log: Arc<dyn MessageLog>) -> Self {
        self.message_log = Some(log);
        self
    }

    pub fn with_event_sink(mut self, sink: Arc<dyn ChannelEventSink>) -> Self {
        self.event_sink = Some(sink);
        self
    }

    pub fn shared_outbound(&self) -> Arc<dyn ChannelOutbound> {
        Arc::new(FeishuOutbound {
            accounts: Arc::clone(&self.accounts),
        })
    }

    pub fn account_ids(&self) -> Vec<String> {
        let accounts = self.accounts.read().unwrap_or_else(|e| e.into_inner());
        accounts.keys().cloned().collect()
    }

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
        let cancel = {
            let mut accounts = self.accounts.write().unwrap_or_else(|e| e.into_inner());
            accounts.remove(account_id).map(|s| s.cancel)
        };
        if let Some(cancel) = cancel {
            cancel.cancel();
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
```

**步骤 2: 验证编译**

```bash
cargo check -p moltis-feishu
```

**步骤 3: Commit**

```bash
git add crates/feishu/src/plugin.rs
git commit -m "feat(feishu): add plugin skeleton with ChannelPlugin impl"
```

---

## Phase 4: Gateway 集成

### Task 11: 集成 FeishuPlugin 到 LiveChannelService

**Files:**
- Modify: `crates/gateway/src/channel.rs`

**步骤 1: 添加 feishu 导入**

```rust
use {
    moltis_channels::{
        ChannelOutbound, ChannelPlugin, ChannelType,
        message_log::MessageLog,
        plugin::ChannelHealthSnapshot,
        store::{ChannelStore, StoredChannel},
    },
    moltis_discord::DiscordPlugin,
    moltis_msteams::MsTeamsPlugin,
    moltis_sessions::metadata::SqliteSessionMetadata,
    moltis_telegram::TelegramPlugin,
    moltis_feishu::FeishuPlugin,  // ADD THIS
};
```

**步骤 2: 修改 LiveChannelService 结构体**

```rust
pub struct LiveChannelService {
    telegram: Arc<RwLock<TelegramPlugin>>,
    msteams: Arc<RwLock<MsTeamsPlugin>>,
    discord: Arc<RwLock<DiscordPlugin>>,
    feishu: Arc<RwLock<FeishuPlugin>>,  // ADD THIS
    #[cfg(feature = "whatsapp")]
    whatsapp: Arc<RwLock<WhatsAppPlugin>>,
    outbound: Arc<dyn ChannelOutbound>,
    store: Arc<dyn ChannelStore>,
    message_log: Arc<dyn MessageLog>,
    session_metadata: Arc<SqliteSessionMetadata>,
}
```

**步骤 3: 修改 new() 方法**

```rust
impl LiveChannelService {
    pub fn new(
        telegram: Arc<RwLock<TelegramPlugin>>,
        msteams: Arc<RwLock<MsTeamsPlugin>>,
        discord: Arc<RwLock<DiscordPlugin>>,
        feishu: Arc<RwLock<FeishuPlugin>>,  // ADD THIS
        #[cfg(feature = "whatsapp")] whatsapp: Arc<RwLock<WhatsAppPlugin>>,
        outbound: Arc<dyn ChannelOutbound>,
        store: Arc<dyn ChannelStore>,
        message_log: Arc<dyn MessageLog>,
        session_metadata: Arc<SqliteSessionMetadata>,
    ) -> Self {
        Self {
            telegram,
            msteams,
            discord,
            feishu,  // ADD THIS
            #[cfg(feature = "whatsapp")]
            whatsapp,
            outbound,
            store,
            message_log,
            session_metadata,
        }
    }
    // ...
}
```

**步骤 4: 修改 resolve_channel_type**

在方法中添加 feishu 检查：

```rust
async fn resolve_channel_type(
    &self,
    params: &Value,
    account_id: &str,
    default_when_unknown: ChannelType,
) -> Result<ChannelType, String> {
    // ... existing checks for telegram, msteams, discord ...

    {
        let fs = self.feishu.read().await;
        if fs.has_account(account_id) {
            matches.push(ChannelType::Feishu);
        }
    }

    // ... rest of method ...
}
```

**步骤 5: 验证编译**

```bash
cargo check -p moltis-gateway
```

**步骤 6: Commit**

```bash
git add crates/gateway/src/channel.rs
git commit -m "feat(gateway): integrate FeishuPlugin into LiveChannelService"
```

---

### Task 12: 更新 server.rs 创建 FeishuPlugin

**Files:**
- Modify: `crates/gateway/src/server.rs`

**步骤 1: 找到 plugin 初始化代码**

搜索现有的 plugin 创建代码：

```bash
grep -n "TelegramPlugin::new\|DiscordPlugin::new" crates/gateway/src/server.rs
```

**步骤 2: 添加 FeishuPlugin 创建**

在适当位置添加：

```rust
let feishu_plugin = Arc::new(RwLock::new(
    FeishuPlugin::new()
        .with_message_log(Arc::new(message_log.clone()))
        .with_event_sink(Arc::new(channel_events.clone())),
));
```

**步骤 3: 修改 LiveChannelService 创建**

找到 `LiveChannelService::new()` 调用，添加 feishu 参数：

```rust
let channel_service = LiveChannelService::new(
    telegram_plugin,
    msteams_plugin,
    discord_plugin,
    feishu_plugin,  // ADD THIS
    #[cfg(feature = "whatsapp")]
    whatsapp_plugin,
    // ... other params
);
```

**步骤 4: 验证编译**

```bash
cargo check -p moltis-gateway --all-features
```

**步骤 5: Commit**

```bash
git add crates/gateway/src/server.rs
git commit -m "feat(gateway): create FeishuPlugin instance in server"
```

---

### Task 13: 更新 CLI 默认 offered channels

**Files:**
- Check: `crates/cli/` 或配置初始化代码

检查是否在代码中硬编码了默认 channel：

```bash
grep -rn "telegram.*discord" crates/cli/ crates/config/ --include="*.rs" | head -10
```

如果需要，更新默认 offered channels 包含 feishu：

```rust
// 在 default_channels_offered() 函数中
fn default_channels_offered() -> Vec<String> {
    vec!["telegram".into(), "discord".into(), "feishu".into()]
}
```

**Commit:**

```bash
git commit -m "feat(config): add feishu to default offered channels"
```

---

## Phase 5: 完整实现（集成 open-lark）

### Task 14: 研究 open-lark API

**Files:**
- Read: `https://docs.rs/open-lark/latest/open_lark/`

**步骤 1: 研究关键 API**

- WebSocket 客户端创建
- 事件订阅和处理
- 消息发送 API
- 认证机制

**步骤 2: 更新 Cargo.toml 添加必要依赖**

根据 open-lark 的要求，可能需要添加：

```toml
[dependencies]
open-lark = "0.3"
# 可能需要的额外依赖
reqwest = { workspace = true }
```

**步骤 3: Commit**

```bash
git add crates/feishu/Cargo.toml
git commit -m "chore(feishu): update dependencies for open-lark integration"
```

---

### Task 15: 实现 WebSocket 连接

**Files:**
- Modify: `crates/feishu/src/plugin.rs`

**步骤 1: 实现 start_account 中的 WebSocket 连接**

```rust
async fn start_account(
    &mut self,
    account_id: &str,
    config: serde_json::Value,
) -> ChannelResult<()> {
    let cfg: FeishuAccountConfig = serde_json::from_value(config)?;
    // ... validation ...

    let cancel = tokio_util::sync::CancellationToken::new();
    let accounts_clone = Arc::clone(&self.accounts);
    let account_id_owned = account_id.to_string();
    let app_id = cfg.app_id.clone();
    let app_secret = cfg.app_secret.expose_secret().clone();

    {
        let otp_cooldown = cfg.otp_cooldown_secs;
        let mut accounts = self.accounts.write().unwrap_or_else(|e| e.into_inner());
        accounts.insert(account_id.to_string(), AccountState {
            account_id: account_id.to_string(),
            config: cfg,
            message_log: self.message_log.clone(),
            event_sink: self.event_sink.clone(),
            cancel: cancel.clone(),
            otp: std::sync::Mutex::new(moltis_channels::otp::OtpState::new(otp_cooldown)),
            tenant_key: None,
        });
    }

    // Spawn WebSocket connection task
    let cancel_for_task = cancel.clone();
    tokio::spawn(async move {
        // TODO: Initialize open-lark client and start event loop
        // - Create client with app_id and app_secret
        // - Subscribe to im.message.receive_v1 events
        // - Handle events with Handler
    });

    Ok(())
}
```

**步骤 2: 验证编译**

```bash
cargo check -p moltis-feishu
```

**步骤 3: Commit**

```bash
git commit -m "feat(feishu): implement WebSocket connection in start_account"
```

---

### Task 16: 实现消息接收处理

**Files:**
- Modify: `crates/feishu/src/handler.rs`

**步骤 1: 实现完整的消息处理逻辑**

```rust
impl Handler {
    pub async fn handle_message(
        &self,
        event: &serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Parse Feishu event structure
        // Extract:
        // - sender.sender_id.open_id
        // - message.chat_id
        // - message.message_id
        // - message.message_type (text/image/file/post)
        // - message.content (JSON string)

        // Deduplication check

        // Group access check

        // Dispatch to chat via event_sink

        Ok(())
    }
}
```

**步骤 2: 验证编译**

```bash
cargo check -p moltis-feishu
```

**步骤 3: Commit**

```bash
git commit -m "feat(feishu): implement message handling logic"
```

---

### Task 17: 实现消息发送

**Files:**
- Modify: `crates/feishu/src/outbound.rs`

**步骤 1: 实现 send_text**

```rust
async fn send_text(
    &self,
    account_id: &str,
    to: &str,
    text: &str,
    reply_to: Option<&str>,
) -> ChannelResult<()> {
    // Get account state
    // Create open-lark client
    // Call send message API
    // Handle reply_to if present
    Ok(())
}
```

**步骤 2: 实现 send_media**

```rust
async fn send_media(
    &self,
    account_id: &str,
    to: &str,
    payload: &ReplyPayload,
    reply_to: Option<&str>,
) -> ChannelResult<()> {
    // Handle different payload types (image, file)
    Ok(())
}
```

**步骤 3: 验证编译**

```bash
cargo check -p moltis-feishu
```

**步骤 4: Commit**

```bash
git commit -m "feat(feishu): implement outbound message sending"
```

---

## Phase 6: 测试和验证

### Task 18: 添加单元测试

**Files:**
- Modify: `crates/feishu/src/` 各文件

**步骤 1: 为各模块添加测试**

确保以下模块都有充分的单元测试：
- config.rs - 配置解析测试
- access.rs - 权限控制测试
- handler.rs - 消息处理测试（使用 mock）
- outbound.rs - 发送逻辑测试（使用 mock）

**步骤 2: 运行所有测试**

```bash
cargo test -p moltis-feishu
```

Expected: 所有测试通过

**步骤 3: Commit**

```bash
git commit -m "test(feishu): add comprehensive unit tests"
```

---

### Task 19: 验证集成编译

**步骤 1: 编译整个 workspace**

```bash
cargo build --workspace
```

**步骤 2: 运行 clippy**

```bash
cargo clippy -p moltis-feishu -- -D warnings
```

**步骤 3: Commit 修复**

```bash
git commit -m "fix(feishu): address clippy warnings"
```

---

### Task 20: 编写文档

**Files:**
- Create: `docs/src/channels/feishu.md`

**步骤 1: 编写使用文档**

```markdown
# Feishu Channel

配置飞书机器人与 moltis 集成。

## 配置

```toml
[channels]
offered = ["telegram", "discord", "feishu"]

[channels.feishu.my-bot]
app_id = "cli_xxx"
app_secret = "xxx"
connection_mode = "websocket"
group_policy = "allowlist"
group_allowlist = ["oc_xxx"]
otp_cooldown_secs = 3600
```

## 获取凭证

1. 访问 [飞书开放平台](https://open.feishu.cn/)
2. 创建企业自建应用
3. 获取 App ID 和 App Secret
4. 订阅 `im.message.receive_v1` 事件
5. 发布应用

## 群组策略

- `allowlist`: 只在指定群组响应
- `open`: 在所有群组响应
- `closed`: 仅私聊
```

**步骤 2: 更新 SUMMARY.md**

```bash
# 添加到 docs/src/SUMMARY.md
```

**步骤 3: Commit**

```bash
git add docs/
git commit -m "docs: add feishu channel documentation"
```

---

## 总结

完成以上所有任务后，飞书 channel 将完全集成到 moltis 中，用户可以：

1. 在配置中添加飞书机器人账号
2. 通过飞书与 moltis Agent 进行对话
3. 使用群组策略控制访问权限
4. 在 Web UI 中查看和管理飞书 channel

**下一步：** 使用 `superpowers:executing-plans` skill 逐个执行上述任务。
