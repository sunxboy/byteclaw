<div align="center">

# Byteclaw — Moltis Fork with Feishu Support

A Rust-native AI assistant with **Feishu (Lark)** channel support.

[![Build](https://github.com/sunxboy/byteclaw/actions/workflows/build.yml/badge.svg)](https://github.com/sunxboy/byteclaw/actions/workflows/build.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.91%2B-orange.svg)](https://www.rust-lang.org)

[Installation](#installation) • [Features](#features) • [Feishu Setup](#feishu-setup) • [Building](#building)

</div>

---

**Byteclaw** is a fork of [Moltis](https://github.com/moltis-org/moltis) with added support for **Feishu (Lark)** enterprise messaging platform.

## What's Different from Moltis?

- **Feishu Channel**: Full support for Feishu (Lark) messaging platform
  - Bot message handling
  - Encrypted token verification
  - Group and private chat support
  - Account linking with OTP flow

## Installation

### Download Pre-built Binaries

Download from [GitHub Releases](https://github.com/sunxboy/byteclaw/releases):

```bash
# Linux x86_64
curl -L https://github.com/sunxboy/byteclaw/releases/latest/download/moltis-linux-x86_64.tar.gz | tar xz

# Linux ARM64 (Raspberry Pi, etc.)
curl -L https://github.com/sunxboy/byteclaw/releases/latest/download/moltis-linux-arm64.tar.gz | tar xz

# macOS (Intel)
curl -L https://github.com/sunxboy/byteclaw/releases/latest/download/moltis-macos-x86_64.tar.gz | tar xz

# macOS (Apple Silicon)
curl -L https://github.com/sunxboy/byteclaw/releases/latest/download/moltis-macos-arm64.tar.gz | tar xz
```

### Build from Source

```bash
git clone https://github.com/sunxboy/byteclaw
cd byteclaw
cargo build --release
```

## Features

All Moltis features plus:

- **Feishu Bot Integration**
  - Receive and respond to messages in Feishu
  - Support for group chats and private conversations
  - Event verification with encrypted tokens

- **Multi-Platform**
  - x86_64 and ARM64 support
  - Linux and macOS binaries
  - Optimized for Raspberry Pi 4 (Cortex-A72)

## Feishu Setup

1. Create a Feishu bot in your Feishu admin console
2. Get your App ID and App Secret
3. Configure in `~/.moltis/moltis.toml`:

```toml
[channel.feishu]
enabled = true
app_id = "cli_xxxxx"
app_secret = "xxxxx"
encrypt_key = "xxxxx"  # Optional: for event encryption
verification_token = "xxxxx"  # Optional: for URL verification
```

4. Set up webhook URL pointing to your Byteclaw instance

## Building

### Cross-compile for Raspberry Pi 4 (ARM64)

On macOS:
```bash
# Install cross-compiler
brew install aarch64-elf-gcc

# Build
cargo build --release --target aarch64-unknown-linux-gnu
```

On Linux:
```bash
sudo apt-get install gcc-aarch64-linux-gnu
cargo build --release --target aarch64-unknown-linux-gnu
```

### Build All Targets

```bash
# Linux x86_64
cargo build --release --target x86_64-unknown-linux-gnu

# Linux ARM64
cargo build --release --target aarch64-unknown-linux-gnu

# macOS Intel
cargo build --release --target x86_64-apple-darwin

# macOS Apple Silicon
cargo build --release --target aarch64-apple-darwin
```

## Original Moltis Features

- Secure by design — keys never leave your machine
- Sandboxed execution (Docker + Apple Container)
- Voice I/O with 15+ providers
- MCP servers support
- Memory/RAG with SQLite + vector search
- Telegram, Discord, WhatsApp, Teams channels
- Browser automation
- Scheduling and cron jobs
- Password + Passkey authentication

See [Moltis documentation](https://docs.moltis.org) for full feature details.

## License

MIT License — see [LICENSE](LICENSE) file.

---

*This is a community fork. For the original project, visit [moltis-org/moltis](https://github.com/moltis-org/moltis).*
