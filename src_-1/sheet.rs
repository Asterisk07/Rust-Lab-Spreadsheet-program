// sheet.rs
use std::cell::RefCell;
use std::cmp::min;
use std::io;
use std::rc::Rc;

use crate::convert::num_to_alpha;
use crate::info::CellInfo;
use crate::status::StatusCode;

pub const N_MAX: usize = 999;
pub const M_MAX: usize = 18278;

// use std::sync::OnceLock;

// pub static M: OnceLock<usize> = OnceLock::new();
// pub static N: OnceLock<usize> = OnceLock::new();

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

    pub fn display(&self) -> io::Result<()> {
        print!("{:3} ", ' ');
        for j in self.py..min(self.py + 10, self.m) {
            let col_heading = num_to_alpha((j + 1) as u32);
            print!("{:11} ", col_heading);
        }
        println!();

        // Print each row
        for i in self.px..min(self.px + 10, self.n) {
            // Print the row number (1-indexed) with a field width of 3
            print!("{:3} ", i + 1);
            for j in self.py..min(self.py + 10, self.m) {
                let cell_index = self.get_cell(i, j);
                let cell = &self.data[cell_index];

                // If the cell's invalid flag is set, print "ERR"
                if cell.info.invalid {
                    print!("{:11} ", "ERR");
                } else {
                    print!("{:11} ", cell.value);
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
        Ok(n) if n > 0 && n <= N_MAX => n,
        _ => return Err("Invalid number of rows"),
    };

    let m: usize = match cols_str.parse() {
        Ok(m) if m > 0 && m <= M_MAX => m,
        _ => return Err("Invalid number of columns"),
    };

    Ok((n, m))
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

// use crate::sheet::{M, N};

// Parse string inputs and validate against initialized M and N
// pub fn parse_dimensions(rows_str: &str, cols_str: &str) -> Result<(usize, usize), &'static str> {
//     let n: usize = rows_str.parse().map_err(|_| "Invalid number of rows")?;
//     let m: usize = cols_str.parse().map_err(|_| "Invalid number of columns")?;

//     if let (Some(&n_max), Some(&m_max)) = (N.get(), M.get()) {
//         if n == 0 || n > n_max {
//             return Err("Invalid number of rows");
//         }
//         if m == 0 || m > m_max {
//             return Err("Invalid number of columns");
//         }
//         Ok((n, m))
//     } else {
//         Err("Dimensions not initialized")
//     }
// }

// // Convert row, col to cell index
// pub fn get_cell(row: usize, col: usize) -> usize {
//     let m = *M.get().expect("M not initialized");
//     row * m + col
// }

// // Convert cell index to (row, col)
// pub fn get_row_and_column(cell: usize) -> (usize, usize) {
//     let m = *M.get().expect("M not initialized");
//     (cell / m, cell % m)
// }

// // Check if (row, col) is within bounds
// pub fn is_valid_cell(row: usize, col: usize) -> bool {
//     match (N.get(), M.get()) {
//         (Some(&n), Some(&m)) => row < n && col < m,
//         _ => false,
//     }
// }

// // Check if cell range is top-left to bottom-right
// pub fn is_valid_range(cell1: usize, cell2: usize) -> bool {
//     let m = *M.get().expect("M not initialized");
//     cell1 <= cell2 && (cell1 % m) <= (cell2 % m) && (cell1 / m) <= (cell2 / m)
// }
