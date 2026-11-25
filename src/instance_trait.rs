/// Access to the system clipboard.
pub trait ClipboardInstance {
    /// Creates a handle to the clipboard.
    fn new() -> Result<Self, String>
    where
        Self: Sized;

    /// Places `content` on the clipboard as plain text.
    fn write_text(&mut self, content: String) -> Result<(), String>;

    /// Places the image stored at `file_path` on the clipboard.
    fn write_image(&mut self, file_path: &str) -> Result<(), String>;

    /// Reads the current clipboard content as raw bytes of the given image MIME type.
    fn read_image(&mut self, mime_type: &str) -> Result<Vec<u8>, String>;
}
