// ===============================
// main.rs
// ===============================
mod formulas;
mod graph;
mod info;
mod sheet;

use crate::formulas::*;
use crate::graph::Graph;
use crate::info::{Cell, Info};
use crate::sheet::Sheet;
use std::io::{self, BufRead};

enum Operation {
    SetValue(usize, usize, i32),                   // row, col, value
    SetFormula(usize, usize, usize, usize, usize), // row, col, function_id, arg1_idx, arg2_idx
    PrintCell(usize, usize),                       // row, col
    PrintSheet,
    Exit,
}

fn parse_operation(input: &str) -> Result<Operation, &'static str> {
    let parts: Vec<&str> = input.trim().split_whitespace().collect();

    if parts.is_empty() {
        return Err("Empty input");
    }

    match parts[0] {
        "set" => {
            if parts.len() != 4 {
                return Err("Invalid set command. Usage: set <row> <col> <value>");
            }
            let row = parts[1].parse::<usize>().map_err(|_| "Invalid row")?;
            let col = parts[2].parse::<usize>().map_err(|_| "Invalid column")?;
            let value = parts[3].parse::<i32>().map_err(|_| "Invalid value")?;
            Ok(Operation::SetValue(row, col, value))
        }
        "formula" => {
            if parts.len() != 6 {
                return Err(
                    "Invalid formula command. Usage: formula <row> <col> <function_id> <arg1_row> <arg1_col> <arg2_row> <arg2_col>",
                );
            }
            let row = parts[1].parse::<usize>().map_err(|_| "Invalid row")?;
            let col = parts[2].parse::<usize>().map_err(|_| "Invalid column")?;
            let function_id = parts[3]
                .parse::<usize>()
                .map_err(|_| "Invalid function ID")?;
            let arg1 = parts[4].parse::<usize>().map_err(|_| "Invalid arg1")?;
            let arg2 = parts[5].parse::<usize>().map_err(|_| "Invalid arg2")?;
            Ok(Operation::SetFormula(row, col, function_id, arg1, arg2))
        }
        "print" => {
            if parts.len() == 3 {
                let row = parts[1].parse::<usize>().map_err(|_| "Invalid row")?;
                let col = parts[2].parse::<usize>().map_err(|_| "Invalid column")?;
                Ok(Operation::PrintCell(row, col))
            } else if parts.len() == 1 {
                Ok(Operation::PrintSheet)
            } else {
                Err("Invalid print command. Usage: print <row> <col> or just print")
            }
        }
        "exit" => Ok(Operation::Exit),
        _ => Err("Unknown command"),
    }
}

fn update_and_evaluate(
    sheet: &Sheet,
    graph: &mut Graph,
    eval_functions: &[EvalFn],
    modified_cell: usize,
) -> bool {
    // Reset graph for new evaluation
    graph.reset();

    // Perform DFS starting from the modified cell to detect cycles and build evaluation order
    let has_cycle = !graph.dfs(sheet, modified_cell);

    if has_cycle {
        // Mark the cell as invalid due to cycle
        // sheet.cells[modified_cell].borrow_mut().info.invalid = true;
        println!("Operation failed : Circular reference detected!");
        // println!("Circular reference detected! Cell marked as invalid.");
        return false;
    }

    // Evaluate in topological order
    graph.evaluate_order(sheet, eval_functions);
    return true;
}

fn main() {
    // Define available formula functions
    let eval_functions: Vec<EvalFn> = vec![
        assignment, // Function ID 0
        add,        // Function ID 1
                    // Add other functions here as needed
    ];

    // Create a new sheet (e.g., 10x10)
    let rows = 10;
    let cols = 10;
    let sheet = Sheet::new(rows, cols);

    // Initialize dependency graph
    let size = rows * cols;
    let mut graph = Graph::new(size);

    println!("Interactive Spreadsheet Application");
    println!("Commands:");
    println!("  set <row> <col> <value> - Set a direct value");
    println!("  formula <row> <col> <function_id> <arg1> <arg2> - Set a formula");
    println!("  print <row> <col> - Print a specific cell");
    println!("  print - Print the entire sheet");
    println!("  exit - Exit the application");

    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();

    while let Some(Ok(line)) = lines.next() {
        match parse_operation(&line) {
            Ok(Operation::SetValue(row, col, value)) => {
                if row >= rows || col >= cols {
                    println!("Cell ({}, {}) is out of bounds", row, col);
                    continue;
                }

                let idx = sheet.get_index(row, col);

                // Update cell
                {
                    let mut cell = sheet.get_cell_mut(row, col);
                    cell.value = value;
                    cell.info.function_id = 0; // Direct assignment
                    cell.info.arg_mask = 0; // No cell references
                    cell.info.arg[0] = value;
                    cell.info.invalid = false;
                }

                // Clear any existing dependencies for this cell
                graph.nodes[idx].borrow_mut().dependents.clear();

                // Evaluate all cells that might depend on this one
                update_and_evaluate(&sheet, &mut graph, &eval_functions, idx);

                println!("Cell ({}, {}) set to {}", row, col, value);
            }
            Ok(Operation::SetFormula(row, col, function_id, arg1, arg2)) => {
                if row >= rows
                    || col >= cols
                    || arg1 >= size
                    || arg2 >= size
                    || function_id >= eval_functions.len()
                {
                    println!("Invalid parameters");
                    continue;
                }

                let idx = sheet.get_index(row, col);

                // Update cell with formula
                {
                    let mut cell = sheet.get_cell_mut(row, col);
                    cell.info.function_id = function_id;
                    cell.info.arg_mask = 3; // Both arguments are cell references
                    cell.info.arg[0] = arg1 as i32;
                    cell.info.arg[1] = arg2 as i32;
                }

                // Update dependencies
                graph.nodes[idx].borrow_mut().dependents.clear();
                let deps = [arg1, arg2];
                graph.build_dependency(&sheet, idx, &deps);

                // Evaluate and check for cycles
                if update_and_evaluate(&sheet, &mut graph, &eval_functions, idx) {
                    let cell = sheet.get_cell(row, col);
                    println!(
                        "Cell ({}, {}) formula set, value = {}",
                        row, col, cell.value
                    );
                }
            }
            Ok(Operation::PrintCell(row, col)) => {
                if row >= rows || col >= cols {
                    println!("Cell ({}, {}) is out of bounds", row, col);
                    continue;
                }

                let cell = sheet.get_cell(row, col);
                println!(
                    "Cell ({}, {}): {} {}",
                    row,
                    col,
                    cell.value,
                    if cell.info.invalid { "[INVALID]" } else { "" }
                );
            }
            Ok(Operation::PrintSheet) => {
                println!("Spreadsheet Contents:");
                for row in 0..rows {
                    for col in 0..cols {
                        let cell = sheet.get_cell(row, col);
                        if cell.info.function_id > 0 || cell.value != 0 {
                            println!(
                                "Cell ({}, {}): {} {}",
                                row,
                                col,
                                cell.value,
                                if cell.info.invalid { "[INVALID]" } else { "" }
                            );
                        }
                    }
                }
            }
            Ok(Operation::Exit) => {
                println!("Exiting application");
                break;
            }
            Err(msg) => {
                println!("Error: {}", msg);
            }
        }
    }
}
