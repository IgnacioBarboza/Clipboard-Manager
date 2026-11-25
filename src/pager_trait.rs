/// Sends desktop notifications to the user.
pub trait Pager {
    fn notify_user(title: &str, body: &str) -> Result<(), String>;
}
