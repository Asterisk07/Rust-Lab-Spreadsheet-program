// ===============================
// sheet.rs
// ===============================
use crate::formulas::evaluate;
use crate::info::{Cell, Info};

#[derive(Debug)]
pub struct Sheet {
    pub rows: usize,
    pub cols: usize,
    pub cells: Vec<Cell>,
}

impl Sheet {
    pub fn new(rows: usize, cols: usize) -> Self {
        let cells = vec![Cell::default(); rows * cols];
        Sheet { rows, cols, cells }
    }

    pub fn get_index(&self, row: usize, col: usize) -> usize {
        row * self.cols + col
    }

    pub fn get_cell(&self, row: usize, col: usize) -> &Cell {
        &self.cells[self.get_index(row, col)]
    }

    pub fn get_cell_mut(&mut self, row: usize, col: usize) -> &mut Cell {
        let idx = self.get_index(row, col);
        &mut self.cells[idx]
    }
}
