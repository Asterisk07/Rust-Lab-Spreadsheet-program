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

    // Example: A1=42 or B2=C1+D3
    if let Some(eq_pos) = trimmed.find('=') {
        let (lhs, rhs) = trimmed.split_at(eq_pos);
        let lhs = lhs.trim();
        let rhs = &rhs[1..].trim(); // skip '='

        let (row, col) = parse_cell(lhs)?;

        // Case: direct assignment (A1 = 42)
        if let Ok(val) = rhs.parse::<i32>() {
            return Ok(Operation::SetValue(row, col, val));
        }

        // Case: formula (A1 = B2 + C3)
        let (op, function_id) = if rhs.contains('+') {
            ('+', 1) // currently only add is supported
        } else {
            return Err("Only + operator is supported for now");
        };

        let parts: Vec<&str> = rhs.split(op).map(str::trim).collect();
        if parts.len() != 2 {
            return Err("Invalid formula format");
        }

        let arg1_idx = cell_to_index(parts[0])?;
        let arg2_idx = cell_to_index(parts[1])?;

        return Ok(Operation::SetFormula(
            row,
            col,
            function_id,
            arg1_idx,
            arg2_idx,
        ));
    }

    // Print command like: print A1
    if trimmed.to_ascii_lowercase().starts_with("print") {
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() == 1 {
            return Ok(Operation::PrintSheet);
        } else if parts.len() == 2 {
            let (row, col) = parse_cell(parts[1])?;
            return Ok(Operation::PrintCell(row, col));
        } else {
            return Err("Invalid print format");
        }
    }

    if trimmed.eq_ignore_ascii_case("exit") {
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

    let row: usize = cell[row_start..]
        .parse()
        .map_err(|_| "Invalid row number")?;
    Ok((row - 1, col - 1))
}

fn cell_to_index(cell: &str) -> Result<usize, &'static str> {
    let (row, col) = parse_cell(cell)?;
    Ok(row * 10 + col) // assumes 10 columns; adjust if needed
}
