# Feishu Channel 设计文档

**日期**: 2026-03-03
**作者**: sunxianbing
**状态**: 已批准

---

## 1. 概述

为 moltis 添加飞书（Feishu/Lark）channel 支持，允许用户通过飞书机器人与 moltis Agent 进行交互。

### 1.1 参考实现

- **OpenClaw**: Python 实现的完整飞书支持（WebSocket + Webhook 双模式）
- **open-lark**: Rust 实现的飞书 SDK（WebSocket 长连接、事件分发、消息收发）

### 1.2 设计原则

- 与现有 channel（Discord/Telegram）架构保持一致
- 使用 open-lark crate 减少重复开发
- MVP 优先：先实现基础功能，后续迭代增强

---

## 2. 架构设计

### 2.1 整体架构

```
┌─────────────────────────────────────────────────────────────────┐
│                        Gateway Layer                            │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │   Telegram  │  │   Discord   │  │   Feishu (NEW)          │  │
│  │   Plugin    │  │   Plugin    │  │   Plugin                │  │
│  └──────┬──────┘  └──────┬──────┘  └───────────┬─────────────┘  │
│         └─────────────────┴─────────────────────┘                │
│                           │                                      │
│                    LiveChannelService                            │
│                           │                                      │
│              ┌────────────┴────────────┐                        │
│              │    ChannelEventSink     │                        │
│              │    (WebSocket events)   │                        │
│              └─────────────────────────┘                        │
└─────────────────────────────────────────────────────────────────┘
```

### 2.2 Crate 结构

新增 `crates/feishu/` 目录：

```
feishu/
├── Cargo.toml
└── src/
    ├── lib.rs           # 导出 FeishuPlugin
    ├── plugin.rs        # ChannelPlugin trait 实现
    ├── config.rs        # FeishuAccountConfig 配置结构
    ├── handler.rs       # 事件处理器（消息接收）
    ├── outbound.rs      # ChannelOutbound trait 实现（消息发送）
    ├── state.rs         # 账号状态管理
    ├── error.rs         # 错误类型定义
    └── access.rs        # 群组策略/权限控制
```

---

## 3. 详细设计

### 3.1 配置结构

```rust
// config.rs
pub struct FeishuAccountConfig {
    pub app_id: String,
    pub app_secret: Secret<String>,
    pub connection_mode: ConnectionMode,     // 仅 WebSocket
    pub group_policy: GroupPolicy,           // allowlist/open/closed
    pub group_allowlist: Vec<String>,        // 群组 ID 列表
    pub otp_cooldown_secs: u64,
    pub encrypt_key: Option<Secret<String>>, // 可选：消息加密
}

pub enum ConnectionMode {
    Websocket,  // 默认且唯一支持的模式
}

pub enum GroupPolicy {
    Allowlist,  // 只在指定群组响应
    Open,       // 在所有群组响应
    Closed,     // 仅私聊
}
```

### 3.2 ChannelType 扩展

```rust
// crates/channels/src/plugin.rs
pub enum ChannelType {
    Telegram,
    Whatsapp,
    MsTeams,
    Discord,
    Feishu,  // NEW
}
```

### 3.3 ChannelsConfig 扩展

```rust
// crates/config/src/schema.rs
pub struct ChannelsConfig {
    pub offered: Vec<String>,        // 添加 "feishu"
    pub telegram: HashMap<String, serde_json::Value>,
    pub whatsapp: HashMap<String, serde_json::Value>,
    pub msteams: HashMap<String, serde_json::Value>,
    pub discord: HashMap<String, serde_json::Value>,
    pub feishu: HashMap<String, serde_json::Value>,  // NEW
}
```

---

## 4. 数据流

### 4.1 消息接收（Inbound）

```
飞书服务器
    │
    ▼ WebSocket
open-lark SDK
    │
    ▼ EventDispatcher
FeishuHandler::handle_message()
    │
    ├── 1. 消息去重（基于 message_id）
    ├── 2. 解析上下文（发送者、群聊/私聊）
    ├── 3. 权限校验（群组策略、allowlist）
    ├── 4. 提取附件（图片、文件）
    └── 5. ChannelEventSink::dispatch_to_chat()
                │
                ▼
          Chat Session
```

### 4.2 消息发送（Outbound）

```
Chat Session
    │
    ▼
ChannelOutbound::send_text()
    │
    ▼
FeishuOutbound::send_text()
    │
    ▼ HTTP API
飞书 Open API
    │
    ▼
用户/群组
```

---

## 5. 功能特性

### 5.1 已实现（MVP）

| 特性 | 状态 | 说明 |
|------|------|------|
| WebSocket 长连接 | ✅ | 使用 open-lark |
| 事件分发 | ✅ | 基于 open-lark EventDispatcher |
| 消息接收 | ✅ | 文本、图片、文件 |
| 消息发送 | ✅ | 文本、图片 |
| 群组策略 | ✅ | allowlist/open/closed |
| 权限控制 | ✅ | OTP 验证、allowlist |
| 消息去重 | ✅ | 基于 message_id |

### 5.2 后续迭代

| 特性 | 优先级 | 说明 |
|------|--------|------|
| 流式响应 | P1 | 使用飞书交互式卡片 |
| @提及转发 | P2 | 路由给其他 Agent |
| 消息加密 | P2 | 支持 encrypt_key |
| 富文本卡片 | P2 | 使用飞书消息卡片 |

---

## 6. 错误处理

| 错误类型 | 处理方式 |
|---------|---------|
| WebSocket 断开 | 自动重连（open-lark 内置） |
| 认证失败 | 禁用账号，记录错误日志 |
| API 限流 | 指数退避重试 |
| 群组权限不足 | 发送提示消息给用户 |
| 消息解析失败 | 记录错误，发送友好提示 |

---

## 7. 测试策略

### 7.1 单元测试

- Config 解析（序列化/反序列化）
- 群组策略逻辑
- 消息格式化

### 7.2 集成测试

- 使用飞书沙箱环境测试 API 调用
- WebSocket 连接/断开/重连

### 7.3 模拟测试

- Mock open-lark 客户端
- 测试事件处理流程
- 测试错误恢复逻辑

---

## 8. 依赖

```toml
[dependencies]
open-lark = "0.3"  # 飞书 API SDK
tokio = { workspace = true }
serde = { workspace = true }
secrecy = { workspace = true }
tracing = { workspace = true }
moltis-channels = { workspace = true }
moltis-common = { workspace = true }
```

---

## 9. 配置示例

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

---

## 10. 风险评估

| 风险 | 可能性 | 影响 | 缓解措施 |
|------|--------|------|---------|
| open-lark 不稳定 | 低 | 中 | 评估 crate 质量，准备备用方案 |
| 飞书 API 变更 | 中 | 低 | 关注 API 变更日志，及时更新 |
| WebSocket 连接不稳定 | 中 | 低 | open-lark 内置重连机制 |

---

## 11. 决策记录

| 决策 | 选项 | 选择 | 理由 |
|------|------|------|------|
| 连接模式 | WebSocket/Webhook/双模式 | WebSocket | 与现有架构一致，open-lark 已支持 |
| 群组策略 | allowlist/open/closed | 三种都支持 | 企业场景需要 |
| 流式响应 | 是/否 | 否（MVP） | 后续迭代添加 |
| @转发 | 是/否 | 否 | 简化 MVP 实现 |
| 实现方案 | open-lark/原生 | open-lark | 减少重复开发，更快交付 |

---

**批准人**: sunxianbing
**日期**: 2026-03-03
