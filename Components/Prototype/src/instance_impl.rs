use arboard::Clipboard;
use std::thread::sleep;
use std::time::Duration;
use crate::instance_trait::ClipboardInstance;

pub type Instance = Clipboard;

impl ClipboardInstance for Instance{
    
    // Initialize instance
    fn new() -> Result<(Self), String>{
        Clipboard::new()
            .map_err(|e| e.to_string())
    }

    // Set text format and return result
    fn set_text(&mut self, content: String) -> Result<(), String>{
        self.set_text(content)?
    }

    // Get text from clipboard and return result
    fn get_text(&self) -> Result<String, String>{
        self.get_text()
    }
}