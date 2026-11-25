pub trait Pager{
    fn notify_user(title:&str, body: &str)-> Result<(),String>;
}