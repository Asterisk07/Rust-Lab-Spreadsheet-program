// ===============================
// formulas.rs
// ===============================
use crate::info::{Cell, Info};
use crate::sheet::Sheet;

pub type EvalFn = fn(usize, &Sheet);

pub fn assignment(idx: usize, sheet: &Sheet) {
    let mut cell = sheet.cells[idx].borrow_mut();
    let info = cell.info.clone(); // Clone to avoid borrow issues

    if info.arg_mask & 1 != 0 {
        let arg_cell = sheet.cells[info.arg[0] as usize].borrow();
        cell.value = arg_cell.value;
        cell.info.invalid = arg_cell.info.invalid;
    } else {
        cell.value = info.arg[0];
        cell.info.invalid = false;
    }
}

pub fn add(idx: usize, sheet: &Sheet) {
    let (v1, v2, invalid) = {
        // Scope to ensure borrowing is dropped before final write
        let cell = sheet.cells[idx].borrow();
        let info = &cell.info;

        let (values, invalid) = resolve_args_and_invalid(info, sheet);
        // values.extend(std::iter::once(invalid));
        (values[0], values[1], invalid)
    };

    // Now update the cell with calculated values
    let mut cell = sheet.cells[idx].borrow_mut();
    cell.value = v1 + v2;
    cell.info.invalid = invalid;
}

// More functions like sub, mul, div, sum, avg, etc.

fn resolve_args_and_invalid(info: &Info, sheet: &Sheet) -> ([i32; 2], bool) {
    let mut values = [0; 2];

    values[0] = if info.arg_mask & 1 != 0 {
        sheet.cells[info.arg[0] as usize].borrow().value
    } else {
        info.arg[0]
    };

    values[1] = if info.arg_mask & 2 != 0 {
        sheet.cells[info.arg[1] as usize].borrow().value
    } else {
        info.arg[1]
    };

    let invalid = (info.arg_mask & 1 != 0
        && sheet.cells[info.arg[0] as usize].borrow().info.invalid)
        || (info.arg_mask & 2 != 0 && sheet.cells[info.arg[1] as usize].borrow().info.invalid);

    (values, invalid)
}

fn resolve_args(info: &Info, sheet: &Sheet) -> (i32, i32) {
    let v1 = if info.arg_mask & 1 != 0 {
        sheet.cells[info.arg[0] as usize].borrow().value
    } else {
        info.arg[0]
    };

    let v2 = if info.arg_mask & 2 != 0 {
        sheet.cells[info.arg[1] as usize].borrow().value
    } else {
        info.arg[1]
    };

    (v1, v2)
}

fn info_invalid(info: &Info, sheet: &Sheet) -> bool {
    (info.arg_mask & 1 != 0 && sheet.cells[info.arg[0] as usize].borrow().info.invalid)
        || (info.arg_mask & 2 != 0 && sheet.cells[info.arg[1] as usize].borrow().info.invalid)
}

pub fn evaluate(cell_id: usize, sheet: &Sheet, fns: &[EvalFn]) {
    let function_id = sheet.cells[cell_id].borrow().info.function_id;
    fns[function_id](cell_id, sheet);
}
