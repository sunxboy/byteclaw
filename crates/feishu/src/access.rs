//! Feishu access control and permissions.

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
