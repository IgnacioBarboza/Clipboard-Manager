use wayland_clipboard_listener::WlClipboardPasteStream;
use wayland_clipboard_listener::WlListenType;

mod circular_trait;
mod circular_impl;

use circular_trait::CircularLog;
use circular_impl::Buffer;

mod instance_trait;
mod instance_impl;

use instance_trait::ClipboardInstance;
use instance_impl::Instance;

mod pager_trait;
mod pager_impl;

use pager_trait::Pager;
use pager_impl::SystemPager;

fn print_content<T: CircularLog>(log: &T) {
    println!("*----------*");
    println!("Content of clipboard: ");
    let items = log.get_items(); 
    
    for (i, item) in items.iter().enumerate() {
        println!("{}: {}", i, item);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>>{
    // RUN cargo add wayland_clipboard_listener
    // RUN cargo add circular_buffer
    
    let mut stream = WlClipboardPasteStream::init(WlListenType::ListenOnCopy).unwrap(); 
    // Open the clipboard listener to passively listen on copy events.
    let mut clipboard_log = Buffer::new();
    // Instance parallel clipboard to set and get content
    //let _clipboard_instance = Instance::new()?;

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
                    // Notify the user
                    let _ = SystemPager::notify_user("New Input in the Clipboard", &text);

                    // Push the content to the buffer
                    clipboard_log.push(text);
                    
                    // Print content of the whole clipboard
                    print_content(&clipboard_log);
                },
                Err(error) => {
                    eprintln!("There's been an error during decoding: {}", error);
                }
            }
        } else {
            let _ = SystemPager::notify_user("New Input in the Clipboard", &("Not Supported for ".to_owned() + &type_actual_event + " type"));
        }
    }
    Ok(())
}