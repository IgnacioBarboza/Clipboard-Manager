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

// Constant path for our temporal state file (stored in RAM on Arch Linux)
const STATE_FILE: &str = "/tmp/clipcrab_state.json";

fn save_state<T: CircularLog>(log: &T) {
    let items = log.get_items();
    // Serialize the Vec<String> to JSON
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
    if let Ok(mut file) = File::open(STATE_FILE) {
        let mut json_str = String::new();
        if file.read_to_string(&mut json_str).is_ok() {
            // Deserialize the JSON back into a Vec<String>
            if let Ok(items) = serde_json::from_str::<Vec<String>>(&json_str) {
                // Re-populate the buffer
                for item in items {
                    log.push(item);
                }
            }
        }
    }
}

fn show_rofi_menu<T: CircularLog>(log: &T) -> Option<String> {
    let items = log.get_items();
    
    if items.is_empty() {
        println!("The Clipboard is empty.");
        return None;
    }

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
        
        // If the user selected something, send it to the clipboard
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
    // Instance parallel clipboard to set and get content

    // Attempt to load existing state if the daemon restarts
    load_state(&mut clipboard_log);

    for event in stream.paste_stream().flatten() {
        // Iterate through each successful event
        
        let available_mimes = event.mime_types.clone();

        let actual_event = event.context;
        // Capture the event context

        let type_actual_event = actual_event.mime_type.clone();
        // Capture the MIME type

        let content_actual_event = actual_event.context;

        // This prevents the browser's default HTML format from overriding the image
        if let Some(image_mime) = available_mimes.iter().find(|m| m.starts_with("image/")) {
            use wl_clipboard_rs::paste::{get_contents, ClipboardType, Seat, MimeType as PasteMimeType};
            
            let result = get_contents(
                ClipboardType::Regular,
                Seat::Unspecified,
                PasteMimeType::Specific(&image_mime.clone()),
            );

            if let Ok((mut reader, _)) = result {
                let mut image_bytes = Vec::new();
                if reader.read_to_end(&mut image_bytes).is_ok() {
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
                }
            }
            
            // Image is saved, skip the rest of the loop for this event
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

            // 2. HTML (IGNORED)
            "text/html" => {
                // Ignore HTML to prevent duplicate entries when browsers copy images/text
                println!("Ignoring HTML input from browser.");
            },

            // 3. Not Supported Format
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