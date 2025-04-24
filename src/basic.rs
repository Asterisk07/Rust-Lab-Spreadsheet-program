// basic.rs

// For macro parity with C's preprocessor behavior
#[macro_export]
macro_rules! min {
    ($x:expr, $y:expr) => {{
        let x_val = $x;
        let y_val = $y;
        if x_val < y_val { x_val } else { y_val }
    }};
}

#[macro_export]
macro_rules! max {
    ($x:expr, $y:expr) => {{
        let x_val = $x;
        let y_val = $y;
        if x_val > y_val { x_val } else { y_val }
    }};
}

// Swap functions using standard library
pub fn swap_char(a: &mut u8, b: &mut u8) {
    std::mem::swap(a, b);
}

pub fn swap_int(a: &mut i32, b: &mut i32) {
    std::mem::swap(a, b);
}
