use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};

use wl_clipboard_rs::paste::{
    ClipboardType, Error as PasteError, MimeType as PasteMimeType, Seat, get_contents,
};

use crate::instance_trait::ClipboardInstance;

/// Wayland clipboard backend.
///
/// Reading goes through `wl-clipboard-rs`; writing is delegated to `wl-copy`,
/// which keeps serving the selection after this process exits.
pub struct WaylandClipboard;

/// Maps an image file extension to its MIME type (`jpg` -> `image/jpeg`, `png` -> `image/png`).
fn image_mime_from_path(file_path: &str) -> Option<String> {
    let extension = Path::new(file_path)
        .extension()?
        .to_str()?
        .to_ascii_lowercase();

    match extension.as_str() {
        "jpg" | "jpeg" => Some("image/jpeg".to_string()),
        "svg" => Some("image/svg+xml".to_string()),
        "" => None,
        other => Some(format!("image/{other}")),
    }
}

impl ClipboardInstance for WaylandClipboard {
    fn new() -> Result<Self, String> {
        Ok(WaylandClipboard)
    }

    fn write_text(&mut self, content: String) -> Result<(), String> {
        let mut child = Command::new("wl-copy")
            .stdin(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to execute wl-copy (is wl-clipboard installed?): {e}"))?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(content.as_bytes())
                .map_err(|e| format!("Failed to write text to wl-copy: {e}"))?;
        }

        child
            .wait()
            .map_err(|e| format!("wl-copy process failed: {e}"))?;

        Ok(())
    }

    fn write_image(&mut self, file_path: &str) -> Result<(), String> {
        let mime_type = image_mime_from_path(file_path)
            .ok_or_else(|| format!("Cannot infer an image type for {file_path}"))?;

        let file = File::open(file_path)
            .map_err(|e| format!("Could not open image file {file_path}: {e}"))?;

        let mut child = Command::new("wl-copy")
            .arg("--type")
            .arg(&mime_type)
            .stdin(Stdio::from(file))
            .spawn()
            .map_err(|e| format!("Failed to execute wl-copy (is wl-clipboard installed?): {e}"))?;

        child
            .wait()
            .map_err(|e| format!("wl-copy process failed: {e}"))?;

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
                reader
                    .read_to_end(&mut image_bytes)
                    .map_err(|e| format!("Failed to read image stream: {e}"))?;
                Ok(image_bytes)
            }
            Err(PasteError::NoSeats) | Err(PasteError::ClipboardEmpty) => {
                Err("Clipboard is empty".to_string())
            }
            Err(e) => Err(format!("Wayland read_image error: {e}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::image_mime_from_path;

    #[test]
    fn maps_common_extensions() {
        assert_eq!(
            image_mime_from_path("/a/b.png").as_deref(),
            Some("image/png")
        );
        assert_eq!(
            image_mime_from_path("/a/b.JPG").as_deref(),
            Some("image/jpeg")
        );
        assert_eq!(
            image_mime_from_path("/a/b.jpeg").as_deref(),
            Some("image/jpeg")
        );
        assert_eq!(
            image_mime_from_path("/a/b.webp").as_deref(),
            Some("image/webp")
        );
        assert_eq!(
            image_mime_from_path("/a/b.gif").as_deref(),
            Some("image/gif")
        );
        assert_eq!(
            image_mime_from_path("/a/b.svg").as_deref(),
            Some("image/svg+xml")
        );
    }

    #[test]
    fn rejects_paths_without_extension() {
        assert_eq!(image_mime_from_path("/a/b"), None);
    }
}
