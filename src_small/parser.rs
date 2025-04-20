// ===================== parser.rs =====================
pub enum Operation {
    SetValue(usize, usize, i32),                   // row, col, value
    SetFormula(usize, usize, usize, usize, usize), // row, col, function_id, arg1_idx, arg2_idx
    PrintCell(usize, usize),                       // row, col
    PrintSheet,
    Exit,
}
pub fn parse_excel_style(input: &str) -> Result<Operation, &'static str> {
    let trimmed = input.trim();

    if let Some(eq_pos) = trimmed.find('=') {
        let (lhs, rhs) = trimmed.split_at(eq_pos);
        let lhs = lhs.trim();
        let rhs = &rhs[1..].trim(); // skip '='

        let (row, col) = parse_cell(lhs)?;

        // Case: direct assignment (A1 = 42)
        if let Ok(val) = rhs.parse::<i32>() {
            println!("Parsed: Set cell {} = {}", lhs, val);
            return Ok(Operation::SetValue(row, col, val));
        }

        // Case: single cell reference (A1 = B1)
        if let Ok(idx) = cell_to_index(rhs) {
            println!("Parsed: Set cell {} = {} (as reference)", lhs, rhs);
            return Ok(Operation::SetFormula(row, col, 0, idx, 0)); // function_id 0 = assignment
        }

        // Case: A1 = X op Y
        let operators = ['+', '-', '*', '/'];
        for (function_id, op) in operators.iter().enumerate() {
            if rhs.contains(*op) {
                let parts: Vec<&str> = rhs.split(*op).map(str::trim).collect();
                if parts.len() != 2 {
                    return Err("Invalid arithmetic formula format");
                }

                // Try evaluating as two constants
                if let (Ok(a), Ok(b)) = (parts[0].parse::<i32>(), parts[1].parse::<i32>()) {
                    let result = match *op {
                        '+' => a + b,
                        '-' => a - b,
                        '*' => a * b,
                        '/' => {
                            if b == 0 {
                                return Err("Division by zero");
                            }
                            a / b
                        }
                        _ => unreachable!(),
                    };
                    println!(
                        "Parsed: Evaluated {} {} {} = {} and storing directly",
                        parts[0], op, parts[1], result
                    );
                    return Ok(Operation::SetValue(row, col, result));
                }

                // Mixed or cell references
                let mut arg_mask = 0;

                let arg1_idx = if let Ok(val) = parts[0].parse::<i32>() {
                    val as usize
                } else {
                    arg_mask |= 1;
                    cell_to_index(parts[0])?
                };

                let arg2_idx = if let Ok(val) = parts[1].parse::<i32>() {
                    val as usize
                } else {
                    arg_mask |= 2;
                    cell_to_index(parts[1])?
                };

                println!(
                    "Parsed: Set cell {} = {} {} {} (as formula)",
                    lhs, parts[0], op, parts[1]
                );
                // You may need to store `arg_mask` in cell.info.arg_mask later
                return Ok(Operation::SetFormula(
                    row,
                    col,
                    function_id + 1,
                    arg1_idx,
                    arg2_idx,
                ));
            }
        }

        return Err("Unsupported formula format");
    }

    // Print command like: print A1
    if trimmed.to_ascii_lowercase().starts_with("print") {
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() == 1 {
            println!("Parsed: Print entire sheet");
            return Ok(Operation::PrintSheet);
        } else if parts.len() == 2 {
            let (row, col) = parse_cell(parts[1])?;
            println!("Parsed: Print cell {}", parts[1]);
            return Ok(Operation::PrintCell(row, col));
        } else {
            return Err("Invalid print format");
        }
    }

    if trimmed.eq_ignore_ascii_case("exit") {
        println!("Parsed: Exit command");
        return Ok(Operation::Exit);
    }

    Err("Unrecognized input format")
}

fn parse_cell(cell: &str) -> Result<(usize, usize), &'static str> {
    if cell.is_empty() {
        return Err("Empty cell reference");
    }
    let mut col = 0;
    let mut row_start = 0;

    for (i, c) in cell.chars().enumerate() {
        if c.is_ascii_digit() {
            row_start = i;
            break;
        }
        if !c.is_ascii_alphabetic() {
            return Err("Invalid character in cell reference");
        }
        col = col * 26 + (c.to_ascii_uppercase() as usize - 'A' as usize + 1);
    }

    if row_start == 0 {
        return Err("Invalid cell reference — missing row number");
    }

    let row: usize = cell[row_start..]
        .parse()
        .map_err(|_| "Invalid row number")?;
    Ok((row - 1, col - 1))
}

fn cell_to_index(cell: &str) -> Result<usize, &'static str> {
    let (row, col) = parse_cell(cell)?;
    Ok(row * 10 + col) // assumes 10 columns
}
