// sheet.rs
//! This module provides a spreadsheet-like structure for managing cell data.
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
/// Global maximum allowed row count.
pub const N_GLOBAL_MAX: usize = 1000;

/// Global maximum allowed column count.
pub const M_GLOBAL_MAX: usize = 18278;

// pub static mut M_MAX: usize = 0;
// pub static mut N_MAX: usize = 0;

static mut M_INTERNAL: usize = 0;
static mut N_INTERNAL: usize = 0;
static mut INIT_DONE: bool = false;

/// Initializes the dimensions of the sheet.
///
/// # Arguments
/// - `m`: Number of columns.
/// - `n`: Number of rows.
///
/// # Panics
/// Panics if initialization is attempted more than once.
///
/// # Examples
/// ```
pub unsafe fn init_dimensions(m: usize, n: usize) {
    if INIT_DONE {
        panic!("Already initialized");
    }
    M_INTERNAL = m;
    N_INTERNAL = n;
    INIT_DONE = true;
}
/// Returns the maximum column count.
///
/// # Panics
/// Panics if not initialized.
///
/// # Examples
/// ```
/// let max_columns = M_MAX();
/// ```
pub fn M_MAX() -> usize {
    unsafe {
        if !INIT_DONE {
            panic!("M not initialized!");
        }
        M_INTERNAL
    }
}
/// Returns the maximum row count.
///
/// # Panics
/// Panics if not initialized.
///
/// # Examples
/// ```
/// let max_rows = N_MAX();
/// ```
pub fn N_MAX() -> usize {
    unsafe {
        if !INIT_DONE {
            panic!("N not initialized!");
        }
        N_INTERNAL
    }
}
/// Represents a spreadsheet sheet that holds cell data.
pub struct Sheet {
    /// Vector holding all cell information.
    pub data: Vec<CellInfo>,
    /// Number of rows.
    pub n: usize,
    /// Number of columns.
    pub m: usize,
    /// Current row cursor position.
    pub px: usize,
    /// Current column cursor position.
    pub py: usize,
}

impl Sheet {
    /// Creates a new sheet with a given number of rows and columns.
    ///
    /// # Arguments
    /// - `n`: Number of rows.
    /// - `m`: Number of columns.
    ///
    /// # Examples
    /// ```
    /// let sheet = Sheet::new(10, 5);
    /// ```
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
    /// Sets the cursor position within the sheet.
    ///
    /// Returns `OutOfBounds` if the position is invalid.
    ///
    /// # Arguments
    /// - `x`: Row index.
    /// - `y`: Column index.
    ///
    /// # Examples
    /// ```
    /// let mut sheet = Sheet::new(10, 5);
    /// assert!(sheet.set_position(3, 2).is_ok());
    /// ```
    pub fn set_position(&mut self, x: usize, y: usize) -> Result<(), StatusCode> {
        if x >= self.n || y >= self.m {
            return Err(StatusCode::OutOfBounds);
        }

        self.px = x;
        self.py = y;
        Ok(())
    }
    /// Scrolls the cursor by a relative amount.
    ///
    /// # Arguments
    /// - `dx`: Rows to move.
    /// - `dy`: Columns to move.
    ///
    /// # Examples
    /// ```
    /// let mut sheet = Sheet::new(10, 5);
    /// assert!(sheet.scroll(1, 1).is_ok());
    /// ```
    pub fn scroll(&mut self, dx: isize, dy: isize) -> Result<(), StatusCode> {
        let new_x = self.px.saturating_add_signed(dx);
        let new_y = self.py.saturating_add_signed(dy);

        self.set_position(new_x, new_y)
    }
    /// Displays the sheet data in tabular format.
    ///
    /// # Arguments
    /// - `context`: The parsing context.
    ///
    /// # Examples
    /// ```
    /// let mut sheet = Sheet::new(10, 5);
    /// let mut context = ParserContext::default();
    /// sheet.display(&mut context).unwrap();
    /// ```
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
    /// Determines if a cell is valid within the sheet.
    // Helper functions for cell access and validation
    pub fn is_valid_cell(&self, r: usize, c: usize) -> bool {
        r < self.n && c < self.m
    }

    pub fn is_valid_range(&self, cell1: usize, cell2: usize) -> bool {
        cell1 <= cell2
            && (cell1 % self.m) <= (cell2 % self.m)
            && (cell1 / self.m) <= (cell2 / self.m)
    }
    /// Returns the row index for a given cell.
    pub fn get_row(&self, cell: usize) -> usize {
        cell / self.m
    }
    /// Returns the column index for a given cell.
    pub fn get_column(&self, cell: usize) -> usize {
        cell % self.m
    }
    /// Gets the cell index given a row and column.
    pub fn get_cell(&self, r: usize, c: usize) -> usize {
        r * self.m + c
    }

    /// Retrieves row and column values from a cell index.
    pub fn get_row_and_column(&self, cell: usize) -> (usize, usize) {
        let row = cell / self.m;
        let col = cell % self.m;
        (row, col)
    }
    /// Gets the cell information from the sheet.
    pub fn get(&self, cell: usize) -> CellInfo {
        self.data[cell].clone()
    }
    /// Sets the cell information for a specific cell.
    pub fn set(&mut self, cell: usize, info: CellInfo) {
        self.data[cell] = info;
    }
}
/// Parses input dimensions into valid row and column counts.
///
/// Returns an error message if the values are invalid.
///
/// # Arguments
/// - `rows_str`: String representing row count.
/// - `cols_str`: String representing column count.
///
/// # Examples
/// ```
/// assert!(parse_dimensions("10", "5").is_ok());
/// assert!(parse_dimensions("0", "5").is_err());
/// ```
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

/// returns cell index
pub fn get_cell(row: usize, col: usize) -> usize {
    row * M_MAX() + col
}
/// return row and col from integer cell index
pub fn get_row_and_column(cell: usize) -> (usize, usize) {
    let row = cell / M_MAX();
    let col = cell % M_MAX();
    (row, col)
}
/// checks the boundary of sheet
pub fn is_valid_cell(row: usize, col: usize) -> bool {
    row < N_MAX() && col < M_MAX()
}
/// checks validity of range provided
pub fn is_valid_range(cell1: usize, cell2: usize) -> bool {
    cell1 <= cell2
        && (cell1 % M_MAX()) <= (cell2 % M_MAX())
        && (cell1 / M_MAX()) <= (cell2 / M_MAX())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::min;
    use std::io;
    use std::panic;

    // Define a simple dummy context that we can convert to a ParserContext.
    #[derive(Debug)]
    struct DummyParserContext {
        pub px: usize,
        pub py: usize,
    }

    impl DummyParserContext {
        fn new() -> Self {
            Self { px: 0, py: 0 }
        }
    }

    // Instead of using ParserContext::default(), we convert our dummy context to a ParserContext manually.
    impl From<DummyParserContext> for ParserContext {
        fn from(dummy: DummyParserContext) -> Self {
            // Use ParserContext::new() and override the position.
            let mut ctx = ParserContext::new();
            ctx.px = dummy.px;
            ctx.py = dummy.py;
            ctx
        }
    }

    #[test]
    fn test_sheet_new() {
        let sheet = Sheet::new(5, 10);
        assert_eq!(sheet.data.len(), 50);
        assert_eq!(sheet.n, 5);
        assert_eq!(sheet.m, 10);
        assert_eq!(sheet.px, 0);
        assert_eq!(sheet.py, 0);
    }

    #[test]
    fn test_set_position_valid() {
        let mut sheet = Sheet::new(5, 10);
        let res = sheet.set_position(3, 5);
        assert!(res.is_ok());
        assert_eq!(sheet.px, 3);
        assert_eq!(sheet.py, 5);
    }

    #[test]
    fn test_set_position_invalid() {
        let mut sheet = Sheet::new(5, 10);
        let res = sheet.set_position(10, 5);
        assert!(res.is_err());
        assert_eq!(res.err().unwrap(), StatusCode::OutOfBounds);
    }

    #[test]
    fn test_scroll() {
        let mut sheet = Sheet::new(10, 10);
        // Scroll by a positive delta.
        let res = sheet.scroll(2, 3);
        assert!(res.is_ok());
        assert_eq!(sheet.px, 2);
        assert_eq!(sheet.py, 3);
        // Scroll by a negative delta (using saturating_add_signed).
        let res2 = sheet.scroll(-1, -1);
        assert!(res2.is_ok());
        // When scrolling down from (2,3) by (-1,-1), expect (1,2).
        assert_eq!(sheet.px, 1);
        assert_eq!(sheet.py, 2);
    }

    #[test]
    fn test_display() {
        let mut sheet = Sheet::new(15, 15);
        // Populate some cells with non-default values.
        for i in 0..15 {
            for j in 0..15 {
                let idx = sheet.get_cell(i, j);
                sheet.data[idx].value = (i * 15 + j) as i32;
            }
        }
        // Create a dummy parser context with offsets.
        let dummy_ctx = DummyParserContext { px: 2, py: 3 };
        let mut parser_ctx: ParserContext = dummy_ctx.into();
        // Calling display() should update sheet.px and sheet.py and return Ok.
        let res = sheet.display(&mut parser_ctx);
        assert!(res.is_ok());
        assert_eq!(sheet.px, 2);
        assert_eq!(sheet.py, 3);
    }

    #[test]
    fn test_is_valid_cell() {
        let sheet = Sheet::new(5, 10);
        assert!(sheet.is_valid_cell(4, 9));
        assert!(!sheet.is_valid_cell(5, 9));
        assert!(!sheet.is_valid_cell(4, 10));
    }

    #[test]
    fn test_is_valid_range() {
        let sheet = Sheet::new(5, 10);
        // A valid range: choose cell indices that satisfy conditions.
        assert!(sheet.is_valid_range(5, 15));
        // An invalid range: first cell greater than second.
        assert!(!sheet.is_valid_range(20, 15));
    }

    #[test]
    fn test_get_row_and_get_column() {
        let sheet = Sheet::new(5, 10);
        // For cell index 23 in a 10-column sheet,
        // row = 23 / 10 = 2 and column = 23 % 10 = 3.
        assert_eq!(sheet.get_row(23), 2);
        assert_eq!(sheet.get_column(23), 3);
        let (row, col) = sheet.get_row_and_column(23);
        assert_eq!(row, 2);
        assert_eq!(col, 3);
    }

    #[test]
    fn test_get_cell_and_set_get() {
        let mut sheet = Sheet::new(5, 10);
        let idx = sheet.get_cell(2, 3);
        let mut cell = sheet.get(idx);
        assert_eq!(cell.value, 0);
        cell.value = 777;
        sheet.set(idx, cell);
        let new_cell = sheet.get(idx);
        assert_eq!(new_cell.value, 777);
    }

    #[test]
    fn test_parse_dimensions() {
        let dims = parse_dimensions("10", "15");
        assert!(dims.is_ok());
        let (n, m) = dims.unwrap();
        assert_eq!(n, 10);
        assert_eq!(m, 15);

        let dims_err = parse_dimensions("0", "15");
        assert!(dims_err.is_err());
        assert_eq!(dims_err.err().unwrap(), "Invalid number of rows");

        let dims_err2 = parse_dimensions("10", "0");
        assert!(dims_err2.is_err());
        assert_eq!(dims_err2.err().unwrap(), "Invalid number of columns");
    }
}
