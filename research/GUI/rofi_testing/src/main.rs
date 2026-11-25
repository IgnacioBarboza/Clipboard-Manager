use circular_buffer::FixedCircularBuffer;
use std::io::Write;
use std::process::{Command, Stdio};

fn main() {
    //We define the CircularBuffer
    let mut buffer = FixedCircularBuffer::<String, 5>::new();
    buffer.push_back("hhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhh".to_string());
    buffer.push_back("testing 1".to_string());
    buffer.push_back("testing 2".to_string());
    buffer.push_back("testing 3".to_string());
    buffer.push_back("testing 4".to_string());

    //We add the diferent values to rofi input
    let mut rofi_input = String::new();
    for (i, item) in buffer.iter().enumerate() {
        rofi_input.push_str(&format!("{}: {}\n", i, item));
    }

    // Execute rofi as dmenu
    let mut child = Command::new("rofi")
        .arg("-dmenu")        
        .arg("-i")            // Low and high CAPS
        .arg("-p")            // Define title
        .arg("ClipCrab")
        .stdin(Stdio::piped())  // Open input pipe
        .stdout(Stdio::piped()) // Open output pipe
        .spawn()
        .expect("Error when executing rofi, is rofi donwloaded?");

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(rofi_input.as_bytes()).expect("Error when writing on input pipeline");
    }

    let output = child.wait_with_output().expect("Error when reading stdout");

    if output.status.success() {
        let selected = String::from_utf8_lossy(&output.stdout);
        let selected = selected.trim(); 
        
        println!("The id/text '{}' was selected.", selected);
        
    } else {
        println!("No element was selected.");
    }
}