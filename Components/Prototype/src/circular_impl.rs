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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::circular_trait::CircularLog;

    #[test]
    fn buffer_push_items_keep_order() {
        // Arrange
        let mut buffer = Buffer::new();

        // Act
        buffer.push("0".to_string());
        buffer.push("1".to_string());
        buffer.push("2".to_string());

        // Assert
        let items = buffer.get_items();

        for i in 0..=2 {
            assert_eq!(i.to_string(), items[i]);
        }
    }

    #[test]
    fn buffer_overflow_deletes_oldest() {
        // Arrange
        let mut buffer = Buffer::new();

        // Act
        buffer.push("0".to_string());
        buffer.push("1".to_string());
        buffer.push("2".to_string());
        buffer.push("3".to_string());
        buffer.push("4".to_string());
        buffer.push("5".to_string());

        // Assert
        let items = buffer.get_items();

        for i in 0..=4 {
            assert_eq!((i + 1).to_string(), items[i]);
        }
    }

    #[test]
    fn buffer_empty_has_no_items() {
        // Arrange
        let buffer = Buffer::new();

        // Act
        let items = buffer.get_items();

        // Assert
        assert!(items.is_empty());
    }

    #[test]
    fn buffer_remove_index_removes_selected_item() {
        // Arrange
        let mut buffer = Buffer::new();

        buffer.push("0".to_string());
        buffer.push("1".to_string());
        buffer.push("2".to_string());
        buffer.push("3".to_string());
        buffer.push("4".to_string());

        // Act
        let removed = buffer.remove_index(2);

        // Assert
        assert_eq!(removed, Ok("2".to_string()));
        
        let items = buffer.get_items();

        assert_eq!(items, vec![
            "0".to_string(),
            "1".to_string(),
            "3".to_string(),
            "4".to_string(),
        ]);
    }
}