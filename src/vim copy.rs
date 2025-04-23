// vim.rs
use crossterm::{
    cursor,
    event::{self, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::{PrintStyledContent, Stylize},
    terminal,
};
use std::{
    cell::RefCell,
    io::{self, Write, stdout},
    rc::Rc,
    time::Duration,
};

use crate::sheet::Sheet;
use crate::status::{StatusCode, print_status, set_status_code, start_time};

pub enum VimMode {
    Normal,
    Insert,
    Command,
}

pub struct VimEditor {
    sheet: Rc<RefCell<Sheet>>,
    cursor_x: usize,
    cursor_y: usize,
    mode: VimMode,
    command_buffer: String,
    last_status: StatusCode,
}

impl VimEditor {
    pub fn new(sheet: Rc<RefCell<Sheet>>) -> Self {
        Self {
            sheet,
            cursor_x: 0,
            cursor_y: 0,
            mode: VimMode::Normal,
            command_buffer: String::new(),
            last_status: StatusCode::Ok,
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        let mut stdout = io::stdout();

        // Enter alternate screen and enable raw mode
        execute!(stdout, terminal::EnterAlternateScreen)?;
        terminal::enable_raw_mode()?;

        self.redraw_screen()?;

        'main_loop: loop {
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

            KeyCode::Char(c) => {
                // Insert value in the current cell
                let cell_idx = self.sheet.borrow().get_cell(self.cursor_y, self.cursor_x);
                let mut sheet = self.sheet.borrow_mut();
                let mut cell_info = sheet.get(cell_idx);
                cell_info.value = match cell_info.value {
                    0 => c.to_digit(10).unwrap_or(0) as i32,
                    n => n * 10 + c.to_digit(10).unwrap_or(0) as i32,
                };
                sheet.set(cell_idx, cell_info);
            }

            KeyCode::Backspace => {
                // Delete last digit
                let cell_idx = self.sheet.borrow().get_cell(self.cursor_y, self.cursor_x);
                let mut sheet = self.sheet.borrow_mut();
                let mut cell_info = sheet.get(cell_idx);
                cell_info.value /= 10;
                sheet.set(cell_idx, cell_info);
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
                self.execute_command();
                self.mode = VimMode::Normal;
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
        } else {
            self.last_status = StatusCode::InvalidCmd;
        }
    }

    fn redraw_screen(&self) -> io::Result<()> {
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
        }
        println!();
        println!();

        // Display spreadsheet
        let sheet = self.sheet.borrow();

        // Column headers
        print!("{:3} ", ' ');
        for j in 0..sheet.m.min(20) {
            let col_heading = crate::convert::num_to_alpha((j + 1) as u32);
            print!("{:11} ", col_heading);
        }
        println!();

        // Print each row
        for i in 0..sheet.n.min(20) {
            // Row number (1-indexed)
            print!("{:3} ", i + 1);

            for j in 0..sheet.m.min(20) {
                let cell_index = sheet.get_cell(i, j);
                let cell = &sheet.data[cell_index];

                // Highlight current cell
                if i == self.cursor_y && j == self.cursor_x {
                    if cell.info.invalid {
                        execute!(stdout, PrintStyledContent("[ERR]      ".bold().red()))?;
                    } else {
                        execute!(
                            stdout,
                            PrintStyledContent(format!("[{:9}]", cell.value).bold())
                        )?;
                    }
                } else {
                    if cell.info.invalid {
                        print!("{:11} ", "ERR");
                    } else {
                        print!("{:11} ", cell.value);
                    }
                }
            }
            println!();
        }

        // Status line at bottom
        execute!(stdout, cursor::MoveTo(0, (sheet.n + 3).min(24) as u16))?;

        // Show command line if in command mode
        if let VimMode::Command = self.mode {
            print!(":{}", self.command_buffer);
        } else {
            // Show help tip
            print!("Press 'i' for insert mode, ':' for commands, 'q' to quit");
        }

        stdout.flush()?;
        Ok(())
    }
}
