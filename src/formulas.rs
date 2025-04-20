// ===============================
// formulas.rs
// ===============================
use crate::info::{Cell, Info};
use crate::sheet::Sheet;

pub type EvalFn = fn(&mut Cell, &Sheet);

pub fn assignment(cell: &mut Cell, sheet: &Sheet) {
    let info = &cell.info;
    if info.arg_mask & 1 != 0 {
        cell.value = sheet.cells[info.arg[0] as usize].value;
        cell.info.invalid = sheet.cells[info.arg[0] as usize].info.invalid;
    } else {
        cell.value = info.arg[0];
        cell.info.invalid = false;
    }
}

pub fn add(cell: &mut Cell, sheet: &Sheet) {
    let info = &cell.info;
    let (v1, v2) = resolve_args(info, sheet);
    cell.value = v1 + v2;
    cell.info.invalid = info_invalid(info, sheet);
}

// More functions like sub, mul, div, sum, avg, etc.

fn resolve_args(info: &Info, sheet: &Sheet) -> (i32, i32) {
    let v1 = if info.arg_mask & 1 != 0 {
        sheet.cells[info.arg[0] as usize].value
    } else {
        info.arg[0]
    };
    let v2 = if info.arg_mask & 2 != 0 {
        sheet.cells[info.arg[1] as usize].value
    } else {
        info.arg[1]
    };
    (v1, v2)
}

fn info_invalid(info: &Info, sheet: &Sheet) -> bool {
    (info.arg_mask & 1 != 0 && sheet.cells[info.arg[0] as usize].info.invalid)
        || (info.arg_mask & 2 != 0 && sheet.cells[info.arg[1] as usize].info.invalid)
}

pub fn evaluate(cell: &mut Cell, sheet: &Sheet, fns: &[EvalFn]) {
    fns[cell.info.function_id](cell, sheet);
}
