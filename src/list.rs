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
    pub nodes: Vec<Node>,
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

    pub fn erase(&mut self, key: i32, pool: &mut ListMemPool) -> bool {
        let mut current = self.head.clone();
        let mut prev = None;

        while let Some(node) = current {
            let node_borrow = node.borrow();
            if node_borrow.data == key {
                if let Some(prev_node) = prev {
                    prev_node.borrow_mut().next = node_borrow.next.clone();
                } else {
                    self.head = node_borrow.next.clone();
                }
                return true;
            }
            prev = current.clone();
            current = node_borrow.next.clone();
        }
        false
    }
}

// Block management
impl ListMemPool {
    pub fn add_block(&mut self) {
        let mut new_block = Block {
            nodes: Vec::with_capacity(BLOCK_SIZE),
            next: self.blocks.take(),
        };

        // Initialize nodes and link them
        for i in 0..BLOCK_SIZE {
            new_block.nodes.push(Node {
                data: 0,
                next: if i < BLOCK_SIZE - 1 {
                    Some(Rc::new(RefCell::new(new_block.nodes[i + 1])))
                } else {
                    self.free_list.clone()
                },
            });
        }

        let block_rc = Rc::new(RefCell::new(new_block));
        self.blocks = Some(block_rc);
        self.free_list = Some(Rc::new(RefCell::new(new_block.nodes[0])));
    }

    pub fn destroy(&mut self) {
        self.blocks = None;
        self.free_list = None;
    }
}