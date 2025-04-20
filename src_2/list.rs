// list.rs
use std::cell::RefCell;
use std::rc::Rc;

const BLOCK_SIZE: usize = 1024;

#[derive(Debug)]
pub struct Node {
    pub data: i32,
    pub next: Option<Rc<RefCell<Node>>>,
}

// #[derive(Debug)]
// pub struct Block {
//     pub nodes: Vec<Node>,
//     pub next: Option<Rc<RefCell<Block>>>,
// }

#[derive(Debug)]
pub struct Block {
    pub nodes: Vec<Rc<RefCell<Node>>>,
    pub next: Option<Rc<RefCell<Block>>>,
}

#[derive(Debug)]
pub struct ListMemPool {
    pub blocks: Option<Rc<RefCell<Block>>>,
    pub free_list: Option<Rc<RefCell<Node>>>,
}

// Memory pool initialization
impl ListMemPool {
    pub fn new() -> Self {
        Self {
            blocks: None,
            free_list: None,
        }
    }
}

// Linked list operations
pub struct LinkedList {
    head: Option<Rc<RefCell<Node>>>,
}

impl LinkedList {
    pub fn new() -> Self {
        Self { head: None }
    }

    pub fn push_front(&mut self, value: i32, pool: &mut ListMemPool) {
        let new_node = Rc::new(RefCell::new(Node {
            data: value,
            next: self.head.take(),
        }));
        self.head = Some(new_node);
    }

    pub fn pop_front(&mut self, pool: &mut ListMemPool) {
        if let Some(head) = self.head.take() {
            self.head = head.borrow().next.clone();
            // Add to free list implementation would go here
        }
    }

    // pub fn erase(&mut self, key: i32, pool: &mut ListMemPool) -> bool {
    //     let mut current = self.head.clone();
    //     let mut prev: Option<Rc<RefCell<Node>>> = None;

    //     while let Some(node) = current {
    //         let node_borrow = node.borrow();
    //         if node_borrow.data == key {
    //             if let Some(prev_node) = prev {
    //                 prev_node.borrow_mut().next = node_borrow.next.clone();
    //             } else {
    //                 self.head = node_borrow.next.clone();
    //             }
    //             return true;
    //         }
    //         prev = current.clone();
    //         current = node_borrow.next.clone();
    //     }
    //     false
    // }

    //     pub fn erase(&mut self, key: i32) -> bool {
    //         let mut current = self.head.clone(); // Cloning the Rc
    //         let mut prev: Option<Rc<RefCell<Node>>> = None;

    //         while let Some(node) = current {
    //             let node_borrow = node.borrow();
    //             if node_borrow.data == key {
    //                 if let Some(prev_node) = prev {
    //                     prev_node.borrow_mut().next = node_borrow.next.clone(); // Skip the current node
    //                 } else {
    //                     self.head = node_borrow.next.clone(); // If the first node matches
    //                 }
    //                 return true; // Key was found and erased
    //             }
    //             // prev = Some(current.clone()); // Keep a reference to the previous node
    //             prev = Some(current.clone().unwrap()); // Keep a reference to the previous node
    //             current = node_borrow.next.clone(); // Move to the next node
    //         }
    //         false // Key not found
    //     }

    // pub fn erase(&mut self, key: i32) -> bool {
    //     let mut current = self.head.clone(); // Cloning the Rc<RefCell<Node>>
    //     let mut prev: Option<Rc<RefCell<Node>>> = None;

    //     while let Some(node) = current {
    //         let node_borrow = node.borrow();
    //         if node_borrow.data == key {
    //             if let Some(prev_node) = prev {
    //                 prev_node.borrow_mut().next = node_borrow.next.clone(); // Skip the current node
    //             } else {
    //                 self.head = node_borrow.next.clone(); // If the first node matches
    //             }
    //             return true; // Key was found and erased
    //         }
    //         prev = current.take(); // Keep a reference to the previous node
    //         current = node_borrow.next.clone(); // Move to the next node
    //     }
    //     false // Key not found
    // }

    pub fn erase(&mut self, key: i32) -> bool {
        let mut current = self.head.take(); // Take ownership of the Rc<RefCell<Node>>
        let mut prev: Option<Rc<RefCell<Node>>> = None;

        while let Some(node) = current {
            {
                let node_borrow = node.borrow();
                if node_borrow.data == key {
                    if let Some(prev_node) = &prev {
                        prev_node.borrow_mut().next = node_borrow.next.clone();
                    } else {
                        self.head = node_borrow.next.clone();
                    }
                    return true;
                }
                current = node_borrow.next.clone();
            }
            prev = Some(node); // move node here (outside of borrow scope)
        }

        false // Key not found

        // while let Some(node) = current {
        //     let node_borrow = node.borrow();
        //     if node_borrow.data == key {
        //         if let Some(prev_node) = prev {
        //             prev_node.borrow_mut().next = node_borrow.next.clone(); // Skip the current node
        //         } else {
        //             self.head = node_borrow.next.clone(); // If the first node matches
        //         }
        //         return true; // Key was found and erased
        //     }
        //     prev = current.take(); // Move current into prev (i.e., take ownership)
        //     current = node_borrow.next.clone(); // Move to the next node
        // }
        // false // Key not found
    }
}

// Block management
impl ListMemPool {
    // pub fn add_block(&mut self) {
    //     let mut new_block = Block {
    //         nodes: Vec::with_capacity(BLOCK_SIZE),
    //         next: self.blocks.take(),
    //     };

    //     // Initialize nodes and link them
    //     for i in 0..BLOCK_SIZE {
    //         new_block.nodes.push(Node {
    //             data: 0,
    //             next: if i < BLOCK_SIZE - 1 {
    //                 Some(Rc::new(RefCell::new(new_block.nodes[i + 1])))
    //             } else {
    //                 self.free_list.clone()
    //             },
    //         });
    //     }

    //     let block_rc = Rc::new(RefCell::new(new_block));
    //     self.blocks = Some(block_rc);
    //     self.free_list = Some(Rc::new(RefCell::new(new_block.nodes[0])));
    // }

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

    pub fn destroy(&mut self) {
        self.blocks = None;
        self.free_list = None;
    }
}
