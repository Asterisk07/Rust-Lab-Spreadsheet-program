// formulas.rs

use crate::info::{CellInfo, Info};
use std::cell::RefCell;
use std::cmp::{max as cmp_max, min as cmp_min};
use std::f64::consts::E;
use std::rc::Rc;
use std::thread;
use std::time::Duration;

pub static FPTR: [fn(&mut CellInfo, &Rc<RefCell<crate::sheet::Sheet>>); 11] = [
    assignment,
    sleep_assignment,
    add,
    sub,
    mul,
    divide,
    min,
    max,
    avg,
    sum,
    stdev,
];

// Helper functions to check function types
pub fn is_range_function(i: u8) -> bool {
    (6..=10).contains(&i)
}

pub fn is_arithmetic_function(i: u8) -> bool {
    (2..=5).contains(&i)
}

pub fn is_single_arg_function(i: u8) -> bool {
    (0..=1).contains(&i)
}

// Range-based functions
pub fn max(cell_info: &mut CellInfo, sheet_rc: &Rc<RefCell<crate::sheet::Sheet>>) {
    let sheet = sheet_rc.borrow();
    let (x1, y1) = sheet.get_row_and_column(cell_info.info.arg[0] as usize);
    let (x2, y2) = sheet.get_row_and_column(cell_info.info.arg[1] as usize);

    cell_info.value = i32::MIN;
    cell_info.info.invalid = false;

    for i in x1..=x2 {
        for j in y1..=y2 {
            let cell = sheet.get_cell(i, j);
            let cell_data = sheet.get(cell);
            cell_info.info.invalid |= cell_data.info.invalid != 0;
            if cell_info.info.invalid {
                return;
            }
            cell_info.value = cmp_max(cell_info.value, cell_data.value);
        }
    }
}

pub fn min(cell_info: &mut CellInfo, sheet_rc: &Rc<RefCell<crate::sheet::Sheet>>) {
    let sheet = sheet_rc.borrow();
    let (x1, y1) = sheet.get_row_and_column(cell_info.info.arg[0] as usize);
    let (x2, y2) = sheet.get_row_and_column(cell_info.info.arg[1] as usize);

    cell_info.value = i32::MAX;
    cell_info.info.invalid = false;

    for i in x1..=x2 {
        for j in y1..=y2 {
            let cell = sheet.get_cell(i, j);
            let cell_data = sheet.get(cell);
            cell_info.info.invalid |= cell_data.info.invalid != 0;
            if cell_info.info.invalid {
                return;
            }
            cell_info.value = cmp_min(cell_info.value, cell_data.value);
        }
    }
}

pub fn avg(cell_info: &mut CellInfo, sheet_rc: &Rc<RefCell<crate::sheet::Sheet>>) {
    let sheet = sheet_rc.borrow();
    let (x1, y1) = sheet.get_row_and_column(cell_info.info.arg[0] as usize);
    let (x2, y2) = sheet.get_row_and_column(cell_info.info.arg[1] as usize);

    let mut avg_value: i64 = 0;
    cell_info.info.invalid = false;

    for i in x1..=x2 {
        for j in y1..=y2 {
            let cell = sheet.get_cell(i, j);
            let cell_data = sheet.get(cell);
            cell_info.info.invalid |= cell_data.info.invalid != 0;
            if cell_info.info.invalid {
                return;
            }
            avg_value += cell_data.value as i64;
        }
    }

    let count = ((x2 - x1 + 1) * (y2 - y1 + 1)) as i64;
    cell_info.value = (avg_value / count) as i32;
}

pub fn sum(cell_info: &mut CellInfo, sheet_rc: &Rc<RefCell<crate::sheet::Sheet>>) {
    let sheet = sheet_rc.borrow();
    let (x1, y1) = sheet.get_row_and_column(cell_info.info.arg[0] as usize);
    let (x2, y2) = sheet.get_row_and_column(cell_info.info.arg[1] as usize);

    cell_info.value = 0;
    cell_info.info.invalid = false;

    for i in x1..=x2 {
        for j in y1..=y2 {
            let cell = sheet.get_cell(i, j);
            let cell_data = sheet.get(cell);
            cell_info.info.invalid |= cell_data.info.invalid != 0;
            if cell_info.info.invalid {
                return;
            }
            cell_info.value += cell_data.value;
        }
    }
}

pub fn stdev(cell_info: &mut CellInfo, sheet_rc: &Rc<RefCell<crate::sheet::Sheet>>) {
    let sheet = sheet_rc.borrow();
    let (x1, y1) = sheet.get_row_and_column(cell_info.info.arg[0] as usize);
    let (x2, y2) = sheet.get_row_and_column(cell_info.info.arg[1] as usize);

    let mut sum_squares: i64 = 0;
    let mut sum: i64 = 0;
    cell_info.info.invalid = false;

    for i in x1..=x2 {
        for j in y1..=y2 {
            let cell = sheet.get_cell(i, j);
            let cell_data = sheet.get(cell);
            cell_info.info.invalid |= cell_data.info.invalid != 0;
            if cell_info.info.invalid {
                return;
            }
            let val = cell_data.value as i64;
            sum_squares += val * val;
            sum += val;
        }
    }

    let count = ((x2 - x1 + 1) * (y2 - y1 + 1)) as i64;
    let mean = sum / count;
    let variance = (sum_squares - 2 * mean * sum + mean * mean * count) as f64 / count as f64;
    cell_info.value = variance.sqrt().round() as i32;
}

// Assignment operations
pub fn assignment(cell_info: &mut CellInfo, sheet_rc: &Rc<RefCell<crate::sheet::Sheet>>) {
    let is_cell_arg = cell_info.info.arg_mask & 0b1 != 0;

    if is_cell_arg {
        let sheet = sheet_rc.borrow();
        let arg_cell = sheet.get(cell_info.info.arg[0] as usize);
        cell_info.value = arg_cell.value;
        cell_info.info.invalid = arg_cell.info.invalid;
    } else {
        cell_info.value = cell_info.info.arg[0];
        cell_info.info.invalid = false;
    }
}

pub fn sleep_assignment(cell_info: &mut CellInfo, sheet_rc: &Rc<RefCell<crate::sheet::Sheet>>) {
    assignment(cell_info, sheet_rc);
    if !cell_info.info.invalid && cell_info.value > 0 {
        thread::sleep(Duration::from_secs(cell_info.value as u64));
    }
}

// Arithmetic operations
fn get_args(info: &Info, sheet: &crate::sheet::Sheet) -> (i32, i32, bool) {
    let val1 = if info.arg_mask & 0b1 != 0 {
        sheet.get(info.arg[0] as usize).value
    } else {
        info.arg[0]
    };

    let val2 = if info.arg_mask & 0b10 != 0 {
        sheet.get(info.arg[1] as usize).value
    } else {
        info.arg[1]
    };

    let invalid = (info.arg_mask & 0b1 != 0 && sheet.get(info.arg[0] as usize).info.invalid != 0)
        || (info.arg_mask & 0b10 != 0 && sheet.get(info.arg[1] as usize).info.invalid != 0);

    (val1, val2, invalid)
}

pub fn add(cell_info: &mut CellInfo, sheet_rc: &Rc<RefCell<crate::sheet::Sheet>>) {
    let sheet = sheet_rc.borrow();
    let (v1, v2, invalid) = get_args(&cell_info.info, &sheet);
    cell_info.value = v1 + v2;
    cell_info.info.invalid = invalid;
}

pub fn sub(cell_info: &mut CellInfo, sheet_rc: &Rc<RefCell<crate::sheet::Sheet>>) {
    let sheet = sheet_rc.borrow();
    let (v1, v2, invalid) = get_args(&cell_info.info, &sheet);
    cell_info.value = v1 - v2;
    cell_info.info.invalid = invalid;
}

pub fn mul(cell_info: &mut CellInfo, sheet_rc: &Rc<RefCell<crate::sheet::Sheet>>) {
    let sheet = sheet_rc.borrow();
    let (v1, v2, invalid) = get_args(&cell_info.info, &sheet);
    cell_info.value = v1 * v2;
    cell_info.info.invalid = invalid;
}

pub fn divide(cell_info: &mut CellInfo, sheet_rc: &Rc<RefCell<crate::sheet::Sheet>>) {
    let sheet = sheet_rc.borrow();
    let (v1, v2, invalid) = get_args(&cell_info.info, &sheet);
    cell_info.info.invalid = invalid || v2 == 0;
    if !cell_info.info.invalid {
        cell_info.value = v1 / v2;
    }
}

// Apply a function to a cell based on its function ID
pub fn apply_function(cell_info: &mut CellInfo, sheet_rc: &Rc<RefCell<crate::sheet::Sheet>>) {
    let func_idx = cell_info.info.function_id as usize;
    if func_idx < FPTR.len() {
        FPTR[func_idx](cell_info, sheet_rc);
    }
}
