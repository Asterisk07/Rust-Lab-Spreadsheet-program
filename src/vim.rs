// vim.rs
use crossterm::{
    cursor,
    event::{self, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::{Color, Print, PrintStyledContent, Stylize},
    terminal,
};
use std::{
    cell::RefCell,
    io::{self, Write, stdout},
    rc::Rc,
    time::{Duration, Instant},
};

// static const:usize ERROR_DURATION = 5;
const ERROR_DURATION: u64 = 2;
use crate::sheet::Sheet;
use crate::status::{StatusCode, print_status, set_status_code, start_time};
use std::collections::HashMap;
pub enum VimMode {
    Normal,
    Insert,
    Command,
    Help, // Added Help mode
}

// Cell formatting options
#[derive(Clone, Default)]
pub struct CellFormat {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub color: Option<Color>,
}

pub struct VimEditor {
    sheet: Rc<RefCell<Sheet>>,
    cursor_x: usize,
    cursor_y: usize,
    mode: VimMode,
    command_buffer: String,
    last_status: StatusCode,
    error_message: Option<(String, Instant)>, // Error message and when it was shown
    cell_formats: Vec<Vec<CellFormat>>,       // Store formatting for each cell
    current_input: String,                    // Add this field
    cell_expressions: HashMap<usize, String>, // Store expressions by cell index
    // top_row : usize,
    start_row: usize,
    start_col: usize,
    display_rows: usize,
    display_cols: usize,
    col_width: usize,
}

impl VimEditor {
    pub fn new(sheet: Rc<RefCell<Sheet>>) -> Self {
        let n = sheet.borrow().n;
        let m = sheet.borrow().m;
        let formats = vec![vec![CellFormat::default(); m]; n];

        Self {
            sheet,
            cursor_x: 0,
            cursor_y: 0,
            mode: VimMode::Normal,
            command_buffer: String::new(),
            last_status: StatusCode::Ok,
            error_message: None,
            cell_formats: formats,
            current_input: String::new(),
            cell_expressions: HashMap::new(),
            start_row: 0,
            start_col: 0,
            display_rows: 20,
            display_cols: 20,
            col_width: 10,
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        let mut stdout = io::stdout();

        // Enter alternate screen and enable raw mode
        execute!(stdout, terminal::EnterAlternateScreen)?;
        terminal::enable_raw_mode()?;

        self.redraw_screen()?;

        'main_loop: loop {
            // Check if we need to clear error message (after 5 seconds)
            if let Some((_, timestamp)) = &self.error_message {
                if timestamp.elapsed().as_secs() >= ERROR_DURATION {
                    self.error_message = None;
                    self.redraw_screen()?;
                }
            }

            if let Ok(true) = event::poll(Duration::from_millis(100)) {
                if let Ok(event::Event::Key(key_event)) = event::read() {
                    if self.handle_key_event(key_event) {
                        break 'main_loop;
                    }
                    self.redraw_screen()?;
                }
            }
        }

        // Restore terminal
        terminal::disable_raw_mode()?;
        execute!(stdout, terminal::LeaveAlternateScreen)?;

        Ok(())
    }

    fn handle_key_event(&mut self, event: KeyEvent) -> bool {
        match self.mode {
            VimMode::Normal => self.handle_normal_mode(event),
            VimMode::Insert => self.handle_insert_mode(event),
            VimMode::Command => self.handle_command_mode(event),
            VimMode::Help => self.handle_help_mode(event),
        }
    }

    fn handle_normal_mode(&mut self, event: KeyEvent) -> bool {
        match event.code {
            // Quit vim mode
            KeyCode::Char('q') if event.modifiers == KeyModifiers::NONE => {
                return true;
            }

            // Movement keys
            KeyCode::Char('h') | KeyCode::Left => {
                if self.cursor_x > 0 {
                    self.cursor_x -= 1;
                }
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if self.cursor_y < self.sheet.borrow().n - 1 {
                    self.cursor_y += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if self.cursor_y > 0 {
                    self.cursor_y -= 1;
                }
            }
            KeyCode::Char('l') | KeyCode::Right => {
                if self.cursor_x < self.sheet.borrow().m - 1 {
                    self.cursor_x += 1;
                }
            }

            // Enter insert mode
            KeyCode::Char('i') => {
                self.mode = VimMode::Insert;
            }

            // Enter command mode
            KeyCode::Char(':') => {
                self.mode = VimMode::Command;
                self.command_buffer.clear();
            }

            _ => {}
        }
        false
    }

    fn handle_insert_mode(&mut self, event: KeyEvent) -> bool {
        match event.code {
            KeyCode::Esc => {
                self.mode = VimMode::Normal;
                self.current_input.clear();
            }

            KeyCode::Enter => {
                if !self.current_input.is_empty() {
                    let cell_idx = self.sheet.borrow().get_cell(self.cursor_y, self.cursor_x);

                    match self.evaluate_expression(&self.current_input) {
                        Ok(value) => {
                            // Update cell value
                            let mut sheet = self.sheet.borrow_mut();
                            let mut cell_info = sheet.get(cell_idx);
                            cell_info.value = value;
                            cell_info.info.invalid = false;

                            // Set literal_mode = false to indicate this is an expression
                            cell_info.literal_mode = false;

                            sheet.set(cell_idx, cell_info);

                            // Store the expression
                            self.cell_expressions
                                .insert(cell_idx, self.current_input.clone());

                            // Update dependencies after we're done with sheet
                            drop(sheet);
                            self.update_dependent_cells(cell_idx);
                        }
                        Err(err_msg) => {
                            // self.set_error_message(format!("Invalid expression: {} ({})",                                                         // self.current_input, err_msg));
                            self.error_message = Some((
                                format!("Invalid expression: {}", self.current_input),
                                Instant::now(),
                            ));
                        }
                    }

                    self.current_input.clear();
                    self.mode = VimMode::Normal;
                    // self.current_input.clear();
                }
            }

            KeyCode::Char(c) => {
                // Allow alphanumeric chars and operators
                if c.is_alphanumeric() || "+-*/".contains(c) {
                    self.current_input.push(c);
                }
            }

            KeyCode::Backspace => {
                self.current_input.pop();
            }

            _ => {}
        }
        false
    }

    // // Modify the handle_insert_mode function
    // fn handle_insert_mode(&mut self, event: KeyEvent) -> bool {
    //     match event.code {
    //         // Exit insert mode
    //         KeyCode::Esc => {
    //             self.mode = VimMode::Normal;
    //             self.current_input.clear();
    //         }

    //         KeyCode::Enter => {
    //             // Parse and evaluate the expression
    //             if !self.current_input.is_empty() {
    //                 let result = self.evaluate_expression(&self.current_input);

    //                 // Update the cell with the result
    //                 let cell_idx = self.sheet.borrow().get_cell(self.cursor_y, self.cursor_x);
    //                 let mut sheet = self.sheet.borrow_mut();
    //                 let mut cell_info = sheet.get(cell_idx);

    //                 // Store a potential error message
    //                 // let mut error_msg = None;

    //                 // match result {
    //                 //     Ok(value) => {
    //                 //         cell_info.value = value;
    //                 //         cell_info.info.invalid = false;
    //                 //     }
    //                 //     Err(_) => {
    //                 //         // Set cell to error state
    //                 //         cell_info.info.invalid = true;
    //                 //         self.set_error_message(format!(
    //                 //             "Invalid expression: {}",
    //                 //             self.current_input
    //                 //         ));
    //                 //     }
    //                 // }

    //                 match result {
    //                     Ok(value) => {
    //                         cell_info.value = value;
    //                         cell_info.info.invalid = false;
    //                     }
    //                     Err(_) => {
    //                         // Set cell to error state
    //                         cell_info.info.invalid = true;
    //                         // error_msg = Some(format!("Invalid expression: {}", self.current_input));

    //                         // self.set_error_message();
    //                         self.error_message = Some((
    //                             format!("Invalid expression: {}", self.current_input),
    //                             Instant::now(),
    //                         ));

    //                         // self.set_error_message(format!(
    //                         //     "Invalid expression: {} ({})",
    //                         //     self.current_input, error_msg
    //                         // ));
    //                     }
    //                 }
    //                 sheet.set(cell_idx, cell_info);
    //                 // Set error message after sheet is no longer borrowed
    //                 self.current_input.clear();
    //                 // if let Some(msg) = error_msg {
    //                 //     self.set_error_message(msg);
    //                 // }
    //             }
    //         }

    //         KeyCode::Char(c) => {
    //             // Allow alphanumeric characters and operators
    //             if c.is_alphanumeric() || "+-*/".contains(c) {
    //                 self.current_input.push(c);
    //             }
    //         }

    //         KeyCode::Backspace => {
    //             // Remove last character
    //             self.current_input.pop();
    //         }

    //         _ => {}
    //     }
    //     false
    // }

    fn evaluate_expression(&self, expr: &str) -> Result<i32, &'static str> {
        // Check if it's a simple number
        if let Ok(num) = expr.parse::<i32>() {
            return Ok(num);
        }

        // Check for cell references like A1, B2
        if expr
            .chars()
            .next()
            .map_or(false, |c| c.is_ascii_alphabetic())
            && expr.chars().skip(1).all(|c| c.is_ascii_digit())
        {
            return self.get_cell_value(expr);
        }

        // Look for basic arithmetic: val1 op val2
        let operations = ['+', '-', '*', '/'];

        for op in operations {
            if let Some(pos) = expr.find(op) {
                let left = &expr[0..pos];
                let right = &expr[pos + 1..];

                // Get values for left and right operands
                let left_val = if left
                    .chars()
                    .next()
                    .map_or(false, |c| c.is_ascii_alphabetic())
                {
                    self.get_cell_value(left)?
                } else {
                    left.parse::<i32>().map_err(|_| "Invalid left operand")?
                };

                let right_val = if right
                    .chars()
                    .next()
                    .map_or(false, |c| c.is_ascii_alphabetic())
                {
                    self.get_cell_value(right)?
                } else {
                    right.parse::<i32>().map_err(|_| "Invalid right operand")?
                };

                // Perform operation
                match op {
                    '+' => return Ok(left_val + right_val),
                    '-' => return Ok(left_val - right_val),
                    '*' => return Ok(left_val * right_val),
                    '/' => {
                        if right_val == 0 {
                            return Err("Division by zero");
                        }
                        return Ok(left_val / right_val);
                    }
                    _ => unreachable!(),
                }
            }
        }

        Err("Invalid expression format")
    }

    fn get_cell_value(&self, cell_ref: &str) -> Result<i32, &'static str> {
        let col_end = cell_ref
            .chars()
            .position(|c| !c.is_ascii_alphabetic())
            .unwrap_or(cell_ref.len());

        let col_str = &cell_ref[0..col_end];
        let row_str = &cell_ref[col_end..];

        // Convert column letters to number (1-based)
        let col = crate::convert::alpha_to_num(col_str).ok_or("Invalid column reference")?;

        // Parse row (1-based)
        let row = row_str
            .parse::<usize>()
            .map_err(|_| "Invalid row reference")?;

        // Convert to 0-based indices
        let row_idx = row - 1;
        let col_idx = col - 1;

        let sheet = self.sheet.borrow();
        if !sheet.is_valid_cell(row_idx, col_idx) {
            return Err("Cell reference out of bounds");
        }

        let cell_idx = sheet.get_cell(row_idx, col_idx);
        let cell = sheet.get(cell_idx);

        if cell.info.invalid {
            return Err("Referenced cell contains an error");
        }

        Ok(cell.value)
    }

    fn update_dependent_cells(&mut self, changed_cell_idx: usize) {
        let (changed_row, changed_col) = self.sheet.borrow().get_row_and_column(changed_cell_idx);
        let changed_cell_ref = format!(
            "{}{}",
            crate::convert::num_to_alpha((changed_col + 1) as u32),
            changed_row + 1
        );

        // Find cells that depend on the changed cell
        let cells_to_update: Vec<usize> = self
            .cell_expressions
            .iter()
            .filter_map(|(&idx, expr)| {
                if idx != changed_cell_idx && expr.contains(&changed_cell_ref) {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect();

        // Update each dependent cell
        for idx in cells_to_update {
            if let Some(expr) = self.cell_expressions.get(&idx).cloned() {
                match self.evaluate_expression(&expr) {
                    Ok(value) => {
                        let mut sheet = self.sheet.borrow_mut();
                        let mut cell_info = sheet.get(idx);
                        cell_info.value = value;
                        cell_info.info.invalid = false;
                        sheet.set(idx, cell_info);

                        // Continue updating the dependency chain
                        drop(sheet);
                        self.update_dependent_cells(idx);
                    }
                    Err(_) => {
                        // Mark cell as invalid
                        let mut sheet = self.sheet.borrow_mut();
                        let mut cell_info = sheet.get(idx);
                        cell_info.info.invalid = true;
                        sheet.set(idx, cell_info);
                    }
                }
            }
        }
    }

    fn parse_token(&self, token: &str) -> Result<i32, &'static str> {
        // If token is a cell reference
        if !token.is_empty() && token.chars().next().unwrap_or(' ').is_ascii_alphabetic() {
            let col_end = token
                .chars()
                .take_while(|c| c.is_ascii_alphabetic())
                .count();
            let col_str = &token[0..col_end];
            let row_str = &token[col_end..];

            if let Some(col) = crate::convert::alpha_to_num(col_str) {
                if let Ok(row) = row_str.parse::<usize>() {
                    // Adjust for 0-based indexing
                    let col_idx = col - 1;
                    let row_idx = row - 1;

                    if self.sheet.borrow().is_valid_cell(row_idx, col_idx) {
                        let cell_idx = self.sheet.borrow().get_cell(row_idx, col_idx);
                        let cell = self.sheet.borrow().get(cell_idx);

                        if !cell.info.invalid {
                            return Ok(cell.value);
                        } else {
                            return Err("Referenced cell contains an error");
                        }
                    } else {
                        return Err("Invalid cell reference");
                    }
                }
            }
            return Err("Invalid cell reference format");
        }

        // Otherwise treat as a number
        token.trim().parse::<i32>().map_err(|_| "Invalid number")
    }

    fn handle_command_mode(&mut self, event: KeyEvent) -> bool {
        match event.code {
            KeyCode::Esc => {
                self.mode = VimMode::Normal;
                self.command_buffer.clear();
            }

            KeyCode::Enter => {
                // Check for help command first - special case
                if self.command_buffer.trim() == "h" || self.command_buffer.trim() == "help" {
                    self.mode = VimMode::Help;
                } else {
                    self.execute_command();
                    self.mode = VimMode::Normal;
                }
                self.command_buffer.clear();
            }

            KeyCode::Char(c) => {
                self.command_buffer.push(c);
            }

            KeyCode::Backspace => {
                self.command_buffer.pop();
            }

            _ => {}
        }
        false
    }

    fn handle_help_mode(&mut self, event: KeyEvent) -> bool {
        // Exit help mode on Escape key
        if event.code == KeyCode::Esc {
            self.mode = VimMode::Normal;
        }
        false
    }

    fn execute_command(&mut self) {
        let cmd = self.command_buffer.trim();

        if cmd == "q" || cmd == "quit" {
            std::process::exit(0);
        } else if cmd == "w" || cmd == "write" {
            // Save functionality could be implemented here
            self.last_status = StatusCode::Ok;
        } else if cmd.starts_with("maxcols ") {
            if let Some(max_str) = cmd.strip_prefix("setmaxcols ") {
                if let Ok(max) = max_str.parse::<usize>() {
                    if max > 0 && max <= 100 {
                        self.display_cols = max;
                        self.last_status = StatusCode::Ok;
                    }
                }
                self.last_status = StatusCode::InvalidValue;
            }
            // return 0; // Default/error value
        } else if cmd.starts_with("goto ") {
            // Parse cell reference and move cursor
            if let Some(cell_ref) = cmd.strip_prefix("goto ") {
                // Simple A1 style reference parser
                if let Some(col_index) = cell_ref.chars().next().filter(|c| c.is_ascii_uppercase())
                {
                    let col = (col_index as u8 - b'A') as usize;
                    if let Ok(row) = cell_ref[1..].parse::<usize>().map(|r| r - 1) {
                        if col < self.sheet.borrow().m && row < self.sheet.borrow().n {
                            self.cursor_x = col;
                            self.cursor_y = row;
                            self.start_col = col;
                            self.start_row = row;
                            self.last_status = StatusCode::Ok;
                            return;
                        }
                    }
                }

                self.last_status = StatusCode::InvalidCell;
            }
        }
        // Text formatting commands
        else if cmd == "b" {
            // Toggle bold for current cell
            let format = &mut self.cell_formats[self.cursor_y][self.cursor_x];
            format.bold = !format.bold;
            self.last_status = StatusCode::Ok;
        } else if cmd == "i" {
            // Toggle italic for current cell
            let format = &mut self.cell_formats[self.cursor_y][self.cursor_x];
            format.italic = !format.italic;
            self.last_status = StatusCode::Ok;
        } else if cmd == "u" {
            // Toggle underline for current cell
            let format = &mut self.cell_formats[self.cursor_y][self.cursor_x];
            format.underline = !format.underline;
            self.last_status = StatusCode::Ok;
        } else if cmd == "reset" {
            // Reset formatting for current cell
            self.cell_formats[self.cursor_y][self.cursor_x] = CellFormat::default();
            self.last_status = StatusCode::Ok;
        } else if cmd.starts_with("color ") {
            // Change text color
            if let Some(color_name) = cmd.strip_prefix("color ") {
                let color = match color_name.trim().to_lowercase().as_str() {
                    "red" => Some(Color::Red),
                    "green" => Some(Color::Green),
                    "blue" => Some(Color::Blue),
                    "yellow" => Some(Color::Yellow),
                    "cyan" => Some(Color::Cyan),
                    "magenta" => Some(Color::Magenta),
                    "white" => Some(Color::White),
                    "black" => Some(Color::Black),
                    _ => None,
                };

                if let Some(c) = color {
                    self.cell_formats[self.cursor_y][self.cursor_x].color = Some(c);
                    self.last_status = StatusCode::Ok;
                } else {
                    self.set_error_message(format!("Invalid color: {}", color_name));
                    self.last_status = StatusCode::InvalidCmd;
                }
            }
        } else {
            self.set_error_message(format!(
                "Invalid command: {}, type ':h' for list of commands.",
                cmd
            ));
            self.last_status = StatusCode::InvalidCmd;
        }
    }

    fn set_error_message(&mut self, message: String) {
        self.error_message = Some((message, Instant::now()));
    }

    fn draw_help_menu(&self) -> io::Result<()> {
        let mut stdout = io::stdout();
        execute!(
            stdout,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0)
        )?;

        let help_text = [
            "📖 Spreadsheet Vim Mode Help Menu",
            "────────────────────────────────",
            "MOVEMENT:",
            "  h, ←        → Move left",
            "  l, →        → Move right",
            "  k, ↑        → Move up",
            "  j, ↓        → Move down",
            "",
            "EDITING:",
            "  i           → Enter insert mode (for numeric input)",
            "  ESC         → Exit insert mode or command mode",
            "",
            "COMMANDS (type : to enter command mode):",
            "  :h, :help   → Show this help menu",
            "  :goto A1    → Jump to cell A1, also scrolls the sheet to that location.",
            "  :q, :quit   → Quit the program",
            "  :w, :write  → Save (placeholder)",
            "",
            "TEXT FORMATTING:",
            "  :b          → Toggle bold for current cell",
            "  :i          → Toggle italic for current cell",
            "  :u          → Toggle underline for current cell",
            "  :color name → Change text color (red, green, blue, yellow, cyan, magenta)",
            "  :reset      → Remove all formatting",
            "",
            "CELL EDITING:",
            "  In insert mode: Type an expression and press Enter to evaluate",
            "  Expressions can include: numbers, cell references (A1, B2), and operators (+, -, *, /)",
            "  Examples: 15+20, A1*5, B3/2, C1+D2",
            "  Backspace: Delete last character",
            "",
            "────────────────────────────────",
            "Press ESC to return to the spreadsheet.",
        ];

        for (i, line) in help_text.iter().enumerate() {
            execute!(stdout, cursor::MoveTo(0, i as u16))?;
            print!("{}", line);
            execute!(stdout, cursor::MoveTo(0, (i + 1) as u16))?;
        }

        stdout.flush()?;
        Ok(())
    }

    fn redraw_screen(&self) -> io::Result<()> {
        // If we're in help mode, show the help menu and return
        if let VimMode::Help = self.mode {
            return self.draw_help_menu();
        }

        let mut stdout = io::stdout();
        execute!(
            stdout,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0)
        )?;

        // Display mode indicator

        match self.mode {
            VimMode::Normal => {
                execute!(stdout, PrintStyledContent("-- NORMAL --".bold()))?;
            }
            VimMode::Insert => {
                execute!(stdout, PrintStyledContent("-- INSERT --".bold().green()))?;
                // Show current input in insert mode
                if !self.current_input.is_empty() {
                    print!(" Input: {}", self.current_input);
                }
            }
            VimMode::Command => {
                execute!(stdout, PrintStyledContent("-- COMMAND --".bold().blue()))?;
                print!(": {}", self.command_buffer);
            }
            VimMode::Help => {
                return Ok(());
            }
        }

        // Move cursor to beginning of next line
        execute!(stdout, cursor::MoveTo(0, 1))?;
        println!();

        // Display spreadsheet
        let sheet = self.sheet.borrow();
        let COL_WIDTH: usize = self.col_width; // Fixed column width for all cells
        // const COL_WIDTH: usize = 10; // Fixed column width for all cells

        // let display_rows = self.display_rows; // Number of rows to display
        // let display_cols = self.display_cols; // Number of columns to display

        // Column headers
        execute!(stdout, cursor::MoveTo(0, 2))?;
        print!("    "); // Row number column space
        // for j in 0..sheet.m.min(20) {
        //     let col_heading = crate::convert::num_to_alpha((j + 1) as u32);
        //     print!("{:^10}", col_heading); // Centered in COL_WIDTH spaces
        // }

        // Column headers (starting from custom column)
        let start_col = self.start_col;
        let start_row = self.start_row;
        for j in start_col..(start_col + self.display_cols).min(sheet.m) {
            let col_heading = crate::convert::num_to_alpha((j + 1) as u32); // +1 if you want 1-based
            print!("{:^10}", col_heading);
        }

        // Print each row
        for i in start_row..(start_row + self.display_rows).min(sheet.n) {
            execute!(stdout, cursor::MoveTo(0, (i - start_row + 4) as u16))?; // Adjust Y position
            print!("{:3} ", i + 1); // Row number (1-based)

            // Print cells for this row (starting from custom column)
            for j in start_col..(start_col + self.display_cols).min(sheet.m) {
                let cell_index = sheet.get_cell(i, j);
                let cell = &sheet.data[cell_index];
                let format = &self.cell_formats[i][j];

                // Create cell content with fixed width
                let (content, is_error) = if cell.info.invalid {
                    ("ERR".to_string(), true)
                } else {
                    (format!("{}", cell.value), false)
                };

                // Handle cursor cell with consistent width
                // if i == self.cursor_y && j == self.cursor_x {
                //     let cursor_content = if is_error {
                //         format!("[{:^(COL_WIDTH-2)}]", "ERR") // 8 characters between brackets
                //     } else {
                //         format!("[{:^(COL_WIDTH-2)}]", content) // 8 characters between brackets
                //     };
                //     execute!(stdout, PrintStyledContent(cursor_content.red().bold()))?;
                // } else {
                //     // For normal cell - apply padding first, then style
                //     let padded_content = format!("{:^COL_WIDTH}", content);

                if i == self.cursor_y && j == self.cursor_x {
                    let cursor_content = if is_error {
                        format!("[{:^width$}]", "ERR", width = COL_WIDTH - 2)
                    } else {
                        format!("[{:^width$}]", content, width = COL_WIDTH - 2)
                    };
                    execute!(stdout, PrintStyledContent(cursor_content.red().bold()))?;
                } else {
                    let padded_content = format!("{:^width$}", content, width = COL_WIDTH);

                    // Apply formatting to the padded content
                    let mut styled_content = padded_content.stylize();
                    if let Some(color) = format.color {
                        styled_content = styled_content.with(color);
                    }
                    if format.bold {
                        styled_content = styled_content.bold();
                    }
                    if format.italic {
                        styled_content = styled_content.italic();
                    }
                    if format.underline {
                        styled_content = styled_content.underlined();
                    }

                    // Print the styled content
                    execute!(stdout, PrintStyledContent(styled_content))?;
                }
            }
        }

        // Status line - show expression for current cell if applicable
        let status_line_y = (sheet.n.min(20) + 5) as u16;
        execute!(stdout, cursor::MoveTo(0, status_line_y))?;

        if let VimMode::Normal = self.mode {
            let current_cell_idx = sheet.get_cell(self.cursor_y, self.cursor_x);
            if let Some(expr) = self.cell_expressions.get(&current_cell_idx) {
                print!(
                    "Cell: {}{} = {}",
                    crate::convert::num_to_alpha((self.cursor_x + 1) as u32),
                    self.cursor_y + 1,
                    expr
                );
            } else {
                print!("Press 'i' for insert mode, ':' for commands, ':h' for help, 'q' to quit");
            }
        } else if let VimMode::Command = self.mode {
            print!(":{}", self.command_buffer);
        }

        // // Status line at bottom
        // let status_line_y = (sheet.n.min(20) + 5) as u16;
        // execute!(stdout, cursor::MoveTo(0, status_line_y))?;

        // if let VimMode::Command = self.mode {
        //     print!(":{}", self.command_buffer);
        // } else {
        //     print!("Press 'i' for insert mode, ':' for commands, ':h' for help, 'q' to quit");
        // }

        // Display error message if any
        if let Some((error_msg, _)) = &self.error_message {
            execute!(stdout, cursor::MoveTo(0, status_line_y + 1))?;
            execute!(stdout, PrintStyledContent(error_msg.as_str().red().bold()))?;
        }

        stdout.flush()?;
        Ok(())
    }
}
