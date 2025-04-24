// sheet.rs
use std::cell::RefCell;
use std::cmp::min;
use std::io;
use std::rc::Rc;

use crate::convert::num_to_alpha;
use crate::info::CellInfo;
use crate::parser::ParserContext;
use crate::status::StatusCode;

// pub const N_MAX: usize = 999;
// pub const M_MAX: usize = 18278;

pub const N_GLOBAL_MAX: usize = 999;
pub const M_GLOBAL_MAX: usize = 18278;

// pub static mut M_MAX: usize = 0;
// pub static mut N_MAX: usize = 0;

static mut M_INTERNAL: usize = 0;
static mut N_INTERNAL: usize = 0;
static mut INIT_DONE: bool = false;

pub unsafe fn init_dimensions(m: usize, n: usize) {
    if INIT_DONE {
        panic!("Already initialized");
    }
    M_INTERNAL = m;
    N_INTERNAL = n;
    INIT_DONE = true;
}

pub fn M_MAX() -> usize {
    unsafe {
        if !INIT_DONE {
            panic!("M not initialized!");
        }
        M_INTERNAL
    }
}

pub fn N_MAX() -> usize {
    unsafe {
        if !INIT_DONE {
            panic!("N not initialized!");
        }
        N_INTERNAL
    }
}

pub struct Sheet {
    pub data: Vec<CellInfo>,
    pub n: usize,
    pub m: usize,
    pub px: usize,
    pub py: usize,
}

impl Sheet {
    pub fn new(n: usize, m: usize) -> Self {
        // Initialize sheet with default values
        let total = n * m;

        Self {
            data: vec![CellInfo::default(); total],
            n,
            m,
            px: 0,
            py: 0,
        }
    }

    pub fn set_position(&mut self, x: usize, y: usize) -> Result<(), StatusCode> {
        if x >= self.n || y >= self.m {
            return Err(StatusCode::OutOfBounds);
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

    pub fn display(&mut self, context: &mut ParserContext) -> io::Result<()> {
        self.px = context.px;
        self.py = context.py;
        print!("{:3} ", ' '); // Space for row numbers column
        for j in self.py..min(self.py + 10, self.m) {
            let col_heading = num_to_alpha((j + 1) as u32);
            print!("{:>11} ", col_heading); // Right-align headers
        }
        println!();

        // Print each row
        for i in self.px..min(self.px + 10, self.n) {
            print!("{:3} ", i + 1); // Row number right-aligned in 3 characters
            for j in self.py..min(self.py + 10, self.m) {
                let cell_index = self.get_cell(i, j);
                let cell = &self.data[cell_index];

                if cell.info.invalid {
                    print!("{:>11} ", "ERR"); // Right-align "ERR"
                } else {
                    print!("{:>11} ", cell.value); // Right-align cell value
                }
            }
            println!();
        }

        Ok(())
    }

    // Helper functions for cell access and validation
    pub fn is_valid_cell(&self, r: usize, c: usize) -> bool {
        r < self.n && c < self.m
    }

    pub fn is_valid_range(&self, cell1: usize, cell2: usize) -> bool {
        cell1 <= cell2
            && (cell1 % self.m) <= (cell2 % self.m)
            && (cell1 / self.m) <= (cell2 / self.m)
    }

    pub fn get_row(&self, cell: usize) -> usize {
        cell / self.m
    }

    pub fn get_column(&self, cell: usize) -> usize {
        cell % self.m
    }

    pub fn get_cell(&self, r: usize, c: usize) -> usize {
        r * self.m + c
    }

    pub fn get_row_and_column(&self, cell: usize) -> (usize, usize) {
        let row = cell / self.m;
        let col = cell % self.m;
        (row, col)
    }

    pub fn get(&self, cell: usize) -> CellInfo {
        self.data[cell].clone()
    }

    pub fn set(&mut self, cell: usize, info: CellInfo) {
        self.data[cell] = info;
    }
}

pub fn parse_dimensions(rows_str: &str, cols_str: &str) -> Result<(usize, usize), &'static str> {
    let n: usize = match rows_str.parse() {
        Ok(n) if n > 0 && n <= N_GLOBAL_MAX => n,
        _ => return Err("Invalid number of rows"),
    };

    let m: usize = match cols_str.parse() {
        Ok(m) if m > 0 && m <= M_GLOBAL_MAX => m,
        _ => return Err("Invalid number of columns"),
    };

    Ok((n, m))
}

// These functions are needed by parser.rs
pub fn get_cell(row: usize, col: usize) -> usize {
    row * M_MAX() + col
}

pub fn get_row_and_column(cell: usize) -> (usize, usize) {
    let row = cell / M_MAX();
    let col = cell % M_MAX();
    (row, col)
}

pub fn is_valid_cell(row: usize, col: usize) -> bool {
    row < N_MAX() && col < M_MAX()
}

pub fn is_valid_range(cell1: usize, cell2: usize) -> bool {
    cell1 <= cell2
        && (cell1 % M_MAX()) <= (cell2 % M_MAX())
        && (cell1 / M_MAX()) <= (cell2 / M_MAX())
}
// // These functions are needed by parser.rs
// pub fn get_cell(row: usize, col: usize) -> usize {
//     row * crate::sheet::M_MAX + col
// }

// pub fn get_row_and_column(cell: usize) -> (usize, usize) {
//     let row = cell / crate::sheet::M_MAX;
//     let col = cell % crate::sheet::M_MAX;
//     (row, col)
// }

// pub fn is_valid_cell(row: usize, col: usize) -> bool {
//     row < crate::sheet::N_MAX && col < crate::sheet::M_MAX
// }

// pub fn is_valid_range(cell1: usize, cell2: usize) -> bool {
//     cell1 <= cell2
//         && (cell1 % crate::sheet::M_MAX) <= (cell2 % crate::sheet::M_MAX)
//         && (cell1 / crate::sheet::M_MAX) <= (cell2 / crate::sheet::M_MAX)
// }
