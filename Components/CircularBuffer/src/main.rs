mod clipboard_trait;
mod circular_impl;

use clipboard_trait::ClipboardBuffer;
use circular_impl::MyBuffer;

fn print_content<T: ClipboardBuffer>(log: &T) {
    println!("*----------*");
    println!("Content of clipboard: ");
    let items = log.get_items(); 
    
    for (i, item) in items.iter().enumerate() {
        println!("{}: {}", i, item);
    }
    println!("*----------*");
}

fn main() {
    let mut buf = MyBuffer::new();

    buf.push("Hello".to_string());
    buf.push(" World".to_string());
    buf.push(",Rust!".to_string());
    
    print_content(&buf); 
}