use std::fs;
use std::io::Read;
use crate::instance_trait::ClipboardInstance;
use wl_clipboard_rs::copy::{MimeType as CopyMimeType, Options, Source};
use wl_clipboard_rs::paste::{get_contents, ClipboardType, Error as PasteError, MimeType as PasteMimeType, Seat};


pub struct WaylandClipboard;

impl ClipboardInstance for WaylandClipboard {
    fn new() -> Result<Self, String> {
        // Initialization succeeds immediately
        Ok(WaylandClipboard)
    }

    fn write_text(&mut self, content: String) -> Result<(), String> {
        let opts = Options::new();
        opts.copy(
            Source::Bytes(content.into_bytes().into()),
            CopyMimeType::Text,
        ).map_err(|e| format!("Wayland write_text error: {}", e))
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
        // 1. Read the raw bytes from the file
        let image_data = fs::read(file_path)
            .map_err(|e| format!("Failed to read image file {}: {}", file_path, e))?;

        // 2. Determine the MIME type based on the file extension
        let mime_type = if file_path.to_lowercase().ends_with(".png") {
            "image/png"
        } else if file_path.to_lowercase().ends_with(".jpg") || file_path.to_lowercase().ends_with(".jpeg") {
            "image/jpeg"
        } else if file_path.to_lowercase().ends_with(".webp") {
            "image/webp"
        } else {
            return Err("Unsupported format. Please use .png, .jpg, or .webp".to_string());
        };

        // 3. Send the raw bytes to Wayland with the exact MIME type
        let opts = Options::new();
        opts.copy(
            Source::Bytes(image_data.into()),
            CopyMimeType::Specific(mime_type.to_string()),
        ).map_err(|e| format!("Wayland write_image error: {}", e))
    }
}