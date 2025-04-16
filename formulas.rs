use std::cmp;
use std::f64;
use std::thread;
use std::time::Duration;

mod info;
use info :: *;
mod sheet;
use sheet :: *;
mod basic;
use basic :: *;

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

pub const fn is_range_function(function_index: i32) -> bool {
    function_index >= 6 && function_index <= 10
}

pub const fn is_arithmetic_function(function_index: i32) -> bool {
    function_index >= 2 && function_index <= 5
}

pub const fn is_single_arg_function(function_index: i32) -> bool {
    function_index >= 0 && function_index <= 1
}

pub fn max(cell_info: &mut CellInfo) {
    let (x1, y1) = get_row_and_column(cell_info.info.arg[0]);
    let (x2, y2) = get_row_and_column(cell_info.info.arg[1]);

    cell_info.value = i32::MIN;
    cell_info.info.invalid = 0;

    for i in x1..=x2 {
        for j in y1..=y2 {
            let cell = get_cell(i, j);
            // Bitwise OR the invalid flag from the referenced cell.
            cell_info.info.invalid |= SHEET[cell].info.invalid;
            if cell_info.info.invalid != 0 {
                return;
            }
            cell_info.value = cmp::max(cell_info.value, SHEET[cell].value);
        }
    }
}

pub fn min(cell_info: &mut CellInfo) {
    let (x1, y1) = get_row_and_column(cell_info.info.arg[0]);
    let (x2, y2) = get_row_and_column(cell_info.info.arg[1]);

    cell_info.value = i32::MAX;
    cell_info.info.invalid = 0;

    for i in x1..=x2 {
        for j in y1..=y2 {
            let cell = get_cell(i, j);
            cell_info.info.invalid |= SHEET[cell].info.invalid;
            if cell_info.info.invalid != 0 {
                return;
            }
            cell_info.value = cmp::min(cell_info.value, SHEET[cell].value);
        }
    }
}

pub fn avg(cell_info: &mut CellInfo) {
    let (x1, y1) = get_row_and_column(cell_info.info.arg[0]);
    let (x2, y2) = get_row_and_column(cell_info.info.arg[1]);

    let mut avg_value: i64 = 0;
    cell_info.info.invalid = 0;

    for i in x1..=x2 {
        for j in y1..=y2 {
            let cell = get_cell(i, j);
            cell_info.info.invalid |= SHEET[cell].info.invalid;
            if cell_info.info.invalid != 0 {
                return;
            }
            avg_value += SHEET[cell].value as i64;
        }
    }

    let num_entries = ((x2 - x1 + 1) * (y2 - y1 + 1)) as i64;
    avg_value /= num_entries;
    cell_info.value = avg_value as i32;
}

pub fn sum(cell_info: &mut CellInfo) {
    let (x1, y1) = get_row_and_column(cell_info.info.arg[0]);
    let (x2, y2) = get_row_and_column(cell_info.info.arg[1]);

    cell_info.value = 0;
    cell_info.info.invalid = 0;

    for i in x1..=x2 {
        for j in y1..=y2 {
            let cell = get_cell(i, j);
            cell_info.info.invalid |= SHEET[cell].info.invalid;
            if cell_info.info.invalid != 0 {
                return;
            }
            cell_info.value += SHEET[cell].value;
        }
    }
}

pub fn stdev(cell_info: &mut CellInfo) {
    let (x1, y1) = get_row_and_column(cell_info.info.arg[0]);
    let (x2, y2) = get_row_and_column(cell_info.info.arg[1]);

    let mut sum_squares: i64 = 0;
    let mut sum: i64 = 0;
    cell_info.info.invalid = 0;

    for i in x1..=x2 {
        for j in y1..=y2 {
            let cell = get_cell(i, j); 
            cell_info.info.invalid |= SHEET[cell].info.invalid;
            if cell_info.info.invalid != 0 {
                return;
            }
            let val = SHEET[cell].value as i64;
            sum_squares += val * val;
            sum += val;
        }
    }

    let num_entries = ((x2 - x1 + 1) * (y2 - y1 + 1)) as i64;
    let mean = sum / num_entries;
    let variance = (sum_squares - 2 * mean * sum + mean * mean * num_entries) as f64 / num_entries as f64;
    cell_info.value = (variance.sqrt().round()) as i32;
}

pub fn assignment(cell_info: &mut CellInfo) {
    if crate::info::is_cell_arg1(cell_info.info.arg_mask) {
        let index = cell_info.info.arg[0] as usize;
        cell_info.value = SHEET[index].value;
        cell_info.info.invalid = SHEET[index].info.invalid;
    } else {
        cell_info.value = cell_info.info.arg[0];
        cell_info.info.invalid = 0;
    }
}

pub fn sleep_assignment(cell_info: &mut CellInfo) {
    assignment(cell_info);
    if cell_info.info.invalid == 0 {
        // Sleep for at least 0 seconds (if value is negative, we sleep zero seconds).
        let delay = cmp::max(0, cell_info.value) as u64;
        thread::sleep(Duration::from_secs(delay));
    }
}

pub fn get_args(info: &Info) -> (i32, i32, i32) {
        let value1 = if crate::info::is_cell_arg1(info.arg_mask) {
            let index = info.arg[0] as usize;
            SHEET[index].value
        } else {
            info.arg[0]
        };

        let value2 = if crate::info::is_cell_arg2(info.arg_mask) {
            let index = info.arg[1] as usize;
            SHEET[index].value
        } else {
            info.arg[1]
        };

        let invalid1 = if crate::info::is_cell_arg1(info.arg_mask) {
            SHEET[info.arg[0] as usize].info.invalid
        } else {
            0
        };

        let invalid2 = if crate::info::is_cell_arg2(info.arg_mask) {
            SHEET[info.arg[1] as usize].info.invalid
        } else {
            0
        };

        let invalid = if invalid1 != 0 || invalid2 != 0 { 1 } else { 0 };
        (value1, value2, invalid)
    }

pub fn add(cell_info: &mut CellInfo) {
    let (value1, value2, invalid) = get_args(&cell_info.info);
    cell_info.value = value1 + value2;
    cell_info.info.invalid = invalid;
}

pub fn sub(cell_info: &mut CellInfo) {
    let (value1, value2, invalid) = get_args(&cell_info.info);
    cell_info.value = value1 - value2;
    cell_info.info.invalid = invalid;
}

pub fn mul(cell_info: &mut CellInfo) {
    let (value1, value2, invalid) = get_args(&cell_info.info);
    cell_info.value = value1 * value2;
    cell_info.info.invalid = invalid;
}

pub fn divide(cell_info: &mut CellInfo) {
    let (value1, value2, invalid) = get_args(&cell_info.info);
    cell_info.info.invalid = if invalid != 0 || value2 == 0 { 1 } else { 0 };
    if cell_info.info.invalid == 0 {
        cell_info.value = value1 / value2;
    }
}

