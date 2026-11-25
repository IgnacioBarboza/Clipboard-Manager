/// A bounded clipboard history that keeps items in insertion order
/// (oldest first) and evicts the oldest item once it is full.
pub trait CircularLog {
    /// Returns a snapshot of the stored items, oldest first.
    fn get_items(&self) -> Vec<String>;

    /// Appends an item, evicting the oldest one if the log is full.
    fn push(&mut self, content: String);

    /// Removes and returns the item at `selected`.
    fn remove_index(&mut self, selected: usize) -> Result<String, &'static str>;
}
