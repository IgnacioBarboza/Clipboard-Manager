use circular_buffer::FixedCircularBuffer;
use crate::circular_trait::CircularLog;

const SIZE: usize = 5;
pub type Buffer = FixedCircularBuffer<String, SIZE>;

impl CircularLog for Buffer {
    fn new() -> Self {
        FixedCircularBuffer::<String, SIZE>::new()
    }

    fn get_items(&self) -> Vec<String> {
        self.iter().cloned().collect()
    }

    fn push(&mut self, content: String) {
        self.push_back(content);
    }

    // Removes and returns the value at the selected index
    fn remove_index(&mut self, selected: usize) -> Result<String, &str> {
        match self.remove(selected) {
            Some(item) => Ok(item),
            None => Err("The buffer has no value at this index"),
        }
    }
}