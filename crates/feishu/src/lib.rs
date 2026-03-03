//! Feishu (Lark) channel plugin.

pub mod config;
pub mod error;
pub mod handler;
pub mod outbound;
pub mod plugin;
pub mod state;
pub mod access;

pub use plugin::FeishuPlugin;
