// construct_sc_inputs.rs
/*[dependencies]
memchr = "2.5"  */

use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::ptr;
use std::slice;
use std::process;

mod convert;
use convert::*;
mod sheet;
use sheet::*;
mod parser;
use parser::*;
mod formula;
use formulas::*;
mod graph;
use graph::*;
mod status;
use status::*;


const MAX_LINE: usize = 1024;
const N: i32 = 999; 
const M: i32 = 18278; 

fn separate_parts(input: &str, letters: &mut String) -> i32 {
    let mut num_str = String::new(); // Store letters and number as a string

    for ch in input.chars() {
        if ch.is_digit(10) {
            num_str.push(ch);
        } else {
            letters.push(ch);
        }
    }

    let num = if !num_str.is_empty() {
        num_str.parse::<i32>().unwrap_or(0) // Convert numeric part to i32
    } else {
        0
    };
    num
}

fn int_to_cell_ref(lhs : &String, cell : i32, zero_indexed : bool){
    pub fn int_to_cell_ref(lhs: &mut String, cell: i32, zero_indexed: bool) {
        let (mut row, col) = get_row_and_column(cell); // Equivalent to GET_ROW_AND_COLUMN
            if zero_indexed {
            row -= 1;
        }
    
        num_to_alpha(lhs, col + 1);
    
        let row_string = format!("{}", row + 1);
        lhs.push_str(&row_string);
    }
}

pub fn main() {
    // Simulate argv input, replace this with actual argument parsing if necessary
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 {
        eprintln!("Invalid arguments");
        process::exit(1);
    }

    if !parse_sheet_dimensions(&args[1], &args[2]) {
        println!("Invalid rows and columns");
        println!("len of argv = {}", args.len());
        println!("argument 3 {}", args[3]);
        return;
    }

    let input_file_path = &args[3];
    let input_file = File::open(input_file_path).expect("Failed to open input file");
    let output_file_path = "sc_inputs.sc";
    let mut output_file = File::create(output_file_path).expect("Failed to create output file");

    let keywords = ["SUM", "AVG", "MIN", "MAX"];
    let max_cell = N * M;

    for i in 0..max_cell {
        let mut lhs = String::new();
        int_to_cell_ref(&mut lhs, i, false); // Assuming int_to_cell_ref is defined elsewhere
        writeln!(output_file, "let {}=@FLOOR(0)", lhs).expect("Failed to write to output file");
    }

    let reader = BufReader::new(input_file);
    for line_result in reader.lines() {
        let mut line = line_result.expect("Failed to read a line");
        println!("Inputs line : {}", line);

        for &keyword in &keywords {
            if let Some(pos) = line.find(keyword) {
                line.insert(pos, '@'); // Insert '@' at the found position
            }
        }

        if let Some(equal_pos) = line.find('=') {
            let (left_side, right_side) = line.split_at(equal_pos);
            let mut right_side = right_side[1..].to_string(); // Skip '='

            if right_side.ends_with('\n') {
                right_side.pop(); // Remove newline
                right_side.push(')'); // Append ')'
                writeln!(output_file, "let {}=@FLOOR({}\n", left_side, right_side)
                    .expect("Failed to write to output file");
            } else {
                writeln!(output_file, "let {}=@FLOOR({})", left_side, right_side)
                    .expect("Failed to write to output file");
            }
        }
    }

    println!("Written to sc_inputs.sc");
}