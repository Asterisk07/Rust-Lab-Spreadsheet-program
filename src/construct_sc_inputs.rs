use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

mod convert {
    pub fn num_to_alpha(num: u32) -> String {
        let mut n = num;
        let mut result = Vec::new();
        
        while n > 0 {
            n -= 1;
            let c = (b'A' + (n % 26) as u8) as char;
            result.push(c);
            n /= 26;
        }
        
        result.iter().rev().collect()
    }
}

fn int_to_cell_ref(cell: usize, cols: usize) -> String {
    let row = cell / cols;
    let col = cell % cols;
    let col_alpha = convert::num_to_alpha((col + 1) as u32);
    format!("{}{}", col_alpha, row + 1)
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage: {} <rows> <columns> <input_file>", args[0]);
        std::process::exit(1);
    }

    let rows: usize = args[1].parse().expect("Invalid rows");
    let cols: usize = args[2].parse().expect("Invalid columns");
    let input_path = &args[3];
    let output_path = "sc_inputs.sc";

    // Initialize output with default cell values
    let mut output_file = File::create(output_path)?;
    for cell in 0..rows * cols {
        let cell_ref = int_to_cell_ref(cell, cols);
        writeln!(&mut output_file, "let {}=@FLOOR(0)", cell_ref)?;
    }

    // Process input file
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);

    for line in reader.lines() {
        let mut line = line?;

        // Insert '@' before spreadsheet functions
        for keyword in &["SUM", "AVG", "MIN", "MAX"] {
            if let Some(pos) = line.find(keyword) {
                line.insert(pos, '@');
            }
        }

        // Split into left/right of assignment
        let parts: Vec<&str> = line.splitn(2, '=').collect();
        if parts.len() != 2 {
            continue;
        }

        let left = parts[0].trim();
        let right = parts[1];
        let ends_with_newline = right.ends_with('\n');

        // Format right side with FLOOR call
        let mut processed = right.trim_end_matches('\n').to_string();
        processed.push(')');

        if ends_with_newline {
            writeln!(&mut output_file, "let {}=@FLOOR({})", left, processed)?;
        } else {
            write!(&mut output_file, "let {}=@FLOOR({})", left, processed)?;
        }
    }

    println!("Conversion complete: {}", output_path);
    Ok(())
}