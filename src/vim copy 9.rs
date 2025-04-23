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

pub enum VimMode {
    Normal,
    Insert,
    Command,
    Help, // Added Help mode
}

// Cell formatting options
// In vim.rs, update the CellFormat struct
#[derive(Clone, Default)]
pub struct CellFormat {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub color: Option<Color>,
    pub formula: Option<String>, // Add this field to store formula text
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
            // Exit insert mode
            KeyCode::Esc => {
                self.mode = VimMode::Normal;
            }

            KeyCode::Enter => {
                // Process formula when Enter is pressed
                let format = &mut self.cell_formats[self.cursor_y][self.cursor_x];
                if let Some(formula) = &format.formula {
                    if formula.starts_with("=") {
                        // Try to parse and evaluate the formula
                        self.evaluate_formula(formula);
                    }
                }
            }

            KeyCode::Char('=') if format.formula.is_none() => {
                // Start a formula if '=' is pressed at the beginning
                let format = &mut self.cell_formats[self.cursor_y][self.cursor_x];
                format.formula = Some("=".to_string());
            }

            KeyCode::Char(c) => {
                let format = &mut self.cell_formats[self.cursor_y][self.cursor_x];

                if let Some(formula) = &mut format.formula {
                    // If we're editing a formula, append to it
                    formula.push(c);
                } else {
                    // Otherwise treat as numeric input as before
                    let cell_idx = self.sheet.borrow().get_cell(self.cursor_y, self.cursor_x);
                    let mut sheet = self.sheet.borrow_mut();
                    let mut cell_info = sheet.get(cell_idx);
                    cell_info.value = match cell_info.value {
                        0 => c.to_digit(10).unwrap_or(0) as i32,
                        n => n * 10 + c.to_digit(10).unwrap_or(0) as i32,
                    };
                    sheet.set(cell_idx, cell_info);
                }
            }

            KeyCode::Backspace => {
                let format = &mut self.cell_formats[self.cursor_y][self.cursor_x];

                if let Some(formula) = &mut format.formula {
                    // If editing a formula, remove last character
                    formula.pop();
                    // If formula is now empty, set to None
                    if formula == "=" || formula.is_empty() {
                        format.formula = None;
                    }
                } else {
                    // Otherwise delete last digit as before
                    let cell_idx = self.sheet.borrow().get_cell(self.cursor_y, self.cursor_x);
                    let mut sheet = self.sheet.borrow_mut();
                    let mut cell_info = sheet.get(cell_idx);
                    cell_info.value /= 10;
                    sheet.set(cell_idx, cell_info);
                }
            }

            _ => {}
        }
        false
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
            "  :goto A1    → Jump to cell A1 (column A, row 1)",
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
            "  In insert mode: Type digits to modify the cell value",
            "  Backspace: Delete last digit",
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
            }
            VimMode::Command => {
                execute!(stdout, PrintStyledContent("-- COMMAND --".bold().blue()))?;
                print!(": {}", self.command_buffer);
            }
            VimMode::Help => {
                // This is handled separately in draw_help_menu
                return Ok(());
            }
        }

        // Get the current cell's value and formula (if any)
        let sheet = self.sheet.borrow();
        let current_cell_index = sheet.get_cell(self.cursor_y, self.cursor_x);
        let current_cell = &sheet.data[current_cell_index];
        let current_format = &self.cell_formats[self.cursor_y][self.cursor_x];

        // Formula bar - display at the top
        execute!(stdout, cursor::MoveTo(0, 1))?;
        print!("Formula: ");
        if let Some(formula) = &current_format.formula {
            // Show the formula if it exists
            print!("{}", formula);
        } else {
            // Otherwise just show the raw value
            print!("{}", current_cell.value);
        }

        // Move cursor to beginning of next line for the spreadsheet display
        execute!(stdout, cursor::MoveTo(0, 2))?;
        println!();

        // Rest of the rendering code...
        // ...

        print!("{:3} ", ' ');
        for j in 0..sheet.m.min(20) {
            let col_heading = crate::convert::num_to_alpha((j + 1) as u32);
            print!("{:11} ", col_heading);
        }

        // Print each row
        for i in 0..sheet.n.min(20) {
            // Go to beginning of line
            execute!(stdout, cursor::MoveTo(0, (i + 4) as u16))?;

            // Row number (1-indexed)
            print!("{:3} ", i + 1);

            for j in 0..sheet.m.min(20) {
                let cell_index = sheet.get_cell(i, j);
                let cell = &sheet.data[cell_index];
                let format = &self.cell_formats[i][j];

                // Highlight current cell with red border and content
                if i == self.cursor_y && j == self.cursor_x {
                    if cell.info.invalid {
                        execute!(stdout, PrintStyledContent("[ERR]      ".red().bold()))?;
                    } else {
                        let value_str = format!("{:9}", cell.value);
                        let formatted = format!("[{}]", value_str);
                        execute!(stdout, PrintStyledContent(formatted.red().bold()))?;
                    }
                } else {
                    // Format the value as a string first
                    let value_str = if cell.info.invalid {
                        "ERR".to_string()
                    } else {
                        format!("{}", cell.value)
                    };

                    // Apply formatting with different methods depending on what's needed
                    // Replace the problematic section in the redraw_screen method with this code:
                    if format.bold || format.italic || format.underline || format.color.is_some() {
                        // Create a styled string with the formatting we need
                        let mut styled_content = value_str.stylize();

                        // Apply color if set
                        if let Some(color) = format.color {
                            styled_content = styled_content.with(color);
                        }

                        // Apply text styles conditionally
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
                    } else {
                        // No formatting needed, just print plain
                        print!("{:11} ", value_str);
                    }
                    // if format.bold || format.italic || format.underline || format.color.is_some() {
                    //     // Create a base styled string
                    //     let mut styled = value_str.stylize();

                    //     // Apply all formatting options
                    //     if let Some(color) = format.color {
                    //         styled = styled.with(color);
                    //     }

                    //     if format.bold {
                    //         styled = styled.bold();
                    //     }

                    //     if format.italic {
                    //         styled = styled.italic();
                    //     }

                    //     if format.underline {
                    //         styled = styled.underlined();
                    //     }

                    //     // Format with padding and execute
                    //     let padded = format!("{:11}", value_str);
                    //     execute!(
                    //         stdout,
                    //         PrintStyledContent(
                    //             padded
                    //                 .stylize()
                    //                 .with(if let Some(c) = format.color {
                    //                     c
                    //                 } else {
                    //                     Color::Reset
                    //                 })
                    //                 .bold_if(format.bold)
                    //                 .italic_if(format.italic)
                    //                 .underlined_if(format.underline)
                    //         )
                    //     )?;
                    // } else {
                    //     // No formatting needed, just print plain
                    //     print!("{:11} ", value_str);
                    // }
                }
            }
        }

        // Status line at bottom
        let status_line_y = (sheet.n + 5).min(24) as u16;
        execute!(stdout, cursor::MoveTo(0, status_line_y))?;

        // Show command line if in command mode
        if let VimMode::Command = self.mode {
            print!(":{}", self.command_buffer);
        } else {
            // Show help tip
            print!("Press 'i' for insert mode, ':' for commands, ':h' for help, 'q' to quit");
        }

        // Display error message if any
        if let Some((error_msg, _)) = &self.error_message {
            execute!(stdout, cursor::MoveTo(0, status_line_y + 1))?;
            execute!(stdout, PrintStyledContent(error_msg.as_str().red().bold()))?;
        }

        stdout.flush()?;
        Ok(())
    }
    fn evaluate_formula(&mut self, formula: &str) {
        // Create a command info structure for the parser
        let mut cmd_info = crate::info::CommandInfo::default();
        let cell_idx = self.sheet.borrow().get_cell(self.cursor_y, self.cursor_x);
        cmd_info.lhs_cell = cell_idx as i32;

        // Skip the '=' at the beginning
        let expr = &formula[1..];

        // Try to parse the expression
        match crate::parser::expression_parser(expr, &mut cmd_info.info) {
            Ok(_) => {
                // If parsing succeeded, update the graph with the new expression
                let mut sheet = self.sheet.borrow_mut();
                match crate::graph::update_expression(
                    &mut crate::graph::Graph::new(
                        sheet.n,
                        sheet.m,
                        self.sheet.clone(),
                        // We need to create a memory pool here
                        Rc::new(RefCell::new(crate::list::ListMemPool::new())),
                    ),
                    cell_idx,
                    &cmd_info.info,
                ) {
                    Ok(_) => {
                        // Formula was successfully evaluated
                    }
                    Err(_) => {
                        // Cyclic dependency detected
                        self.set_error_message(format!(
                            "Cyclic dependency in formula: {}",
                            formula
                        ));
                    }
                }
            }
            Err(_) => {
                // Invalid formula syntax
                self.set_error_message(format!("Invalid formula syntax: {}", formula));
            }
        }
    }
}
