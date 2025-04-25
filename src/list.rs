// list.rs
//! This module implements a memory pool for linked lists, optimizing dynamic allocations.
use std::cell::RefCell;
use std::rc::Rc;
/// The size of a block in the memory pool.
const BLOCK_SIZE: usize = 1024;
/// Represents a node in the linked list.
#[derive(Debug)]
pub struct Node {
    /// The stored integer data.
    pub data: i32,
    /// A reference to the next node in the list.
    pub next: Option<Rc<RefCell<Node>>>,
}
/// Represents a block of allocated nodes.
#[derive(Debug)]
pub struct Block {
    /// Collection of allocated nodes.
    pub nodes: Vec<Rc<RefCell<Node>>>,
    /// Pointer to the next block.
    pub next: Option<Rc<RefCell<Block>>>,
}
/// Represents a memory pool for managing linked list nodes efficiently
#[derive(Debug)]
pub struct ListMemPool {
    /// Collection of allocated blocks.
    pub blocks: Option<Rc<RefCell<Block>>>,
    /// Head of the free list.
    pub free_list: Option<Rc<RefCell<Node>>>,
}

// Memory pool initialization
impl ListMemPool {
    /// Creates a new empty memory pool.
    ///
    /// # Examples
    /// ```
    /// let pool = ListMemPool::new();
    /// ```
    pub fn new() -> Self {
        Self {
            blocks: None,
            free_list: None,
        }
    }
    /// Adds a new block to the memory pool.
    ///
    /// This function initializes a new block with nodes, linking them into the free list.
    pub fn add_block(&mut self) {
        let mut new_block = Block {
            nodes: Vec::with_capacity(BLOCK_SIZE),
            next: self.blocks.take(),
        };

        // Step 1: Fill with placeholder Rc<RefCell<Node>> (with next = None for now)
        for _ in 0..BLOCK_SIZE {
            new_block.nodes.push(Rc::new(RefCell::new(Node {
                data: 0,
                next: None,
            })));
        }

        // Step 2: Link each node to the next one in the block
        for i in 0..BLOCK_SIZE - 1 {
            let next = Some(new_block.nodes[i + 1].clone());
            new_block.nodes[i].borrow_mut().next = next;
        }

        // Step 3: Link the last node to the current free_list
        new_block.nodes[BLOCK_SIZE - 1].borrow_mut().next = self.free_list.clone();

        // Step 4: Update pool pointers
        let block_rc = Rc::new(RefCell::new(new_block));
        self.free_list = Some(block_rc.borrow().nodes[0].clone());
        self.blocks = Some(block_rc);
    }
    /// Releases all allocated memory by resetting the pool.
    pub fn destroy(&mut self) {
        self.blocks = None;
        self.free_list = None;
    }
    /// Allocates a node from the pool.
    ///
    /// If no free nodes are available, a new block is added.
    ///
    /// # Returns
    /// An `Option<Rc<RefCell<Node>>>` containing the allocated node.
    // Allocate a new node from the pool
    pub fn alloc(&mut self) -> Option<Rc<RefCell<Node>>> {
        if self.free_list.is_none() {
            self.add_block();
        }

        if let Some(node) = self.free_list.take() {
            let next = node.borrow_mut().next.take();
            self.free_list = next;
            Some(node)
        } else {
            None
        }
    }
    /// Returns a node to the pool, adding it to the free list.
    ///
    /// # Arguments
    /// - `node`: The node to be freed.
    // Return a node to the pool
    pub fn free(&mut self, node: Rc<RefCell<Node>>) {
        node.borrow_mut().next = self.free_list.clone();
        self.free_list = Some(node);
    }
}
/// Adds a node at the front of the linked list.
///
/// # Arguments
/// - `head`: The current head of the list.
/// - `value`: The value to insert.
/// - `pool`: The memory pool used for allocation.
pub fn push_front(head: &mut Option<Rc<RefCell<Node>>>, value: i32, pool: &mut ListMemPool) {
    let new_node = pool.alloc().unwrap();

    {
        let mut node = new_node.borrow_mut();
        node.data = value;
        node.next = head.clone();
    }

    *head = Some(new_node);
}
/// Removes a node with the given value from the linked list.
///
/// # Arguments
/// - `head`: The head of the list.
/// - `value`: The value to be erased.
/// - `pool`: The memory pool used for deallocation.
///
/// # Returns
/// `true` if a node was removed, `false` otherwise.
pub fn erase_list(
    head: &mut Option<Rc<RefCell<Node>>>,
    value: i32,
    pool: &mut ListMemPool,
) -> bool {
    if head.is_none() {
        return false;
    }

    let mut current = head.clone();
    let mut prev: Option<Rc<RefCell<Node>>> = None;

    while let Some(node_rc) = current {
        let node_data;
        let next;
        {
            // Create a limited scope for borrowing
            let node = node_rc.borrow();
            node_data = node.data;
            next = node.next.clone();
        } // borrow is dropped here

        if node_data == value {
            // Update links outside of any borrows
            if let Some(prev_node) = &prev {
                let next_in_list = {
                    let mut prev_borrowed = prev_node.borrow_mut();
                    let old_next = prev_borrowed.next.clone();
                    prev_borrowed.next = next.clone();
                    old_next
                };
            } else {
                *head = next.clone();
            }

            // Now free the node - since we're not holding any borrows on it
            pool.free(node_rc.clone());
            return true;
        }

        prev = Some(node_rc);
        current = next;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    // Test that a new pool is empty.
    #[test]
    fn test_new_pool() {
        let pool = ListMemPool::new();
        assert!(pool.blocks.is_none(), "Expected blocks to be None");
        assert!(pool.free_list.is_none(), "Expected free_list to be None");
    }

    // Test that alloc() triggers add_block() and returns a node.
    #[test]
    fn test_add_block_and_alloc() {
        let mut pool = ListMemPool::new();
        // free_list is None initially so calling alloc must add a block.
        let node_opt = pool.alloc();
        assert!(node_opt.is_some(), "alloc() should return a Some(node)");
        let node = node_opt.unwrap();
        // Newly allocated node's data is default (0)
        assert_eq!(node.borrow().data, 0);
    }

    // Test that free() properly reinserts a node into the free_list.
    #[test]
    fn test_free_method() {
        let mut pool = ListMemPool::new();
        let node1 = pool.alloc().unwrap();
        node1.borrow_mut().data = 123;
        // Free the node.
        pool.free(node1.clone());
        // The free list should now start with the freed node.
        let free_node = pool.free_list.unwrap();
        assert_eq!(free_node.borrow().data, 123);
    }

    // Test that destroy() resets the pool.
    #[test]
    fn test_destroy() {
        let mut pool = ListMemPool::new();
        pool.add_block();
        assert!(pool.blocks.is_some());
        assert!(pool.free_list.is_some());
        pool.destroy();
        assert!(pool.blocks.is_none());
        assert!(pool.free_list.is_none());
    }

    // Test push_front by inserting nodes and verifying the sequence.
    #[test]
    fn test_push_front() {
        let mut pool = ListMemPool::new();
        let mut head: Option<Rc<RefCell<Node>>> = None;
        // Insert 10 into the empty list.
        push_front(&mut head, 10, &mut pool);
        assert!(head.is_some());
        assert_eq!(head.as_ref().unwrap().borrow().data, 10);

        // Insert 20 so that it becomes the new head.
        push_front(&mut head, 20, &mut pool);
        assert!(head.is_some());
        assert_eq!(head.as_ref().unwrap().borrow().data, 20);
        // The next node should hold the first inserted value (10).
        let second = head.as_ref().unwrap().borrow().next.clone();
        assert!(second.is_some());
        assert_eq!(second.unwrap().borrow().data, 10);
    }

    // Test erase_list() on an empty list.
    #[test]
    fn test_erase_list_empty() {
        let mut pool = ListMemPool::new();
        let mut head: Option<Rc<RefCell<Node>>> = None;
        let result = erase_list(&mut head, 10, &mut pool);
        assert!(!result, "Erasing from an empty list should return false");
    }

    // Test erasing the head node.
    #[test]
    fn test_erase_list_head() {
        let mut pool = ListMemPool::new();
        let mut head: Option<Rc<RefCell<Node>>> = None;
        // Create list: push_front values 10, then 20, then 30 so that head is 30.
        push_front(&mut head, 10, &mut pool);
        push_front(&mut head, 20, &mut pool);
        push_front(&mut head, 30, &mut pool);
        // List is now: 30 -> 20 -> 10
        let erased = erase_list(&mut head, 30, &mut pool);
        assert!(erased, "Erasing the head value should succeed");
        // Head should now have value 20.
        assert!(head.is_some());
        assert_eq!(head.as_ref().unwrap().borrow().data, 20);
    }

    // Test erasing a middle node.
    #[test]
    fn test_erase_list_middle() {
        let mut pool = ListMemPool::new();
        let mut head: Option<Rc<RefCell<Node>>> = None;
        // Create list: 30 -> 20 -> 10
        push_front(&mut head, 10, &mut pool);
        push_front(&mut head, 20, &mut pool);
        push_front(&mut head, 30, &mut pool);
        // Erase the middle value (20).
        let erased = erase_list(&mut head, 20, &mut pool);
        assert!(erased, "Erasing a middle node should succeed");
        // New list should be: 30 -> 10.
        let first_val = head.as_ref().unwrap().borrow().data;
        let second_val = head
            .as_ref()
            .unwrap()
            .borrow()
            .next
            .as_ref()
            .unwrap()
            .borrow()
            .data;
        assert_eq!(first_val, 30);
        assert_eq!(second_val, 10);
    }

    // Test erasing a value not present in the list.
    #[test]
    fn test_erase_list_not_found() {
        let mut pool = ListMemPool::new();
        let mut head: Option<Rc<RefCell<Node>>> = None;
        push_front(&mut head, 5, &mut pool);
        push_front(&mut head, 15, &mut pool);
        let erased = erase_list(&mut head, 99, &mut pool);
        assert!(!erased, "Erasing a non-existent value should return false");
    }
}
