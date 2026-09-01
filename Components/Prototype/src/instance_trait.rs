pub trait ClipboardInstance {
    // Initialize instance
    fn new() -> Result<Self, String> where Self: Sized;

    // Write text format and return result
    fn write_text(&mut self, content: String) -> Result<(), String>;

    // Read text from clipboard and return result
    fn read_text(&mut self) -> Result<String, String>;

}