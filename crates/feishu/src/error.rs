//! Feishu channel error types.

use moltis_channels::Error as ChannelError;
use thiserror::Error;

/// Errors that can occur in the Feishu channel.
#[derive(Debug, Error)]
pub enum FeishuError {
    /// Configuration error.
    #[error("configuration error: {0}")]
    Config(String),

    /// API request error.
    #[error("API error: {0}")]
    Api(String),

    /// WebSocket connection error.
    #[error("WebSocket error: {0}")]
    WebSocket(String),

    /// Authentication failed.
    #[error("authentication failed: {0}")]
    Auth(String),
}

impl From<FeishuError> for ChannelError {
    fn from(err: FeishuError) -> Self {
        match err {
            FeishuError::Config(msg) => ChannelError::invalid_input(msg),
            FeishuError::Api(msg) => ChannelError::external("Feishu API", std::io::Error::other(msg)),
            FeishuError::WebSocket(msg) => ChannelError::unavailable(msg),
            FeishuError::Auth(msg) => ChannelError::unavailable(format!("auth failed: {msg}")),
        }
    }
}
