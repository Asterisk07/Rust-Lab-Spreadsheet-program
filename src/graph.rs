// graph.rs
//! Dependency graph module for formula computation in a spreadsheet-like system.
use std::cell::RefCell;
use std::rc::Rc;

use crate::formulas::{apply_function, is_range_function};
use crate::info::{CellInfo, Info};
use crate::list::{ListMemPool, Node, erase_list, push_front};
use crate::status::StatusCode;
/// Enum representing the visit status of a node during DFS traversal.
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum VisitStatus {
    /// Node has not been visited.
    NotVisited = 0,
    /// Node is in the current DFS call stack.
    InStack = 1,
    /// Node has been fully visited.
    Visited = 2,
}

/// Struct representing an adjacency list node in the graph.
#[derive(Debug, Clone)]
pub struct AdjList {
    /// Pointer to the head of the linked list of adjacent nodes.
    pub head: Option<Rc<RefCell<Node>>>,
    /// Pointer used for traversal during DFS.
    pub ptr: Option<Rc<RefCell<Node>>>,
}
// Graph structure to hold state
/// Represents the dependency graph of the spreadsheet.
pub struct Graph {
    /// Adjacency list of the graph.
    pub adj_list: Vec<AdjList>,
    /// Stack used for DFS traversal.
    pub stack: Vec<i32>,
    /// Pointer to current position in topological order.
    pub order_ptr: usize,
    /// Pointer to the top of the DFS stack.
    pub stack_ptr: usize,
    /// Memory pool for reusing list nodes.
    pub mem_pool: Rc<RefCell<ListMemPool>>,
    /// Reference to the spreadsheet data.
    pub sheet: Rc<RefCell<crate::sheet::Sheet>>,
}

impl Graph {
    // Initialize graph data structures
    /// Creates a new graph for a spreadsheet with given dimensions.
    pub fn new(
        n: usize,
        m: usize,
        sheet: Rc<RefCell<crate::sheet::Sheet>>,
        mem_pool: Rc<RefCell<ListMemPool>>,
    ) -> Self {
        let total_cells = n * m;
        Self {
            adj_list: vec![
                AdjList {
                    head: None,
                    ptr: None,
                };
                total_cells
            ],
            stack: vec![0; total_cells],
            order_ptr: total_cells,
            stack_ptr: 0,
            mem_pool,
            sheet,
        }
    }
    /// Checks if a given cell is a dependency of a formula in another cell.
    // Check if a cell is in the dependency of a formula
    pub fn in_dependency(&self, cell: i32, info: &CellInfo) -> bool {
        let sheet_borrow = self.sheet.borrow();

        if is_range_function(info.info.function_id) {
            // Check if cell is in the given range
            let col = sheet_borrow.get_column(cell as usize);
            return cell >= info.info.arg[0]
                && cell <= info.info.arg[1]
                && col >= sheet_borrow.get_column(info.info.arg[0] as usize)
                && col <= sheet_borrow.get_column(info.info.arg[1] as usize);
        }

        // Check if cell is one of the direct arguments
        (self.is_cell_arg1(info.info.arg_mask) && info.info.arg[0] == cell)
            || (self.is_cell_arg2(info.info.arg_mask) && info.info.arg[1] == cell)
    }
    /// Checks if argument 1 is a reference to a cell.
    // Helper functions to check if arguments are cells
    pub fn is_cell_arg1(&self, arg_mask: u8) -> bool {
        arg_mask & 0b1 != 0
    }
    /// Checks if argument 2 is a reference to a cell.
    pub fn is_cell_arg2(&self, arg_mask: u8) -> bool {
        arg_mask & 0b10 != 0
    }
    /// Generalized function to modify the dependency graph using a passed-in function
    // Helper function to modify the graph by adding or removing dependencies
    pub fn modify_graph<F>(&mut self, cell: i32, info: &CellInfo, func: F)
    where
        F: Fn(&mut Option<Rc<RefCell<Node>>>, i32, &mut Rc<RefCell<ListMemPool>>),
    {
        let sheet_borrow = self.sheet.borrow();

        if is_range_function(info.info.function_id) {
            // Handle range function dependency
            let (x1, y1) = sheet_borrow.get_row_and_column(info.info.arg[0] as usize);
            let (x2, y2) = sheet_borrow.get_row_and_column(info.info.arg[1] as usize);

            for i in x1..=x2 {
                for j in y1..=y2 {
                    let x = sheet_borrow.get_cell(i, j);
                    func(&mut self.adj_list[x].head, cell, &mut self.mem_pool);
                    self.adj_list[x].ptr = self.adj_list[x].head.clone(); // Reset pointer
                }
            }
        } else {
            // Handle direct cell arguments
            if self.is_cell_arg1(info.info.arg_mask) {
                let arg_idx = info.info.arg[0] as usize;
                func(&mut self.adj_list[arg_idx].head, cell, &mut self.mem_pool);
                self.adj_list[arg_idx].ptr = self.adj_list[arg_idx].head.clone();
            }

            if self.is_cell_arg2(info.info.arg_mask) {
                let arg_idx = info.info.arg[1] as usize;
                func(&mut self.adj_list[arg_idx].head, cell, &mut self.mem_pool);
                self.adj_list[arg_idx].ptr = self.adj_list[arg_idx].head.clone();
            }
        }
    }
    /// Removes all dependencies of a given cell's expression from the graph.
    // Delete expression dependencies
    pub fn delete_expression(&mut self, cell: i32) {
        let mut sheet_borrow = self.sheet.borrow_mut();
        let cell_info = sheet_borrow.data[cell as usize].clone();
        drop(sheet_borrow); // Release the borrow before calling modify_graph

        // self.modify_graph(cell, &cell_info, |head, value, mem_pool| {
        //     erase_list(head, value);
        // });

        self.modify_graph(cell, &cell_info, |head, value, mem_pool| {
            let mut pool = mem_pool.borrow_mut();
            erase_list(head, value, &mut pool);
        });
    }
    /// Adds a new expression's dependencies into the graph.
    // Add new expression dependencies
    pub fn add_expression(&mut self, cell: i32, new_info: &CellInfo) {
        // self.modify_graph(cell, new_info, |head, value, mem_pool| {
        //     push_front(head, value);
        // });

        self.modify_graph(cell, new_info, |head, value, mem_pool| {
            let mut pool = mem_pool.borrow_mut();
            push_front(head, value, &mut pool);
        });
    }
    /// Performs a non-recursive DFS to detect cycles and build topological order.
    // Perform iterative DFS to detect cycles and build topological order
    pub fn iterative_dfs(&mut self, cell: i32, new_info: &CellInfo) -> bool {
        {
            let mut sheet_borrow = self.sheet.borrow_mut();
            // Mark initial cell and push to stack
            sheet_borrow.data[cell as usize].info.visit = VisitStatus::InStack as u8;
        }

        self.stack[self.stack_ptr] = cell;
        self.stack_ptr += 1;

        while self.stack_ptr > 0 {
            let u = self.stack[self.stack_ptr - 1]; // Top of stack

            if self.in_dependency(u, new_info) {
                // Found a cycle
                return false;
            }

            // Check if there are unvisited dependencies
            if let Some(ref ptr_node) = self.adj_list[u as usize].ptr {
                let v = ptr_node.borrow().data;

                // Move to next dependency for future iteration
                let next = ptr_node.borrow().next.clone();
                self.adj_list[u as usize].ptr = next;

                // Check the status of the destination node
                let v_status = {
                    let sheet_borrow = self.sheet.borrow();
                    sheet_borrow.data[v as usize].info.visit
                };

                if v_status == VisitStatus::InStack as u8 {
                    // Cycle detected
                    return false;
                }

                if v_status == VisitStatus::NotVisited as u8 {
                    // Add unvisited node to stack
                    {
                        let mut sheet_borrow = self.sheet.borrow_mut();
                        sheet_borrow.data[v as usize].info.visit = VisitStatus::InStack as u8;
                    }

                    self.stack[self.stack_ptr] = v;
                    self.stack_ptr += 1;
                }

                continue;
            }

            // All dependencies processed, mark as visited and add to topo order
            {
                let mut sheet_borrow = self.sheet.borrow_mut();
                sheet_borrow.data[u as usize].info.visit = VisitStatus::Visited as u8;
            }

            self.order_ptr -= 1;
            self.stack[self.order_ptr] = u;
            self.stack_ptr -= 1;
        }

        true // No cycles found
    }
    /// Resets visit statuses and graph traversal pointers.
    // Reset all visit statuses after traversal
    pub fn reset(&mut self) {
        let n_cells = {
            let sheet_borrow = self.sheet.borrow();
            sheet_borrow.n * sheet_borrow.m
        };

        // Reset nodes in stack
        for i in 0..self.stack_ptr {
            let node_idx = self.stack[i] as usize;
            let mut sheet_borrow = self.sheet.borrow_mut();
            sheet_borrow.data[node_idx].info.visit = VisitStatus::NotVisited as u8;
            drop(sheet_borrow);

            self.adj_list[node_idx].ptr = self.adj_list[node_idx].head.clone();
        }

        // Reset nodes in topological order
        for i in self.order_ptr..n_cells {
            let node_idx = self.stack[i] as usize;
            let mut sheet_borrow = self.sheet.borrow_mut();
            sheet_borrow.data[node_idx].info.visit = VisitStatus::NotVisited as u8;
            drop(sheet_borrow);

            self.adj_list[node_idx].ptr = self.adj_list[node_idx].head.clone();
        }

        self.stack_ptr = 0;
        self.order_ptr = n_cells;
    }
    /// Recomputes values for all cells in topological order.
    // Update values in topological order
    pub fn update_values(&mut self) {
        let n_cells = {
            let sheet_borrow = self.sheet.borrow();
            sheet_borrow.n * sheet_borrow.m
        };

        for i in self.order_ptr..n_cells {
            let cell_idx = self.stack[i] as usize;
            let mut sheet_borrow = self.sheet.borrow_mut();
            let mut cell_info = sheet_borrow.data[cell_idx].clone();
            drop(sheet_borrow);

            // Only compute if not in literal mode
            if !cell_info.literal_mode {
                apply_function(&mut cell_info, &self.sheet);
            }

            let mut sheet_borrow = self.sheet.borrow_mut();
            sheet_borrow.data[cell_idx] = cell_info;
        }
    }
    /// Updates a cell's expression and its dependency graph.
    ///
    /// Returns `Err(StatusCode::CyclicDep)` if a cycle is detected.
    // Main function to update an expression and its dependencies
    pub fn update_expression(&mut self, cell: usize, info: &Info) -> Result<(), StatusCode> {
        let new_info = &mut CellInfo {
            info: info.clone(),
            value: 0,
            literal_mode: false,
        };

        if !self.iterative_dfs(cell as i32, new_info) {
            // Cycle detected
            self.reset();
            return Err(StatusCode::CyclicDep);
        }

        // No cycles, proceed with updates
        self.delete_expression(cell as i32);
        self.add_expression(cell as i32, new_info);

        // Update cell info
        {
            let mut sheet_borrow = self.sheet.borrow_mut();
            sheet_borrow.data[cell] = new_info.clone();
        }

        self.update_values();
        self.reset();

        Ok(())
    }
}

// Global graph instance
/// Global graph used across the spreadsheet engine.
static mut GRAPH: Option<Graph> = None;

// Initialize graph for dependency tracking
/// Initializes the global dependency graph.
pub fn init_graph() {
    unsafe {
        let sheet = Rc::new(RefCell::new(crate::sheet::Sheet::new(0, 0)));
        let mem_pool = Rc::new(RefCell::new(ListMemPool::new()));
        GRAPH = Some(Graph::new(
            crate::sheet::N_MAX(),
            crate::sheet::M_MAX(),
            sheet,
            mem_pool,
        ));
    }
}
/// Public wrapper to update expression using an external graph instance.
pub fn update_expression(graph: &mut Graph, cell: usize, info: &Info) -> Result<(), StatusCode> {
    graph.update_expression(cell, info)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sheet::Sheet;
    use std::cell::RefCell;
    use std::rc::Rc;

    fn create_test_graph() -> Graph {
        let sheet = Rc::new(RefCell::new(Sheet::new(3, 3)));
        let mem_pool = Rc::new(RefCell::new(ListMemPool::new()));
        Graph::new(3, 3, sheet, mem_pool)
    }

    fn create_cell_info(function_id: u8, arg: [i32; 2], arg_mask: u8) -> CellInfo {
        CellInfo {
            info: Info {
                function_id,
                arg,
                arg_mask,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[test]
    fn test_cycle_detection() {
        let mut graph = create_test_graph();
        let cell_idx = {
            let mut sheet = graph.sheet.borrow_mut();
            let cell_idx = sheet.get_cell(0, 0);
            sheet.data[cell_idx] = create_cell_info(0, [cell_idx as i32, 0], 0b1);
            cell_idx
        };

        let info = graph.sheet.borrow().data[cell_idx].info.clone();
        let result = graph.update_expression(cell_idx, &info);
        assert_eq!(result, Err(StatusCode::CyclicDep));
    }

    #[test]
    fn test_valid_dependency_chain() {
        let mut graph = create_test_graph();
        let (a1, b1, c1) = {
            let mut sheet = graph.sheet.borrow_mut();
            let a1 = sheet.get_cell(0, 0);
            let b1 = sheet.get_cell(0, 1);
            let c1 = sheet.get_cell(0, 2);

            sheet.data[b1] = create_cell_info(2, [a1 as i32, 0], 0b1);
            sheet.data[c1] = create_cell_info(2, [b1 as i32, 0], 0b1);
            (a1, b1, c1)
        };

        let info = graph.sheet.borrow().data[c1].info.clone();
        let result = graph.update_expression(c1, &info);
        assert!(result.is_ok());
    }

    #[test]
    fn test_range_dependencies() {
        let mut graph = create_test_graph();
        let target_cell = {
            let mut sheet = graph.sheet.borrow_mut();
            let cell = sheet.get_cell(2, 2);
            sheet.data[cell] = create_cell_info(8, [0, 5], 0b11);
            cell
        };

        let info = graph.sheet.borrow().data[target_cell].info.clone();
        let result = graph.update_expression(target_cell, &info);
        assert!(result.is_ok());
    }

    #[test]
    fn test_in_dependency_checks() {
        let graph = create_test_graph();
        let info = create_cell_info(2, [1, 2], 0b11);
        assert!(graph.in_dependency(1, &info));
        assert!(graph.in_dependency(2, &info));
    }

    #[test]
    fn test_graph_reset() {
        let mut graph = create_test_graph();
        graph.stack_ptr = 5;
        graph.order_ptr = 10;
        graph.reset();

        assert_eq!(graph.stack_ptr, 0);
        assert_eq!(graph.order_ptr, 9);
    }

    #[test]
    fn test_dependency_management() {
        let mut graph = create_test_graph();
        let cell_idx = {
            let mut sheet = graph.sheet.borrow_mut();
            let cell_idx = sheet.get_cell(0, 0);
            sheet.data[cell_idx] = create_cell_info(2, [1, 2], 0b11);
            cell_idx
        };

        let cell_data = graph.sheet.borrow().data[cell_idx].clone();
        graph.add_expression(cell_idx as i32, &cell_data);
        assert!(graph.adj_list[1].head.is_some());

        graph.delete_expression(cell_idx as i32);
        assert!(graph.adj_list[1].head.is_none());
    }
}
