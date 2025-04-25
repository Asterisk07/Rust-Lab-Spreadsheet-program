// main.rs
//! This module is the entry point for the spreadsheet system.
#![cfg(not(tarpaulin_include))]
#![allow(warnings)] //disable warnings
use crossterm::{ExecutableCommand, terminal};
use std::cell::RefCell;
use std::env;
use std::io::{self, Write};
use std::rc::Rc;

mod basic;
mod compare;
mod convert;
mod formulas;
mod graph;
mod info;
mod list;
mod parser;
mod sheet;
mod status;
mod vector;
mod vim;

use crate::info::CommandInfo;
use crate::info::{CellInfo, Info};
use crate::parser::ParserContext;
use crate::status::{StatusCode, print_status, set_status_code, start_time};

/// Represents a single entry in the undo/redo history.
struct HistoryEntry {
    /// The cell index where the change occurred.
    cell_idx: usize,
    /// Information about the command execution.
    info: Info,
    /// The previous value before the change.
    value: i32,
    /// Whether literal mode was enabled.
    literal_mode: bool,
}
/// The main function that runs the spreadsheet application.
///
/// # Returns
/// An `io::Result<()>` indicating success or failure.
fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    // Check for vim flag
    let vim_mode = args.iter().any(|arg| arg == "--vim");

    if vim_mode {
        if args.len() < 3 {
            eprintln!(
                "Vim mode : Invalid arguments\nUsage: {} <rows> <columns> [--vim]",
                args[0]
            );
            return Ok(());
        }
    } else {
        if args.len() != 3 {
            eprintln!("Invalid arguments\nUsage: {} <rows> <columns>", args[0]);
            return Ok(());
        }
    }

    let (n, m) = match sheet::parse_dimensions(&args[1], &args[2]) {
        Ok((n, m)) => (n, m),
        Err(_) => {
            eprintln!("Invalid rows and columns LMAO");
            return Ok(());
        }
    };

    unsafe {
        sheet::init_dimensions(m, n);
    }

    // Initialize memory pool
    let mem_pool = Rc::new(RefCell::new(list::ListMemPool::new()));
    mem_pool.borrow_mut().add_block();

    // Initialize sheet
    let mut sheet = Rc::new(RefCell::new(sheet::Sheet::new(n, m)));

    // Initialize graph
    let mut graph = graph::Graph::new(n, m, sheet.clone(), mem_pool.clone());

    // If vim mode flag is present, run in vim mode
    if vim_mode {
        // let mut vim_editor = vim::VimEditor::new(sheet.clone());
        // return vim_editor.run();
        // let graph = Rc::new(RefCell::new(graph));
        // let mut vim_editor = vim::VimEditor::new(sheet.clone(), graph);
        let mut vim_editor = vim::VimEditor::new(sheet.clone());
        return vim_editor.run();
    }

    // undo-redo stack initialization !!!
    let mut undo_stack: Vec<HistoryEntry> = Vec::new();
    let mut redo_stack: Vec<HistoryEntry> = Vec::new();

    let mut parser_ctx = ParserContext::new();
    let mut stdout = io::stdout();

    start_time();

    loop {
        if parser_ctx.output_enabled {
            // sheet.display()?;
            sheet.borrow_mut().display(&mut parser_ctx)?; // Borrow for display 
        }

        print_status();
        stdout.flush()?;

        set_status_code(StatusCode::Ok);

        let input = read_command()?;
        status::start_time();

        let cmd_info = match parser::parse(&input, &mut parser_ctx) {
            Ok(info) => info,
            Err(_) => {
                set_status_code(StatusCode::InvalidCmd);
                continue;
            }
        };

        if cmd_info.lhs_cell == -1 {
            continue;
        }
        if cmd_info.lhs_cell == -2 {
            // Handle Undo
            if let Some(entry) = undo_stack.pop() {
                let mut temp_cell_info = CellInfo {
                    info: entry.info.clone(),
                    value: entry.value,
                    literal_mode: entry.literal_mode,
                };

                // Cycle check for old dependencies
                if !graph.iterative_dfs(entry.cell_idx as i32, &temp_cell_info) {
                    undo_stack.push(entry);
                    set_status_code(StatusCode::CyclicDep);
                    continue;
                }

                // Save current state to redo stack
                let (current_info, current_value, current_literal) = {
                    let sheet_borrow = sheet.borrow();
                    (
                        sheet_borrow.data[entry.cell_idx].info.clone(),
                        sheet_borrow.data[entry.cell_idx].value,
                        sheet_borrow.data[entry.cell_idx].literal_mode,
                    )
                };
                redo_stack.push(HistoryEntry {
                    cell_idx: entry.cell_idx,
                    info: current_info,
                    value: current_value,
                    literal_mode: current_literal,
                });

                // Revert the cell state
                graph.delete_expression(entry.cell_idx as i32);
                graph.add_expression(entry.cell_idx as i32, &temp_cell_info);

                {
                    let mut sheet_borrow = sheet.borrow_mut();
                    let cell = &mut sheet_borrow.data[entry.cell_idx];
                    cell.info = entry.info;
                    cell.value = entry.value;
                    cell.literal_mode = true; // Preserve historical value
                }

                graph.update_values();
                graph.reset();
            } else {
                set_status_code(StatusCode::NothingToUndo);
            }
            continue;
        } else if cmd_info.lhs_cell == -3 {
            // Handle Redo (similar structure to undo)
            if let Some(entry) = redo_stack.pop() {
                let mut temp_cell_info = CellInfo {
                    info: entry.info.clone(),
                    value: entry.value,
                    literal_mode: entry.literal_mode,
                };

                if !graph.iterative_dfs(entry.cell_idx as i32, &temp_cell_info) {
                    redo_stack.push(entry);
                    set_status_code(StatusCode::CyclicDep);
                    continue;
                }

                // Save current state to undo stack
                let (current_info, current_value, current_literal) = {
                    let sheet_borrow = sheet.borrow();
                    (
                        sheet_borrow.data[entry.cell_idx].info.clone(),
                        sheet_borrow.data[entry.cell_idx].value,
                        sheet_borrow.data[entry.cell_idx].literal_mode,
                    )
                };
                undo_stack.push(HistoryEntry {
                    cell_idx: entry.cell_idx,
                    info: current_info,
                    value: current_value,
                    literal_mode: current_literal,
                });

                // Apply redo state
                graph.delete_expression(entry.cell_idx as i32);
                graph.add_expression(entry.cell_idx as i32, &temp_cell_info);

                {
                    let mut sheet_borrow = sheet.borrow_mut();
                    let cell = &mut sheet_borrow.data[entry.cell_idx];
                    cell.info = entry.info;
                    cell.value = entry.value;
                    cell.literal_mode = true;
                }

                graph.update_values();
                graph.reset();
            } else {
                set_status_code(StatusCode::NothingToRedo);
            }
            continue;
        }

        let cell_idx = cmd_info.lhs_cell as usize;

        // Save current state to undo stack
        let (current_info, current_value, current_literal) = {
            let sheet_borrow = sheet.borrow();
            (
                sheet_borrow.data[cell_idx].info.clone(),
                sheet_borrow.data[cell_idx].value,
                sheet_borrow.data[cell_idx].literal_mode,
            )
        };
        undo_stack.push(HistoryEntry {
            cell_idx,
            info: current_info,
            value: current_value,
            literal_mode: current_literal,
        });

        match graph::update_expression(&mut graph, cell_idx as usize, &cmd_info.info) {
            Ok(_) => {
                redo_stack.clear();
                sheet.borrow_mut().data[cell_idx].literal_mode = false; // Reset literal mode
            }
            Err(code) => {
                set_status_code(code);
                undo_stack.pop();
            }
        }
    }
}
/// Reads a command from standard input.
///
/// # Returns
/// The trimmed command as a `String`.
fn read_command() -> io::Result<String> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}
