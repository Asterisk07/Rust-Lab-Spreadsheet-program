// graph.rs
use std::cell::RefCell;
use std::rc::Rc;

use crate::formulas::{apply_function, is_range_function};
use crate::info::{CellInfo, Info};
use crate::list::{erase_list, push_front, Node};
use crate::sheet::{get_cell, get_column, get_row_and_column};
use crate::status::StatusCode;

// Visit status for DFS
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum VisitStatus {
    NotVisited = 0,
    InStack = 1,
    Visited = 2,
}

// Adjacency list structure
#[derive(Debug)]
pub struct AdjList {
    pub head: Option<Rc<RefCell<Node>>>,
    pub ptr: Option<Rc<RefCell<Node>>>,
}

// Constants for sheet dimensions from sheet.rs
const N: usize = 100; // Default value, will be replaced at runtime
const M: usize = 100; // Default value, will be replaced at runtime

// Static variables needed for graph operations
static mut STACK: Option<Vec<i32>> = None;
static mut ORDER_PTR: usize = 0;
static mut STACK_PTR: usize = 0;
static mut ADJ_LIST: Option<Vec<AdjList>> = None;

// Initialize graph data structures
pub fn init_graph() {
    unsafe {
        STACK = Some(vec![0; N * M]);
        ADJ_LIST = Some(vec![
            AdjList {
                head: None,
                ptr: None,
            };
            N * M
        ]);
        STACK_PTR = 0;
        ORDER_PTR = N * M; // topo order starts from the end
    }
}

// Check if a cell is in the dependency of a formula
pub fn in_dependency(cell: i32, info: &CellInfo) -> bool {
    if is_range_function(info.info.function_id) {
        // Check if cell is in the given range
        let col = get_column(cell);
        return cell >= info.info.arg[0]
            && cell <= info.info.arg[1]
            && col >= get_column(info.info.arg[0])
            && col <= get_column(info.info.arg[1]);
    }

    // Check if cell is one of the direct arguments
    (is_cell_arg1(info.info.arg_mask) && info.info.arg[0] == cell)
        || (is_cell_arg2(info.info.arg_mask) && info.info.arg[1] == cell)
}

// Helper functions to check if arguments are cells
pub fn is_cell_arg1(arg_mask: u8) -> bool {
    arg_mask & 0b1 != 0
}

pub fn is_cell_arg2(arg_mask: u8) -> bool {
    arg_mask & 0b10 != 0
}

// Helper function to modify the graph by adding or removing dependencies
pub fn modify_graph<F>(cell: i32, info: &CellInfo, func: F)
where
    F: Fn(&mut Option<Rc<RefCell<Node>>>, i32),
{
    let adj = unsafe { ADJ_LIST.as_mut().unwrap() };

    if is_range_function(info.info.function_id) {
        // Handle range function dependency
        let (x1, y1) = get_row_and_column(info.info.arg[0]);
        let (x2, y2) = get_row_and_column(info.info.arg[1]);

        for i in x1..=x2 {
            for j in y1..=y2 {
                let x = get_cell(i, j);
                func(&mut adj[x as usize].head, cell);
                adj[x as usize].ptr = adj[x as usize].head.clone(); // Reset pointer
            }
        }
    } else {
        // Handle direct cell arguments
        if is_cell_arg1(info.info.arg_mask) {
            let arg_idx = info.info.arg[0] as usize;
            func(&mut adj[arg_idx].head, cell);
            adj[arg_idx].ptr = adj[arg_idx].head.clone();
        }

        if is_cell_arg2(info.info.arg_mask) {
            let arg_idx = info.info.arg[1] as usize;
            func(&mut adj[arg_idx].head, cell);
            adj[arg_idx].ptr = adj[arg_idx].head.clone();
        }
    }
}

// Delete expression dependencies
pub fn delete_expression(cell: i32) {
    unsafe {
        let sheet = crate::sheet::SHEET.as_ref().unwrap();
        modify_graph(cell, &sheet[cell as usize], erase_list);
    }
}

// Add new expression dependencies
pub fn add_expression(cell: i32, new_info: &CellInfo) {
    modify_graph(cell, new_info, push_front);
}

// Perform iterative DFS to detect cycles and build topological order
pub fn iterative_dfs(cell: i32, new_info: &CellInfo) -> bool {
    let adj = unsafe { ADJ_LIST.as_mut().unwrap() };
    let stack = unsafe { STACK.as_mut().unwrap() };

    unsafe {
        let sheet = crate::sheet::SHEET.as_mut().unwrap();

        // Mark initial cell and push to stack
        sheet[cell as usize].info.visit = VisitStatus::InStack as u8;
        stack[STACK_PTR] = cell;
        STACK_PTR += 1;

        while STACK_PTR > 0 {
            let u = stack[STACK_PTR - 1]; // Top of stack

            if in_dependency(u, new_info) {
                // Found a cycle
                return false;
            }

            // Check if there are unvisited dependencies
            if let Some(ref ptr_node) = adj[u as usize].ptr {
                let v = ptr_node.borrow().data;

                // Move to next dependency for future iteration
                let next = ptr_node.borrow().next.clone();
                adj[u as usize].ptr = next;

                // Check the status of the destination node
                if sheet[v as usize].info.visit == VisitStatus::InStack as u8 {
                    // Cycle detected
                    return false;
                }

                if sheet[v as usize].info.visit == VisitStatus::NotVisited as u8 {
                    // Add unvisited node to stack
                    sheet[v as usize].info.visit = VisitStatus::InStack as u8;
                    stack[STACK_PTR] = v;
                    STACK_PTR += 1;
                }

                continue;
            }

            // All dependencies processed, mark as visited and add to topo order
            sheet[u as usize].info.visit = VisitStatus::Visited as u8;
            ORDER_PTR -= 1;
            stack[ORDER_PTR] = u;
            STACK_PTR -= 1;
        }
    }

    true // No cycles found
}

// Reset all visit statuses after traversal
pub fn reset() {
    let adj = unsafe { ADJ_LIST.as_mut().unwrap() };
    let stack = unsafe { STACK.as_mut().unwrap() };

    unsafe {
        let sheet = crate::sheet::SHEET.as_mut().unwrap();

        // Reset nodes in stack
        for i in 0..STACK_PTR {
            let node_idx = stack[i] as usize;
            sheet[node_idx].info.visit = VisitStatus::NotVisited as u8;
            adj[node_idx].ptr = adj[node_idx].head.clone();
        }

        // Reset nodes in topological order
        for i in ORDER_PTR..N * M {
            let node_idx = stack[i] as usize;
            sheet[node_idx].info.visit = VisitStatus::NotVisited as u8;
            adj[node_idx].ptr = adj[node_idx].head.clone();
        }

        STACK_PTR = 0;
        ORDER_PTR = N * M;
    }
}

// Update values in topological order
pub fn update_values() {
    let stack = unsafe { STACK.as_ref().unwrap() };

    unsafe {
        let sheet = crate::sheet::SHEET.as_mut().unwrap();

        for i in ORDER_PTR..N * M {
            let cell_idx = stack[i] as usize;
            apply_function(&mut sheet[cell_idx]);
        }
    }
}

// Main function to update an expression and its dependencies
// pub fn update_expression(cell: i32, new_info: &CellInfo) -> bool {
//     let status_code = unsafe { &mut crate::status::STATUS_CODE };

//     if !iterative_dfs(cell, new_info) {
//         // Cycle detected
//         *status_code.lock().unwrap() = StatusCode::CyclicDep;
//         reset();
//         return false;
//     }

//     // No cycles, proceed with updates
//     delete_expression(cell);
//     add_expression(cell, new_info);

//     // Update cell info
//     unsafe {
//         let sheet = crate::sheet::SHEET.as_mut().unwrap();
//         sheet[cell as usize] = new_info.clone();
//     }

//     update_values();
//     reset();

//     true
// }

// // In graph.rs, update the signature of update_expression to return Result<(), StatusCode>
// Replace:
// pub fn update_expression(cell: i32, new_info: &CellInfo) -> bool {
// With:
pub fn update_expression(
    cell: usize,
    info: &Info,
    sheet: &mut sheet::Sheet,
) -> Result<(), StatusCode> {
    let new_info = &mut CellInfo {
        info: info.clone(),
        value: 0,
    };

    if !iterative_dfs(cell as i32, new_info) {
        // Cycle detected
        reset();
        return Err(StatusCode::CyclicDep);
    }

    // No cycles, proceed with updates
    delete_expression(cell as i32);
    add_expression(cell as i32, new_info);

    // Update cell info
    unsafe {
        let sheet_data = crate::sheet::SHEET.as_mut().unwrap();
        sheet_data[cell] = new_info.clone();
    }

    update_values();
    reset();

    Ok(())
}
