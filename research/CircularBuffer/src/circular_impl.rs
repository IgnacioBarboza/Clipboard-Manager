use circular_buffer::FixedCircularBuffer;
use crate::clipboard_trait::ClipboardBuffer;

const SIZE: usize = 5;
pub type MyBuffer = FixedCircularBuffer<String, SIZE>;

impl ClipboardBuffer for MyBuffer {
    fn new() -> Self {
        FixedCircularBuffer::<String, SIZE>::new()
    }

    fn get_items(&self) -> Vec<String> {
        self.iter().cloned().collect()
    }

    fn push(&mut self, content: String) {
        self.push_back(content);
    }
}