// sheet.rs
use std::cmp::min;
use std::io;

use crate::convert::num_to_alpha;
use crate::info::CellInfo;
use crate::status::StatusCode;

pub const N_MAX: usize = 999;
pub const M_MAX: usize = 18278;

pub static mut N: usize = 0;
pub static mut M: usize = 0;
pub static mut SHEET: Option<Vec<CellInfo>> = None;

pub struct Sheet {
    px: usize,
    py: usize,
}

impl Sheet {
    pub fn new(n: usize, m: usize) -> Self {
        unsafe {
            N = n;
            M = m;

            // Initialize sheet with default values
            let total = n * m;
            SHEET = Some(vec![CellInfo::default(); total]);
        }

        Self { px: 0, py: 0 }
    }

    pub fn set_position(&mut self, x: usize, y: usize) -> Result<(), StatusCode> {
        unsafe {
            if x >= N || y >= M {
                return Err(StatusCode::OutOfBounds);
            }
        }

        self.px = x;
        self.py = y;
        Ok(())
    }

    pub fn scroll(&mut self, dx: isize, dy: isize) -> Result<(), StatusCode> {
        let new_x = self.px.saturating_add_signed(dx);
        let new_y = self.py.saturating_add_signed(dy);

        self.set_position(new_x, new_y)
    }

    pub fn display(&self) -> io::Result<()> {
        unsafe {
            print!("{:3} ", ' ');
            for j in self.py..min(self.py + 10, M) {
                let col_heading = num_to_alpha((j + 1) as u32);
                print!("{:11} ", col_heading);
            }
            println!();

            // Print each row
            for i in self.px..min(self.px + 10, N) {
                // Print the row number (1-indexed) with a field width of 3
                print!("{:3} ", i + 1);
                for j in self.py..min(self.py + 10, M) {
                    let cell_index = get_cell(i, j) as usize;
                    if let Some(sheet) = &SHEET {
                        let cell = &sheet[cell_index];
                        // If the cell's invalid flag is set, print "ERR"
                        if cell.info.invalid != 0 {
                            print!("{:11} ", "ERR");
                        } else {
                            print!("{:11} ", cell.value);
                        }
                    } else {
                        print!("{:11} ", "NULL");
                    }
                }
                println!();
            }
        }

        Ok(())
    }
}

// Helper functions for cell access and validation
pub fn is_valid_cell(r: usize, c: usize) -> bool {
    unsafe { r < N && c < M }
}

pub fn is_valid_range(cell1: usize, cell2: usize) -> bool {
    unsafe { cell1 <= cell2 && (cell1 % M) <= (cell2 % M) && (cell1 / M) <= (cell2 / M) }
}

pub fn get_row(cell: usize) -> usize {
    unsafe { cell / M }
}

pub fn get_column(cell: usize) -> usize {
    unsafe { cell % M }
}

pub fn get_cell(r: usize, c: usize) -> usize {
    unsafe { r * M + c }
}

pub fn get_row_and_column(cell: usize) -> (usize, usize) {
    unsafe {
        let row = cell / M;
        let col = cell % M;
        (row, col)
    }
}

pub fn get(cell: usize) -> CellInfo {
    unsafe {
        if let Some(sheet) = &SHEET {
            sheet[cell].clone()
        } else {
            CellInfo::default()
        }
    }
}

pub fn set(cell: usize, info: CellInfo) {
    unsafe {
        if let Some(sheet) = &mut SHEET {
            sheet[cell] = info;
        }
    }
}

pub fn parse_dimensions(rows_str: &str, cols_str: &str) -> Result<(usize, usize), &'static str> {
    let n: usize = match rows_str.parse() {
        Ok(n) if n > 0 && n <= N_MAX => n,
        _ => return Err("Invalid number of rows"),
    };

    let m: usize = match cols_str.parse() {
        Ok(m) if m > 0 && m <= M_MAX => m,
        _ => return Err("Invalid number of columns"),
    };

    Ok((n, m))
}
