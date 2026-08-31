use std::process::Command;

fn main() {

    let title = String::from("Title");
    let body = String::from("Body");
    // Investigate how -i icon works.
    let status = Command::new("notify-send")
        .arg(title)
        .arg(body)
        .status()
        .expect("Error at trying to send a notification");

    if status.success() {
        println!("The notification has been sent");
    } else {
        println!("There's been an error: {}", status);
    }
}
