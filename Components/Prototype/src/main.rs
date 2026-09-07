use std::env;
use std::fs; 
use std::fs::File;
use std::io::{Write, Read};
use std::process::{Command, Stdio};
use serde_json;
use std::time::{SystemTime, UNIX_EPOCH};


use wayland_clipboard_listener::WlClipboardPasteStream;
use wayland_clipboard_listener::WlListenType;

mod circular_trait;
mod circular_impl;

use circular_trait::CircularLog;
use circular_impl::Buffer;
use crate::instance_trait::ClipboardInstance;
use crate::instance_impl::WaylandClipboard;
mod instance_trait;
mod instance_impl;

mod pager_trait;
mod pager_impl;

use pager_trait::Pager;
use pager_impl::SystemPager;

// Constant path for our temporal state file (stored in RAM on Arch)
const STATE_FILE: &str = "/tmp/clipcrab_state.json";

fn save_state<T: CircularLog>(log: &T) {
    // This function saves the current ClipboardLog to the PATH set in the variable "STATE_FILE" in a json format

    let items = log.get_items();
    match serde_json::to_string(&items) {
        Ok(json_str) => {
            if let Err(e) = fs::write(STATE_FILE, json_str) {
                eprintln!("Failed to write state file: {}", e);
            }
        },
        Err(e) => eprintln!("Failed to serialize state: {}", e),
    }
}

fn load_state(log: &mut Buffer) {
    // Loads the Buffer with the content of the current clipcrab_state.json

    if let Ok(mut file) = File::open(STATE_FILE) {
        let mut json_str = String::new();
        if file.read_to_string(&mut json_str).is_ok() {
            if let Ok(items) = serde_json::from_str::<Vec<String>>(&json_str) {
                for item in items {
                    log.push(item);
                }
            }
        }
    }
}

fn show_rofi_menu<T: CircularLog>(log: &T) -> Option<String> {
    // Uses rofi to show the current Clipboard log to the user. If the user chooses the options, the clipboard
    // will add it to the current choice.
    let items = log.get_items();
    
    if items.is_empty() {
        println!("The Clipboard is empty.");
        return None;
    }

    // Loads the rofi panel with the options.
    let mut rofi_input = String::new();
    for (i, item) in items.iter().rev().enumerate() {
        let sanitized_item = item.replace('\n', " ↵ ");
        rofi_input.push_str(&format!("{}: {}\n", i + 1, sanitized_item));
    }
    
    // Execute rofi as dmenu
    let mut child = Command::new("rofi")
        .arg("-dmenu")        
        .arg("-i")            
        .arg("-p")            
        .arg("ClipCrab")
        .stdin(Stdio::piped())  
        .stdout(Stdio::piped()) 
        .spawn()
        .expect("Error when executing rofi, is rofi downloaded?");

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(rofi_input.as_bytes()).expect("Error when writing on input pipeline");
    }

    let output = child.wait_with_output().expect("Error when reading stdout");

    // Condition if the user choosed a item.
    if output.status.success() {
        let selected = String::from_utf8_lossy(&output.stdout);
        let selected = selected.trim(); 
        
        if let Some(colon_pos) = selected.find(':') {
            if let Ok(display_index) = selected[..colon_pos].parse::<usize>() {
                let real_index = display_index.saturating_sub(1);
                if let Some(original_text) = items.iter().rev().nth(real_index) {
                    return Some(original_text.clone());
                }
            }
        }
    } else {
        println!("No element was selected or Rofi was closed.");
    }
    
    None
}

fn main() -> Result<(), Box<dyn std::error::Error>>{
    // RUN cargo add wayland_clipboard_listener
    // RUN cargo add circular_buffer
    // RUN cargo add serde --features derive 
    // RUN cargo add serde_json
    // RUN cargo add image
    // RUN cargo add wl_clipboard_rs

    let args: Vec<String> = env::args().collect();

    // Client Mode: `clipcrab --menu`
    if args.len() > 1 && args[1] == "--menu" {
        let mut mock_log = Buffer::new();
        // Load the state from the JSON file into the mock log
        load_state(&mut mock_log);
        
        // Fetches the content of the log in the choosed item by the user.
        if let Some(selected_text) = show_rofi_menu(&mock_log) {
            match WaylandClipboard::new() {
                Ok(mut clipboard) => {
                    // 1. IMAGE
                    if selected_text.starts_with("[Imagen] ") {
                        // Extract the file path by removing the prefix
                        let file_path = selected_text.trim_start_matches("[Imagen] ").trim();
                        
                        match clipboard.write_image(file_path) {
                            Ok(_) => println!("Successfully copied IMAGE!"),
                            Err(e) => eprintln!("Failed to write image: {}", e),
                        }
                    } 
                    // 2. HTML (Not Supported)
                    else if selected_text.starts_with("[HTML] ") {
                        println!("Format not supported.");
                        let _ = SystemPager::notify_user("ClipCrab", "HTML format not supported from menu.");
                    } 
                    // 3. TEXT PLAIN
                    else {
                        match clipboard.write_text(selected_text) {
                            Ok(_) => println!("Successfully copied TEXT!"),
                            Err(e) => eprintln!("Failed to write text: {}", e),
                        }
                    }
                },
                Err(e) => eprintln!("Failed to init clipboard: {}", e),
            }
        }
        return Ok(());
    }

    // Daemon Mode: Passively listen on copy events
    let mut stream = WlClipboardPasteStream::init(WlListenType::ListenOnCopy).unwrap(); 
    // Open the clipboard listener to passively listen on copy events.
    let mut clipboard_log = Buffer::new();

    // Attempt to load existing state if the daemon restarts
    load_state(&mut clipboard_log);

    for event in stream.paste_stream().flatten() {
        // Iterate through each successful event
        
        let available_mimes = event.mime_types.clone();

        let actual_event = event.context;
        // Capture the event context

        let type_actual_event = actual_event.mime_type.clone();
        // Capture the MIME type

        // Capture the content of the event.
        let content_actual_event = actual_event.context;

        // Checks whether any of the mimes types of the chosen file, is an image.
        if let Some(image_mime) = available_mimes.iter().find(|m| m.starts_with("image/")) {
            // Instantiating the clipboard safely using match, just like in client mode
            match WaylandClipboard::new() {
                Ok(mut clipboard) => {
                    // Delegate reading logic to the ClipboardInstance trait
                    if let Ok(image_bytes) = clipboard.read_image(image_mime) {
                        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
                        let ext = image_mime.split('/').last().unwrap_or("png");
                        let file_path = format!("/tmp/clipcrab_img_{}.{}", timestamp, ext);

                        if fs::write(&file_path, &image_bytes).is_ok() {
                            let _ = SystemPager::notify_user("New Image Copied", &format!("Saved to {}", file_path));
                            let formatted_image = format!("[Imagen] {}", file_path);
                            clipboard_log.push(formatted_image);
                            save_state(&clipboard_log);
                        } else {
                            eprintln!("Failed to save image to disk.");
                        }
                    } else {
                        eprintln!("Failed to read image from clipboard.");
                    }
                },
                Err(e) => eprintln!("Failed to init clipboard in daemon: {}", e),
            }
            
            // Image is saved (or failed safely), skip the rest of the loop for this event
            continue; 
        }

        match type_actual_event.as_str() {
            // 1. Plain text
            "text/plain;charset=utf-8" | "text/plain" => {
                if let Ok(text) = String::from_utf8(content_actual_event) {
                    let _ = SystemPager::notify_user("New Text Copied", &text);
                    clipboard_log.push(text);
                    save_state(&clipboard_log);
                }
            },

            // 2. Not Supported Format
            _ => {
                let _ = SystemPager::notify_user(
                    "New Input in the Clipboard", 
                    &format!("Not Supported for {} type", type_actual_event)
                );
            }
        }
    }
    Ok(())
}