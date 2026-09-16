//! macOS iMessage Bridge Connector.
//!
//! Provides local AppleScript commands and SQLite integration for
//! reading and sending iMessages natively on macOS.

/// macOS iMessage Bridge Client.
#[derive(Debug, Default, Clone)]
pub struct IMessageBridge;

impl IMessageBridge {
    /// Creates a new iMessage bridge client.
    pub fn new() -> Self {
        Self
    }

    /// Builds AppleScript command string to send a plain text message to a phone or email.
    pub fn build_send_text_script(&self, recipient: &str, body: &str) -> String {
        let escaped_body = body.replace('\\', "\\\\").replace('"', "\\\"");
        let escaped_recipient = recipient.replace('\\', "\\\\").replace('"', "\\\"");
        format!(
            "tell application \"Messages\"\n  set targetService to 1st service whose service type = iMessage\n  set targetBuddy to buddy \"{}\" of targetService\n  send \"{}\" to targetBuddy\nend tell",
            escaped_recipient, escaped_body
        )
    }

    /// Builds AppleScript command string to send an image or file attachment.
    pub fn build_send_file_script(&self, recipient: &str, posix_file_path: &str) -> String {
        let escaped_path = posix_file_path.replace('\\', "\\\\").replace('"', "\\\"");
        let escaped_recipient = recipient.replace('\\', "\\\\").replace('"', "\\\"");
        format!(
            "tell application \"Messages\"\n  set targetService to 1st service whose service type = iMessage\n  set targetBuddy to buddy \"{}\" of targetService\n  send POSIX file \"{}\" to targetBuddy\nend tell",
            escaped_recipient, escaped_path
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_imessage_applescript_builder() {
        let bridge = IMessageBridge::new();
        let script = bridge.build_send_text_script("+15551234567", "Hello from Kineti iMessage!");

        assert!(script.contains("tell application \"Messages\""));
        assert!(script.contains("buddy \"+15551234567\""));
        assert!(script.contains("send \"Hello from Kineti iMessage!\""));
    }

    #[test]
    fn test_imessage_send_file_script() {
        let bridge = IMessageBridge::new();
        let script = bridge.build_send_file_script("+15551234567", "/tmp/generated_workspace.png");

        assert!(script.contains("send POSIX file \"/tmp/generated_workspace.png\""));
    }
}
