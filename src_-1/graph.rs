// graph.rs
use std::cell::RefCell;
use std::rc::Rc;

use crate::formulas::{apply_function, is_range_function};
use crate::info::{CellInfo, Info};
use crate::list::{ListMemPool, Node, erase_list, push_front};
use crate::status::StatusCode;

// Visit status for DFS
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum VisitStatus {
    NotVisited = 0,
    InStack = 1,
    Visited = 2,
}

// Adjacency list structure
// #[derive(Debug)]
// pub struct AdjList {
//     pub head: Option<Rc<RefCell<Node>>>,
//     pub ptr: Option<Rc<RefCell<Node>>>,
// }

#[derive(Debug, Clone)]
pub struct AdjList {
    pub head: Option<Rc<RefCell<Node>>>,
    pub ptr: Option<Rc<RefCell<Node>>>,
}

// Graph structure to hold state
pub struct Graph {
    pub adj_list: Vec<AdjList>,
    pub stack: Vec<i32>,
    pub order_ptr: usize,
    pub stack_ptr: usize,
    pub mem_pool: Rc<RefCell<ListMemPool>>,
    pub sheet: Rc<RefCell<crate::sheet::Sheet>>,
}

impl Graph {
    // Initialize graph data structures
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

    // Helper functions to check if arguments are cells
    pub fn is_cell_arg1(&self, arg_mask: u8) -> bool {
        arg_mask & 0b1 != 0
    }

    pub fn is_cell_arg2(&self, arg_mask: u8) -> bool {
        arg_mask & 0b10 != 0
    }

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

    // Perform iterative DFS to detect cycles and build topological order
    pub fn iterative_dfs(&mut self, cell: i32, new_info: &CellInfo) -> bool {
        {
            let mut sheet_borrow = self.sheet.borrow_mut();
            // println!("cell is {}", cell);
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

            apply_function(&mut cell_info, &self.sheet);

            let mut sheet_borrow = self.sheet.borrow_mut();
            sheet_borrow.data[cell_idx] = cell_info;
        }
    }

    // Main function to update an expression and its dependencies
    pub fn update_expression(&mut self, cell: usize, info: &Info) -> Result<(), StatusCode> {
        let new_info = &mut CellInfo {
            info: info.clone(),
            value: 0,
        };
        // println!("cell is {}", cell);

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
static mut GRAPH: Option<Graph> = None;

// Initialize graph for dependency tracking
// pub fn init_graph() {
//     unsafe {
//         let sheet = Rc::new(RefCell::new(crate::sheet::Sheet::new(0, 0)));
//         let mem_pool = Rc::new(RefCell::new(ListMemPool::new()));
//         GRAPH = Some(Graph::new(
//             crate::sheet::N_MAX,
//             crate::sheet::M_MAX,
//             sheet,
//             mem_pool,
//         ));
//     }
// }

pub fn update_expression(graph: &mut Graph, cell: usize, info: &Info) -> Result<(), StatusCode> {
    //  println!("cell is {}", cell);
    graph.update_expression(cell, info)
}

// pub fn update_expression(
//     cell: usize,
//     info: &Info,
//     sheet: &Rc<RefCell<crate::sheet::Sheet>>, // Change parameter type
// ) -> Result<(), StatusCode> {
//     unsafe {
//         if let Some(graph) = &mut GRAPH {
//             // Just assign the Rc (no clone needed)
//             graph.sheet = sheet.clone();
//             // Update expression
//             let result = graph.update_expression(cell, info);
//             result
//         } else {
//             Err(StatusCode::InternalError)
//         }
//     }
// }

// // Update expression in the graph
// pub fn update_expression(
//     cell: usize,
//     info: &Info,
//     sheet: &mut crate::sheet::Sheet,
// ) -> Result<(), StatusCode> {
//     let sheet_rc = Rc::new(RefCell::new(sheet.clone()));
//     unsafe {
//         if let Some(graph) = &mut GRAPH {
//             // Update graph's sheet reference
//             graph.sheet = sheet_rc;
//             // Update expression
//             let result = graph.update_expression(cell, info);
//             // Transfer any changes back to the original sheet
//             let updated_sheet = graph.sheet.borrow();
//             *sheet = updated_sheet.clone();
//             result
//         } else {
//             Err(StatusCode::InternalError)
//         }
//     }
// }
