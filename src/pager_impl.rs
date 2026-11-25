use std::process::Command;

use crate::pager_trait::Pager;

/// Notification backend built on `notify-send` (libnotify).
pub struct SystemPager;

impl Pager for SystemPager {
    fn notify_user(title: &str, body: &str) -> Result<(), String> {
        let status = Command::new("notify-send")
            .arg("--app-name=ClipCrab")
            .arg("--")
            .arg(title)
            .arg(body)
            .status()
            .map_err(|e| format!("Failed to execute notify-send (is libnotify installed?): {e}"))?;

        if status.success() {
            Ok(())
        } else {
            Err(format!("notify-send exited with {status}"))
        }
    }
}
