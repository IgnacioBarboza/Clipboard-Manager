use circular_buffer::FixedCircularBuffer;

use crate::circular_trait::CircularLog;

/// Maximum number of entries kept in the clipboard history.
pub const HISTORY_SIZE: usize = 10;

pub type Buffer = FixedCircularBuffer<String, HISTORY_SIZE>;

impl CircularLog for Buffer {
    fn get_items(&self) -> Vec<String> {
        self.iter().cloned().collect()
    }

    fn push(&mut self, content: String) {
        self.push_back(content);
    }

    fn remove_index(&mut self, selected: usize) -> Result<String, &'static str> {
        self.remove(selected)
            .ok_or("The buffer has no value at this index")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn filled(count: usize) -> Buffer {
        let mut buffer = Buffer::new();
        for i in 0..count {
            buffer.push(i.to_string());
        }
        buffer
    }

    fn strings(range: std::ops::Range<usize>) -> Vec<String> {
        range.map(|i| i.to_string()).collect()
    }

    #[test]
    fn empty_buffer_has_no_items() {
        assert!(Buffer::new().get_items().is_empty());
    }

    #[test]
    fn push_keeps_insertion_order() {
        assert_eq!(filled(3).get_items(), strings(0..3));
    }

    #[test]
    fn overflow_evicts_oldest() {
        let buffer = filled(HISTORY_SIZE + 2);

        assert_eq!(buffer.get_items(), strings(2..HISTORY_SIZE + 2));
    }

    #[test]
    fn remove_index_removes_selected_item() {
        let mut buffer = filled(5);

        assert_eq!(buffer.remove_index(2), Ok("2".to_string()));
        assert_eq!(buffer.get_items(), vec!["0", "1", "3", "4"]);
    }

    #[test]
    fn remove_index_out_of_range_fails() {
        let mut buffer = filled(2);

        assert!(buffer.remove_index(5).is_err());
        assert_eq!(buffer.get_items().len(), 2);
    }
}
