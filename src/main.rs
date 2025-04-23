// main.rs
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
// mod random;
mod sheet;
mod status;
mod vector;
mod vim; // Add the new vim module

use info::CommandInfo;
use parser::ParserContext;
use status::{StatusCode, print_status, set_status_code, start_time};

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    // Check for vim flag
    let vim_mode = args.iter().any(|arg| arg == "--vim");

    // Need at least rows and columns arguments
    if args.len() < 3 {
        eprintln!(
            "Invalid arguments\nUsage: {} <rows> <columns> [--vim]",
            args[0]
        );
        return Ok(());
    }

    let (n, m) = match sheet::parse_dimensions(&args[1], &args[2]) {
        Ok((n, m)) => (n, m),
        Err(_) => {
            eprintln!("Invalid rows and columns");
            return Ok(());
        }
    };

    // Initialize dimensions
    unsafe {
        sheet::init_dimensions(m, n);
    }

    // Initialize memory pool
    let mem_pool = Rc::new(RefCell::new(list::ListMemPool::new()));
    mem_pool.borrow_mut().add_block();

    // Initialize sheet
    let sheet = Rc::new(RefCell::new(sheet::Sheet::new(n, m)));

    // Initialize graph
    let mut graph = graph::Graph::new(n, m, sheet.clone(), mem_pool.clone());

    // If vim mode flag is present, run in vim mode
    if vim_mode {
        let mut vim_editor = vim::VimEditor::new(sheet.clone());
        return vim_editor.run();
    }

    // Otherwise, run in standard mode
    let mut parser_ctx = ParserContext::new();
    let mut stdout = io::stdout();

    start_time();

    loop {
        if parser_ctx.output_enabled {
            sheet.borrow().display()?;
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

        match graph::update_expression(&mut graph, cmd_info.lhs_cell as usize, &cmd_info.info) {
            Ok(_) => {}
            Err(_) => set_status_code(StatusCode::CyclicDep),
        }
    }
}

fn read_command() -> io::Result<String> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}
