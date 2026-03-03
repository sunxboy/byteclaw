//! Feishu channel error types.

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

    /// Webhook validation error.
    #[error("webhook validation error: {0}")]
    Webhook(String),
}
