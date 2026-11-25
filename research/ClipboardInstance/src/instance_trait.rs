pub trait ClipboardInstance {
    // Initialize instance
    fn new() -> Result<(Self), String>;

    // Set text format and return result
    fn set_text(&mut self, content: String) -> Result<(), String>;

    // Get text from clipboard and return result
    fn get_text(&self) -> Result<String, String>;

}