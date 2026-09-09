pub trait CircularLog {
    // Initialize buffer
    fn new() -> Self;

    // DEBUG, prints content
    fn get_items(&self) -> Vec<String>;

    // Inserts a Value to the back of the buffer
    fn push(&mut self, content: String);

<<<<<<< Updated upstream
    // Removes and returns the value at the selected index
=======
    // Selects the index and retrives the content
>>>>>>> Stashed changes
    fn remove_index(&mut self, selected: usize) -> Result<String, &str>;
}