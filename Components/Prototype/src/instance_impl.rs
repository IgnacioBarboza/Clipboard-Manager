use arboard::Clipboard;
use crate::instance_trait::ClipboardInstance;

pub type Instance = Clipboard;

impl ClipboardInstance for Instance{
    
    // Initialize instance
    fn new() -> Result<Self, String>{
        Clipboard::new()
            .map_err(|e| e.to_string())
    }

    // Write text format and return result
    fn write_text(&mut self, content: String) -> Result<(), String>{
        self.set_text(content).map_err(|e| e.to_string())
    }

    // Read text from clipboard and return result
    fn read_text(&mut self) -> Result<String, String>{
        self.get_text().map_err(|e| e.to_string())
    }
}