// ===============================
// sheet.rs
// ===============================
use crate::info::{Cell, Info};
use std::cell::{Ref, RefCell, RefMut};

#[derive(Debug)]
pub struct Sheet {
    pub rows: usize,
    pub cols: usize,
    pub cells: Vec<RefCell<Cell>>,
}

impl Sheet {
    pub fn new(rows: usize, cols: usize) -> Self {
        let cells = (0..rows * cols)
            .map(|_| RefCell::new(Cell::default()))
            .collect();
        Sheet { rows, cols, cells }
    }

    pub fn get_index(&self, row: usize, col: usize) -> usize {
        row * self.cols + col
    }

    pub fn get_cell(&self, row: usize, col: usize) -> Ref<Cell> {
        self.cells[self.get_index(row, col)].borrow()
    }

    pub fn get_cell_mut(&self, row: usize, col: usize) -> RefMut<Cell> {
        self.cells[self.get_index(row, col)].borrow_mut()
    }
}
