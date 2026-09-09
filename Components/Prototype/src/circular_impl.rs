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

<<<<<<< Updated upstream
    // Removes and returns the value at the selected index
=======
>>>>>>> Stashed changes
    fn remove_index(&mut self, selected: usize) -> Result<String, &str> {
        match self.remove(selected) {
            Some(item) => Ok(item),
            None => Err("The buffer has no value at this index"),
        }
    }
<<<<<<< Updated upstream
}
=======
}

#[test]
    fn buffer_pushitems_keeporder() {
        // Arrange
        let mut buffer = Buffer::new();

        // Act
        buffer.push("0".to_string());
        buffer.push("1".to_string());
        buffer.push("2".to_string());

        // Assert
        let vec = buffer.get_items();
        for i in 0..=2{
            assert_eq!(i.to_string(),vec[i]);
        }
    }

#[test]
    fn buffer_overflow_deletesoldest() {
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
        let vec = buffer.get_items();
        for i in 0..=4{
            assert_eq!((i+1).to_string(),vec[i]);
        }
    }

#[test]
    fn buffer_empty_noneitem(){
        //Arrange
        let buffer = Buffer::new();

        //Act
        let v = buffer.get_items();

        //Assert
        assert!(v.is_empty());
    }

#[test]
    fn buffer_select_notduplicated(){
        //Arrange
        let mut buffer = Buffer::new();
        buffer.push("0".to_string());
        buffer.push("1".to_string());
        buffer.push("2".to_string());
        buffer.push("3".to_string());
        buffer.push("4".to_string());

        //Act
        let content = buffer.remove_index(2);

        //Assert
        let v = buffer.get_items();
        assert_eq!("0",v[0]);
        assert_eq!("1",v[1]);
        assert_eq!("3",v[2]);
        assert_eq!("4",v[3]);

    }
>>>>>>> Stashed changes
