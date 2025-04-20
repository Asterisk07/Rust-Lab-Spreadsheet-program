// dummy dfs
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
struct Node {
    val: i32,
    neighbors: Vec<Rc<RefCell<Node>>>,
    visited: bool,
}

fn dfs(node: Rc<RefCell<Node>>) {
    let mut node_ref = node.borrow_mut();

    if node_ref.visited {
        return;
    }

    println!("Visiting: {}", node_ref.val);
    node_ref.visited = true;

    let neighbors = node_ref.neighbors.clone(); // clone to avoid double borrow
    drop(node_ref); // drop borrow before recursion

    for neighbor in neighbors {
        dfs(neighbor);
    }
}

fn main() {
    // Create nodes
    let node1 = Rc::new(RefCell::new(Node {
        val: 1,
        neighbors: vec![],
        visited: false,
    }));
    let node2 = Rc::new(RefCell::new(Node {
        val: 2,
        neighbors: vec![],
        visited: false,
    }));
    let node3 = Rc::new(RefCell::new(Node {
        val: 3,
        neighbors: vec![],
        visited: false,
    }));

    // Build connections (undirected graph)
    node1.borrow_mut().neighbors.push(Rc::clone(&node2));
    node2.borrow_mut().neighbors.push(Rc::clone(&node1));

    node2.borrow_mut().neighbors.push(Rc::clone(&node3));
    node3.borrow_mut().neighbors.push(Rc::clone(&node2));

    // Start DFS from node1
    dfs(Rc::clone(&node1));
}
