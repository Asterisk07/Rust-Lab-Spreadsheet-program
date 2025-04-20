use crate::info::Cell;
use crate::sheet::Sheet;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub struct GraphNode {
    pub index: usize,
    pub dependents: Vec<Rc<RefCell<GraphNode>>>,
    pub visited: bool,
    pub in_stack: bool,
}

pub struct Graph {
    pub nodes: Vec<Rc<RefCell<GraphNode>>>,
    pub order: Vec<usize>,
}

impl Graph {
    pub fn new(size: usize) -> Self {
        let mut nodes = Vec::with_capacity(size);

        // Create all nodes
        for i in 0..size {
            nodes.push(Rc::new(RefCell::new(GraphNode {
                index: i,
                dependents: Vec::new(),
                visited: false,
                in_stack: false,
            })));
        }

        Graph {
            nodes,
            order: Vec::new(),
        }
    }

    pub fn build_dependency(&mut self, sheet: &Sheet, cell: usize, deps: &[usize]) {
        for &dep in deps {
            // Add cell as a dependent of dep
            let cell_node = Rc::clone(&self.nodes[cell]);
            self.nodes[dep].borrow_mut().dependents.push(cell_node);
        }
    }

    pub fn dfs(&mut self, sheet: &mut Sheet, u: usize) -> bool {
        let node = Rc::clone(&self.nodes[u]);
        let mut node_ref = node.borrow_mut();

        // If already visited and in stack, we have a cycle
        if node_ref.in_stack {
            return false;
        }

        // If already visited but not in stack, we've already processed this node
        if node_ref.visited {
            return true;
        }

        // Mark as visited and in stack
        node_ref.visited = true;
        node_ref.in_stack = true;

        // Get all dependents to avoid borrowing issues
        let dependents = node_ref.dependents.clone();
        drop(node_ref); // Drop borrow before recursion

        // DFS through all dependents
        for dependent in dependents {
            let dependent_index = dependent.borrow().index;
            if !self.dfs(sheet, dependent_index) {
                return false; // Cycle detected
            }
        }

        // Mark as not in stack anymore
        self.nodes[u].borrow_mut().in_stack = false;

        // Add to evaluation order
        self.order.push(u);

        true
    }

    pub fn evaluate_order(&mut self, sheet: &mut Sheet, eval_fns: &[crate::formulas::EvalFn]) {
        for &idx in self.order.iter().rev() {
            crate::formulas::evaluate(idx, sheet, eval_fns);
            // crate::formulas::evaluate(&mut sheet.cells[idx], sheet, eval_fns);
        }
    }

    pub fn reset(&mut self) {
        for node in &self.nodes {
            let mut node_ref = node.borrow_mut();
            node_ref.visited = false;
            node_ref.in_stack = false;
        }
        self.order.clear();
    }
}
