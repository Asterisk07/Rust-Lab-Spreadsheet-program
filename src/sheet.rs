// sheet.rs

pub const N_MAX: usize = 999;
pub const M_MAX: usize = 18278;

#[inline]
pub fn is_valid_cell(row: usize, col: usize, cols: usize) -> bool {
    row < N_MAX && col < cols && col < M_MAX
}

#[inline]
pub fn is_valid_range(cell1: usize, cell2: usize, cols: usize) -> bool {
    cell1 <= cell2 && (cell1 % cols) <= (cell2 % cols)
}

#[inline]
pub fn get_row(cell: usize, cols: usize) -> usize {
    cell / cols
}

#[inline]
pub fn get_column(cell: usize, cols: usize) -> usize {
    cell % cols
}

#[inline]
pub fn get_cell(row: usize, col: usize, cols: usize) -> usize {
    row * cols + col
}

#[inline]
pub fn get_row_and_column(cell: usize, cols: usize) -> (usize, usize) {
    (cell / cols, cell % cols)
}