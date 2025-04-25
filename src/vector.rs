// vector.rs
//! This module provides an implementation of a simple dynamic integer vector.

/// A dynamic vector holding `i32` values.
#[derive(Debug)]
pub struct Vector {
    /// Internal storage for elements.
    container: Vec<i32>,
}

impl Vector {
    /// Creates a new empty vector.
    ///
    /// # Examples
    /// ```
    /// let vec = Vector::new();
    /// assert_eq!(vec.container.len(), 0);
    /// ```
    pub fn new() -> Self {
        Vector {
            container: Vec::new(),
        }
    }
    /// Resizes the vector to the given capacity.
    ///
    /// Returns `false` if the new capacity is smaller than the current size.
    ///
    /// # Arguments
    /// - `new_capacity`: Desired new capacity.
    ///
    /// # Examples
    /// ```
    /// let mut vec = Vector::new();
    /// vec.push_back(10);
    /// assert!(vec.resize(20));
    /// ```
    pub fn resize(&mut self, new_capacity: usize) -> bool {
        if new_capacity < self.container.len() {
            return false;
        }
        self.container
            .reserve_exact(new_capacity - self.container.capacity());
        true
    }

    /// Adds an element to the back of the vector.
    ///
    /// # Arguments
    /// - `value`: The `i32` value to add.
    ///
    /// # Examples
    /// ```
    /// let mut vec = Vector::new();
    /// assert!(vec.push_back(42));
    /// ```
    pub fn push_back(&mut self, value: i32) -> bool {
        self.container.push(value);
        true
    }
    /// Returns the last element in the vector, if any.
    ///
    /// # Examples
    /// ```
    /// let mut vec = Vector::new();
    /// vec.push_back(100);
    /// assert_eq!(vec.back(), Some(&100));
    /// ```
    pub fn back(&self) -> Option<&i32> {
        self.container.last()
    }
    /// Removes the last element from the vector.
    ///
    /// If the size becomes significantly smaller than the capacity, it triggers `shrink_to_fit()`.
    ///
    /// # Examples
    /// ```
    /// let mut vec = Vector::new();
    /// vec.push_back(50);
    /// vec.pop_back();
    /// assert_eq!(vec.back(), None);
    /// ```
    pub fn pop_back(&mut self) {
        self.container.pop();
        if self.container.len() <= self.container.capacity() / 4 {
            self.container.shrink_to_fit();
        }
    }
    /// Erases an element with the specified value, if found.
    ///
    /// Returns `true` if the element was removed, `false` otherwise.
    ///
    /// # Arguments
    /// - `key`: The value to remove.
    ///
    /// # Examples
    /// ```
    /// let mut vec = Vector::new();
    /// vec.push_back(30);
    /// assert!(vec.erase(30));
    /// assert!(!vec.erase(99));
    /// ```
    pub fn erase(&mut self, key: i32) -> bool {
        if let Some(pos) = self.container.iter().position(|&x| x == key) {
            self.container.remove(pos);
            true
        } else {
            false
        }
    }
    /// Prints the vector in `(a, b, c, ...)` format.
    ///
    /// # Examples
    /// ```
    /// let mut vec = Vector::new();
    /// vec.push_back(1);
    /// vec.push_back(2);
    /// vec.print();
    /// ```
    pub fn print(&self) {
        print!("(");
        for (i, val) in self.container.iter().enumerate() {
            if i == self.container.len() - 1 {
                print!("{}", val);
            } else {
                print!("{}, ", val);
            }
        }
        println!(")");
    }
}
/// Custom drop implementation.
/// Deallocation is automatically handled by Rust's memory management.
impl Drop for Vector {
    fn drop(&mut self) {
        // Vec automatically handles deallocation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_vector() {
        let vec = Vector::new();
        // New vector is empty.
        assert_eq!(vec.container.len(), 0);
    }

    #[test]
    fn test_push_back_and_back() {
        let mut vec = Vector::new();
        assert!(vec.push_back(10));
        assert!(vec.push_back(20));
        // Back returns the last element
        assert_eq!(vec.back(), Some(&20));
        // The container length should match the number of pushes.
        assert_eq!(vec.container.len(), 2);
    }

    #[test]
    fn test_resize_smaller_than_length() {
        let mut vec = Vector::new();
        // Push elements so that the length is at least 3.
        vec.push_back(1);
        vec.push_back(2);
        vec.push_back(3);
        // Attempting to resize to a capacity smaller than the current length should fail.
        assert!(!vec.resize(2));
        // The vector should still have the same length.
        assert_eq!(vec.container.len(), 3);
    }

    #[test]
    fn test_pop_back_shrink() {
        let mut vec = Vector::new();
        // Insert a bunch of elements.
        for i in 0..20 {
            vec.push_back(i);
        }
        let len_before = vec.container.len();
        vec.pop_back();
        assert_eq!(vec.container.len(), len_before - 1);
        // Now remove elements until length is very low. In the process, shrink_to_fit should be triggered.
        while vec.container.len() > 1 {
            vec.pop_back();
        }
        // After enough pops, the vector's length is 1 and capacity should be adjusted (equal or slightly above 1).
        assert!(vec.container.len() == 1);
        assert!(vec.container.capacity() >= 1);
    }

    #[test]
    fn test_erase_existing() {
        let mut vec = Vector::new();
        vec.push_back(5);
        vec.push_back(10);
        vec.push_back(15);
        // Erase an element that exists.
        let erased = vec.erase(10);
        assert!(erased);
        // Ensure that the value 10 no longer appears.
        assert!(vec.container.iter().all(|&x| x != 10));
        // And the length is reduced by one.
        assert_eq!(vec.container.len(), 2);
    }

    #[test]
    fn test_erase_non_existing() {
        let mut vec = Vector::new();
        vec.push_back(5);
        vec.push_back(10);
        // Erase an element that does not exist.
        let erased = vec.erase(20);
        assert!(!erased);
        // Container length remains unchanged.
        assert_eq!(vec.container.len(), 2);
    }

    #[test]
    fn test_print_function() {
        let mut vec = Vector::new();
        vec.push_back(1);
        vec.push_back(2);
        vec.push_back(3);
        // Call print() to cover its code path.
        // (This test doesn't capture stdout; its purpose is solely to execute the function.)
        vec.print();
    }

    #[test]
    fn test_back_empty() {
        let vec = Vector::new();
        // When empty, back() should return None.
        assert_eq!(vec.back(), None);
    }
}
