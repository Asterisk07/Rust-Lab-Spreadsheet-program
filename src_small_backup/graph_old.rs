// ===============================
// graph.rs
// ===============================
use crate::info::Cell;
use crate::sheet::Sheet;
use std::cell::RefCell;
pub struct Graph {
    pub adj: Vec<Vec<usize>>, // adj[x] = list of dependents of x
    pub order: Vec<usize>,
    pub visited: RefCell<Vec<u8>>,
    // pub visited: Vec<u8>,
    pub stack: Vec<usize>,
}

impl Graph {
    pub fn new(size: usize) -> Self {
        Graph {
            adj: vec![vec![]; size],
            order: Vec::new(),
            // visited: vec![0; size],
            visited: RefCell::new(vec![0; size]), // Wrap the visited vector in RefCell
            stack: Vec::new(),
        }
    }

    pub fn build_dependency(&mut self, sheet: &Sheet, cell: usize, deps: &[usize]) {
        for &dep in deps {
            self.adj[dep].push(cell);
        }
    }

    pub fn dfs(&mut self, sheet: &mut Sheet, u: usize) -> bool {
        let mut visited = self.visited.borrow_mut(); // Borrow mutably inside dfs

        visited[u] = 1;
        self.stack.push(u);

        for &v in &self.adj[u] {
            if visited[v] == 0 {
                if !self.dfs(sheet, v) {
                    return false;
                }
            } else if visited[v] == 1 {
                return false; // Cycle
            }
        }

        visited[u] = 2;
        self.order.push(u);
        true
    }

    pub fn evaluate_order(&mut self, sheet: &mut Sheet, eval_fns: &[crate::formulas::EvalFn]) {
        for &idx in self.order.iter().rev() {
            crate::formulas::evaluate(&mut sheet.cells[idx], sheet, eval_fns);
        }
    }

    pub fn reset(&mut self) {
        self.visited.fill(0);
        self.stack.clear();
        self.order.clear();
    }
}
