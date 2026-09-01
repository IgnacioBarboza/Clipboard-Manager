use std::process::Command;
use crate pager_trait::Pager;

pub struct SystemPager;

impl Pager for SystemPager{
    fn notify_user(title: &str, body: &str) -> Result<(),String>{
        let status = Command::new("notify-send")
            .arg(title)
            .arg(body)
            .status()
            .expect("Error at trying to send a notification");

        if status.success() {
            Ok(())
        } else {
            Err(status.to_string())
        }
    }
}


