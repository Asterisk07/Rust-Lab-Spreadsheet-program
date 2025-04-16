// For macro parity with C's preprocessor behavior
#[macro_export]
macro_rules! min {
    ($x:expr, $y:expr) => {{
        if $x < $y { $x } else { $y }
    }};
}

#[macro_export]
macro_rules! max {
    ($x:expr, $y:expr) => {{
        if $x > $y { $x } else { $y }
    }};
}

// Swap functions using standard library
pub fn swap_char(a: &mut char, b: &mut char) {
    std::mem::swap(a, b);
}

pub fn swap_int(a: &mut i32, b: &mut i32) {
    std::mem::swap(a, b);
}