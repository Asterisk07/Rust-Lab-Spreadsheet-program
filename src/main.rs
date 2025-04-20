// ===============================
// main.rs
// ===============================
mod formulas;
mod graph;
mod info;
mod sheet;

use crate::formulas::{add, assignment};
use crate::graph::Graph;
use crate::sheet::Sheet;

fn main() {
    let rows = 10;
    let cols = 10;
    let mut sheet = Sheet::new(rows, cols);

    let eval_fns: Vec<formulas::EvalFn> = vec![assignment, add]; // etc.

    // Dummy: set A1 = 5, B1 = A1 + 3
    {
        let a1 = sheet.get_index(0, 0);
        sheet.cells[a1].info.arg_mask = 0;
        sheet.cells[a1].info.arg[0] = 5;
        sheet.cells[a1].info.function_id = 0;

        let b1 = sheet.get_index(0, 1);
        sheet.cells[b1].info.arg_mask = 3;
        sheet.cells[b1].info.arg[0] = a1 as i32;
        sheet.cells[b1].info.arg[1] = 3;
        sheet.cells[b1].info.function_id = 1;
    }

    // Build graph
    let mut graph = Graph::new(rows * cols);
    graph.build_dependency(&sheet, sheet.get_index(0, 1), &[sheet.get_index(0, 0)]);

    // Evaluate
    if graph.dfs(&mut sheet, sheet.get_index(0, 1)) {
        graph.evaluate_order(&mut sheet, &eval_fns);
        println!("B1 = {}", sheet.get_cell(0, 1).value);
    } else {
        println!("Cycle detected");
    }
}
