use std::fs::File; // Added for write_image
use std::io::Read;
use std::process::{Command, Stdio};
use std::io::Write;
use crate::instance_trait::ClipboardInstance;
use wl_clipboard_rs::paste::{get_contents, ClipboardType, Error as PasteError, MimeType as PasteMimeType, Seat};

pub struct WaylandClipboard;

impl ClipboardInstance for WaylandClipboard {
    fn new() -> Result<Self, String> {
        // Initialization succeeds immediately
        Ok(WaylandClipboard)
    }

    fn write_text(&mut self, content: String) -> Result<(), String> {
        let mut child = Command::new("wl-copy")
            .stdin(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to execute wl-copy: {}", e))?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(content.as_bytes())
                .map_err(|e| format!("Failed to write text to wl-copy: {}", e))?;
        }

        child.wait().map_err(|e| format!("wl-copy process failed: {}", e))?;
        
        Ok(())
    }

    fn read_text(&mut self) -> Result<String, String> {
        // Request standard text format from the regular clipboard
        let result = get_contents(
            ClipboardType::Regular,
            Seat::Unspecified,
            PasteMimeType::Text,
        );

        match result {
            Ok((mut reader, _mime_type)) => {
                let mut content = String::new();
                reader.read_to_string(&mut content)
                    .map_err(|e| format!("Failed to read stream to string: {}", e))?;
                Ok(content)
            }
            Err(PasteError::NoSeats) | Err(PasteError::ClipboardEmpty) => {
                Ok(String::new()) // Return empty string if clipboard is empty
            }
            Err(e) => Err(format!("Wayland read_text error: {}", e)),
        }
    }

    fn write_image(&mut self, file_path: &str) -> Result<(), String> {
        // 1. Determine the MIME type based on the file extension
        let mime_type = if file_path.to_lowercase().ends_with(".png") {
            "image/png"
        } else if file_path.to_lowercase().ends_with(".jpg") || file_path.to_lowercase().ends_with(".jpeg") {
            "image/jpeg"
        } else if file_path.to_lowercase().ends_with(".webp") {
            "image/webp"
        } else {
            return Err("Unsupported format. Please use .png, .jpg, or .webp".to_string());
        };

        // 2. Open the physical temporary file saved by the daemon
        let file = File::open(file_path)
            .map_err(|e| format!("Could not open image file {}: {}", file_path, e))?;

        // 3. Delegate to wl-copy, injecting the file directly into Stdio
        let mut child = Command::new("wl-copy")
            .arg("-t")          // Specify MIME type
            .arg(mime_type)
            .stdin(Stdio::from(file)) // Pass the file directly
            .spawn()
            .map_err(|e| format!("Failed to execute wl-copy for image: {}", e))?;

        child.wait().map_err(|e| format!("wl-copy process failed: {}", e))?;

        Ok(())
    }
    fn read_image(&mut self, mime_type: &str) -> Result<Vec<u8>, String> {
        let result = get_contents(
            ClipboardType::Regular,
            Seat::Unspecified,
            PasteMimeType::Specific(mime_type),
        );

        match result {
            Ok((mut reader, _)) => {
                let mut image_bytes = Vec::new();
                reader.read_to_end(&mut image_bytes)
                    .map_err(|e| format!("Failed to read image stream: {}", e))?;
                Ok(image_bytes)
            }
            Err(PasteError::NoSeats) | Err(PasteError::ClipboardEmpty) => {
                Err("Clipboard is empty".to_string())
            }
            Err(e) => Err(format!("Wayland read_image error: {}", e)),
        }
    }
}