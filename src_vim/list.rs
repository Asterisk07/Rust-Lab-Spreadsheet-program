// list.rs
use std::cell::RefCell;
use std::rc::Rc;

const BLOCK_SIZE: usize = 1024;

#[derive(Debug)]
pub struct Node {
    pub data: i32,
    pub next: Option<Rc<RefCell<Node>>>,
}

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

    // Return a node to the pool
    pub fn free(&mut self, node: Rc<RefCell<Node>>) {
        node.borrow_mut().next = self.free_list.clone();
        self.free_list = Some(node);
    }
}
pub fn push_front(head: &mut Option<Rc<RefCell<Node>>>, value: i32, pool: &mut ListMemPool) {
    let new_node = pool.alloc().unwrap();

    {
        let mut node = new_node.borrow_mut();
        node.data = value;
        node.next = head.clone();
    }

    *head = Some(new_node);
}

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

// pub fn erase_list(
//     head: &mut Option<Rc<RefCell<Node>>>,
//     value: i32,
//     pool: &mut ListMemPool,
// ) -> bool {
//     if head.is_none() {
//         return false;
//     }

//     let mut current = head.clone();
//     let mut prev: Option<Rc<RefCell<Node>>> = None;

//     while let Some(node_rc) = current {
//         let next;
//         {
//             let node = node_rc.borrow();
//             if node.data == value {
//                 if let Some(prev_node) = &prev {
//                     prev_node.borrow_mut().next = node.next.clone();
//                 } else {
//                     *head = node.next.clone();
//                 }
//                 // You might want to return the node to the pool here
//                 pool.free(node_rc.clone());
//                 return true;
//             }
//             next = node.next.clone();
//         }
//         prev = Some(node_rc);
//         current = next;
//     }

//     false
// }

// // Global memory pool initialization
// static mut MEM_POOL: ListMemPool = ListMemPool {
//     blocks: None,
//     free_list: None,
// };

// pub fn init_mem_pool() {
//     unsafe {
//         MEM_POOL = ListMemPool::new();
//         MEM_POOL.add_block();
//     }
// }
