// formulas.rs

use crate::info::{CellInfo, Info};
use crate::sheet;
use crate::sheet::{get_cell, get_row_and_column};
use std::cmp::{max as cmp_max, min as cmp_min};
use std::f64::consts::E;
use std::thread;
use std::time::Duration;

pub static FPTR: [fn(&mut CellInfo); 11] = [
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
pub fn max(cell_info: &mut CellInfo) {
    let (x1, y1) = get_row_and_column(cell_info.info.arg[0]);
    let (x2, y2) = get_row_and_column(cell_info.info.arg[1]);

    cell_info.value = i32::MIN;
    cell_info.info.invalid = false;

    for i in x1..=x2 {
        for j in y1..=y2 {
            let cell = get_cell(i, j);
            cell_info.info.invalid |= sheet::get(cell).info.invalid != 0;
            if cell_info.info.invalid {
                return;
            }
            cell_info.value = cmp_max(cell_info.value, sheet::get(cell).value);
        }
    }
}

pub fn min(cell_info: &mut CellInfo) {
    let (x1, y1) = get_row_and_column(cell_info.info.arg[0]);
    let (x2, y2) = get_row_and_column(cell_info.info.arg[1]);

    cell_info.value = i32::MAX;
    cell_info.info.invalid = false;

    for i in x1..=x2 {
        for j in y1..=y2 {
            let cell = get_cell(i, j);
            cell_info.info.invalid |= sheet::get(cell).info.invalid != 0;
            if cell_info.info.invalid {
                return;
            }
            cell_info.value = cmp_min(cell_info.value, sheet::get(cell).value);
        }
    }
}

pub fn avg(cell_info: &mut CellInfo) {
    let (x1, y1) = get_row_and_column(cell_info.info.arg[0]);
    let (x2, y2) = get_row_and_column(cell_info.info.arg[1]);

    let mut avg_value: i64 = 0;
    cell_info.info.invalid = false;

    for i in x1..=x2 {
        for j in y1..=y2 {
            let cell = get_cell(i, j);
            cell_info.info.invalid |= sheet::get(cell).info.invalid != 0;
            if cell_info.info.invalid {
                return;
            }
            avg_value += sheet::get(cell).value as i64;
        }
    }

    let count = ((x2 - x1 + 1) * (y2 - y1 + 1)) as i64;
    cell_info.value = (avg_value / count) as i32;
}

pub fn sum(cell_info: &mut CellInfo) {
    let (x1, y1) = get_row_and_column(cell_info.info.arg[0]);
    let (x2, y2) = get_row_and_column(cell_info.info.arg[1]);

    cell_info.value = 0;
    cell_info.info.invalid = false;

    for i in x1..=x2 {
        for j in y1..=y2 {
            let cell = get_cell(i, j);
            cell_info.info.invalid |= sheet::get(cell).info.invalid != 0;
            if cell_info.info.invalid {
                return;
            }
            cell_info.value += sheet::get(cell).value;
        }
    }
}

pub fn stdev(cell_info: &mut CellInfo) {
    let (x1, y1) = get_row_and_column(cell_info.info.arg[0]);
    let (x2, y2) = get_row_and_column(cell_info.info.arg[1]);

    let mut sum_squares: i64 = 0;
    let mut sum: i64 = 0;
    cell_info.info.invalid = false;

    for i in x1..=x2 {
        for j in y1..=y2 {
            let cell = get_cell(i, j);
            cell_info.info.invalid |= sheet::get(cell).info.invalid != 0;
            if cell_info.info.invalid {
                return;
            }
            let val = sheet::get(cell).value as i64;
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
pub fn assignment(cell_info: &mut CellInfo) {
    let is_cell_arg = cell_info.info.arg_mask & 0b1 != 0;
    cell_info.value = if is_cell_arg {
        sheet::get(cell_info.info.arg[0]).value
    } else {
        cell_info.info.arg[0]
    };
    cell_info.info.invalid = if is_cell_arg {
        sheet::get(cell_info.info.arg[0]).info.invalid
    } else {
        false
    };
}

pub fn sleep_assignment(cell_info: &mut CellInfo) {
    assignment(cell_info);
    if !cell_info.info.invalid && cell_info.value > 0 {
        thread::sleep(Duration::from_secs(cell_info.value as u64));
    }
}

// Arithmetic operations
fn get_args(info: &Info) -> (i32, i32, bool) {
    let val1 = if info.arg_mask & 0b1 != 0 {
        sheet::get(info.arg[0]).value
    } else {
        info.arg[0]
    };

    let val2 = if info.arg_mask & 0b10 != 0 {
        sheet::get(info.arg[1]).value
    } else {
        info.arg[1]
    };

    let invalid = (info.arg_mask & 0b1 != 0 && sheet::get(info.arg[0]).info.invalid != 0)
        || (info.arg_mask & 0b10 != 0 && sheet::get(info.arg[1]).info.invalid != 0);

    (val1, val2, invalid)
}

pub fn add(cell_info: &mut CellInfo) {
    let (v1, v2, invalid) = get_args(&cell_info.info);
    cell_info.value = v1 + v2;
    cell_info.info.invalid = invalid;
}

pub fn sub(cell_info: &mut CellInfo) {
    let (v1, v2, invalid) = get_args(&cell_info.info);
    cell_info.value = v1 - v2;
    cell_info.info.invalid = invalid;
}

pub fn mul(cell_info: &mut CellInfo) {
    let (v1, v2, invalid) = get_args(&cell_info.info);
    cell_info.value = v1 * v2;
    cell_info.info.invalid = invalid;
}

pub fn divide(cell_info: &mut CellInfo) {
    let (v1, v2, invalid) = get_args(&cell_info.info);
    cell_info.info.invalid = invalid || v2 == 0;
    if !cell_info.info.invalid {
        cell_info.value = v1 / v2;
    }
}

// Apply a function to a cell based on its function ID
pub fn apply_function(cell_info: &mut CellInfo) {
    let func_idx = cell_info.info.function_id as usize;
    if func_idx < FPTR.len() {
        FPTR[func_idx](cell_info);
    }
}
