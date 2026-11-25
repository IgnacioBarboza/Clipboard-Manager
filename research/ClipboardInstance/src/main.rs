use arboard::Clipboard;
use std::thread::sleep;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    // Initilize clipboard
    let mut clipboard = Clipboard::new()?;

    // Set initial value
    let original_text = "INITIAL";
    clipboard.set_text(original_text)?;

    // Read whats inside de clipboard
    match clipboard.get_text() {
        Ok(text) => println!("Succesful initial writing: {}", text),
        Err(e) => println!("Theres been an error: {}", e),
    }

    // Write a new text
    let new_text = "Modified";
    clipboard.set_text(new_text)?;


    match clipboard.get_text() {
        Ok(text) => println!("Succesful modified writing: {}", text),
        Err(e) => println!("Theres been an error: {}", e),
    }
    sleep(Duration::from_secs(2));

    Ok(())
}