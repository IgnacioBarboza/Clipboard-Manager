use wayland_clipboard_listener::WlClipboardPasteStream;
use std::process::Command;
use wayland_clipboard_listener::WlListenType;

fn notify(title: String, body: String) {
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

fn main() {
    // RUN cargo add wayland_clipboard_listener
    
    let mut stream = WlClipboardPasteStream::init(WlListenType::ListenOnCopy).unwrap(); 
    // Open the clipboard listener to passively listen on copy events.
    
    for event in stream.paste_stream().flatten() {
        // Iterate through each successful event

        let actual_event = event.context;
        // Capture the event context

        let type_actual_event = actual_event.mime_type;
        // Capture the MIME type

        let content_actual_event = actual_event.context;

        if type_actual_event == "text/plain;charset=utf-8" || type_actual_event == "text/plain" {
            // If the event format is text or plain text
            match String::from_utf8(content_actual_event) {
                // Decode the content to UTF-8
                Ok(text) => {
                    notify("New Input in the Clipboard".to_string(), text);
                    // Notify the user
                },
                Err(error) => {
                    eprintln!("There's been an error during decoding: {}", error);
                }
            }
        } else {
            notify("New Input in the Clipboard".to_string(), "Not Supported for ".to_owned() + &type_actual_event + " type");
        }
    }
}