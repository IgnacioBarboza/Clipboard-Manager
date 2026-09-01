use circular_buffer::FixedCircularBuffer;
use crate::circular_trait::CircularLog;

const SIZE: usize = 5;
pub type Buffer = FixedCircularBuffer<String, SIZE>;

impl ClipboardBuffer for Buffer {
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