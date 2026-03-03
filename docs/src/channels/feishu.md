# Feishu Channel

Moltis 支持通过飞书（Lark）机器人与你的 Agent 进行交互。

## 配置

在 `moltis.toml` 中添加飞书账号配置：

```toml
[channels]
offered = ["telegram", "discord", "feishu"]

[channels.feishu.my-bot]
app_id = "cli_xxxxxxxxxx"
app_secret = "xxxxxxxxxx"
connection_mode = "websocket"
group_policy = "allowlist"
group_allowlist = ["oc_xxxxxxxx"]
otp_cooldown_secs = 3600
```

## 配置选项

| 选项 | 类型 | 必填 | 默认值 | 描述 |
|------|------|------|--------|------|
| `app_id` | string | 是 | - | 飞书应用 ID |
| `app_secret` | string | 是 | - | 飞书应用密钥 |
| `connection_mode` | string | 否 | `websocket` | 连接模式，目前仅支持 WebSocket |
| `group_policy` | string | 否 | `closed` | 群组策略: `allowlist`, `open`, `closed` |
| `group_allowlist` | array | 否 | `[]` | 允许访问的群组 ID 列表 |
| `otp_cooldown_secs` | number | 否 | `3600` | OTP 验证冷却时间（秒） |

## 群组策略

- **`closed`** (默认): 仅在私聊中响应，不响应任何群组消息
- **`allowlist`**: 仅响应白名单中的群组和私聊
- **`open`**: 响应所有群组和私聊

## 获取凭证

1. 访问 [飞书开放平台](https://open.feishu.cn/)
2. 创建企业自建应用
3. 在「凭证与基础信息」中获取 **App ID** 和 **App Secret**
4. 在「事件订阅」中订阅以下事件：
   - `im.message.receive_v1` - 接收消息
5. 在「权限管理」中申请以下权限：
   - `im:chat:readonly` - 读取群组信息
   - `im:message:send` - 发送消息
   - `im:message.group_msg` - 接收群消息
   - `im:message.p2p_msg` - 接收单聊消息
6. 发布应用（需要管理员审批）

## WebSocket 模式

飞书 channel 使用 WebSocket 长连接接收消息，无需配置回调 URL。启动后会自动建立连接并保持在线状态。

## 用户验证

对于不在白名单中的私聊用户，飞书 channel 支持 OTP 验证流程：

1. 用户首次发送消息时会收到验证码提示
2. 用户在 Web UI 的「Channel Access」页面输入验证码
3. 验证通过后，用户可以正常使用

## 消息类型

### 支持接收
- 文本消息
- 图片消息
- 文件消息

### 支持发送
- 文本消息
- Markdown 消息

## 故障排查

### 无法接收消息
- 确认应用已发布并通过审批
- 检查事件订阅是否正确配置了 `im.message.receive_v1`
- 查看日志确认 WebSocket 连接状态

### 无法发送消息
- 确认已申请 `im:message:send` 权限
- 检查 App ID 和 App Secret 是否正确

### 群组无响应
- 检查群组策略配置
- 确认机器人已被添加到群组
- 如果是 allowlist 策略，检查群组 ID 是否在白名单中
