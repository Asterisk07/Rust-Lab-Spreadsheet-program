// sheet.rs
use std::cmp::min;
mod convert ;
use convert :: *;
mod basic ;
use basic :: *;
mod parser ;
use parser :: *;
mod graph ;
use graph :: *;
mod list ;
use list :: *;
mod info ;
use info :: *;

pub const N_MAX: i32 = 999;
pub const M_MAX: i32 = 18278;

static mut N: i32 = 0;
static mut M: i32 = 0;
static mut PX: i32 = 0;
static mut PY: i32 = 0;
static mut SHEET: Option<Vec<CellInfo>> = None;
static mut OUTPUT: i32 = 1;

pub fn is_valid_cell(r: i32, c: i32) -> bool {
    unsafe { r >= 0 && r < N && c >= 0 && c < M }
}

pub fn is_valid_range(cell1: i32, cell2: i32) -> bool {
    unsafe { cell1 <= cell2 && (cell1 % M) <= (cell2 % M) }
}

pub fn get_row(cell: i32) -> i32 {
    unsafe { cell / M }
}

pub fn get_column(cell: i32) -> i32 {
    unsafe { cell % M }
}

pub fn get_cell(r: i32, c: i32) -> i32 {
    unsafe { r * M + c }
}

pub fn get_row_and_column(cell: i32) -> (i32, i32) {
    unsafe {
        let row = cell / M;
        let col = cell % M;
        (row, col)
    }
}

pub fn set_dimensions(n: i32, m: i32) {
    unsafe {
        N = n;
        M = m;
    }
}

pub fn set_offsets(px: i32, py: i32) {
    unsafe {
        PX = px;
        PY = py;
    }
}

pub fn init() {
    // Call the assumed external initialization functions.
    crate::basic::init_regex();
    crate::graph::init_graph();
    crate::basic::init_mem_pool();

    unsafe {
        let total = (N * M) as usize;
        SHEET = Some(vec![CellInfo::default(); total]);
    }
}

pub fn display_sheet() {
    unsafe {
        print!("{:3} ", ' ');
        for j in PY..min(PY + 10, M) {
            let col_heading = num_to_alpha(j + 1);
            print!("{:11} ", col_heading);
        }
        println!();

        // Print each row.
        for i in PX..min(PX + 10, N) {
            // Print the row number (1-indexed) with a field width of 3.
            print!("{:3} ", i + 1);
            for j in PY..min(PY + 10, M) {
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
}
