// basic.rs
//! This module provides utility macros for finding the minimum and maximum values,
//! as well as functions for swapping integers and characters efficiently.

/// Macro to compute the minimum of two values.
///
/// # Examples
/// ```
/// let min_value = min!(3, 5);
/// assert_eq!(min_value, 3);
/// ```
// For macro parity with C's preprocessor behavior
#[macro_export]
macro_rules! min {
    ($x:expr, $y:expr) => {{
        let x_val = $x;
        let y_val = $y;
        if x_val < y_val { x_val } else { y_val }
    }};
}
/// Macro to compute the maximum of two values.
///
/// # Examples
/// ```
/// let max_value = max!(3, 5);
/// assert_eq!(max_value, 5);
/// ```
#[macro_export]
macro_rules! max {
    ($x:expr, $y:expr) => {{
        let x_val = $x;
        let y_val = $y;
        if x_val > y_val { x_val } else { y_val }
    }};
}
/// Swaps two `u8` values in place using Rust's standard library.
///
/// # Arguments
/// - `a`: A mutable reference to a `u8` value.
/// - `b`: A mutable reference to another `u8` value.
///
/// # Examples
/// ```
/// let mut x = b'a';
/// let mut y = b'b';
/// swap_char(&mut x, &mut y);
/// assert_eq!(x, b'b');
/// assert_eq!(y, b'a');
/// ```
// Swap functions using standard library
pub fn swap_char(a: &mut u8, b: &mut u8) {
    std::mem::swap(a, b);
}

/// Swaps two `i32` values in place using Rust's standard library.
///
/// # Arguments
/// - `a`: A mutable reference to an `i32` value.
/// - `b`: A mutable reference to another `i32` value.
///
/// # Examples
/// ```
/// let mut x = 10;
/// let mut y = 20;
/// swap_int(&mut x, &mut y);
/// assert_eq!(x, 20);
/// assert_eq!(y, 10);
/// ```
pub fn swap_int(a: &mut i32, b: &mut i32) {
    std::mem::swap(a, b);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_min_macro() {
        // a < b
        assert_eq!(min!(3, 5), 3);
        // a > b
        assert_eq!(min!(5, 3), 3);
        // equal values
        assert_eq!(min!(4, 4), 4);
        // negative numbers
        assert_eq!(min!(-5, 3), -5);
        // same type different values
        assert_eq!(min!(10u32, 5u32), 5u32);
    }

    #[test]
    fn test_max_macro() {
        // a > b
        assert_eq!(max!(5, 3), 5);
        // a < b
        assert_eq!(max!(3, 5), 5);
        // equal values
        assert_eq!(max!(4, 4), 4);
        // negative numbers
        assert_eq!(max!(-5, -3), -3);
        // same type different values
        assert_eq!(max!(10u32, 5u32), 10u32);
    }

    #[test]
    fn test_swap_char() {
        // Different values
        let mut a = b'a';
        let mut b = b'b';
        swap_char(&mut a, &mut b);
        assert_eq!(a, b'b');
        assert_eq!(b, b'a');

        // Same values
        let mut c = b'x';
        let mut d = b'x';
        swap_char(&mut c, &mut d);
        assert_eq!(c, b'x');
        assert_eq!(d, b'x');

        // Edge cases
        let mut e = u8::MIN;
        let mut f = u8::MAX;
        swap_char(&mut e, &mut f);
        assert_eq!(e, u8::MAX);
        assert_eq!(f, u8::MIN);
    }

    #[test]
    fn test_swap_int() {
        // Positive numbers
        let mut a = 10;
        let mut b = 20;
        swap_int(&mut a, &mut b);
        assert_eq!(a, 20);
        assert_eq!(b, 10);

        // Negative numbers
        let mut c = -5;
        let mut d = -3;
        swap_int(&mut c, &mut d);
        assert_eq!(c, -3);
        assert_eq!(d, -5);

        // Zero and positive
        let mut e = 0;
        let mut f = 15;
        swap_int(&mut e, &mut f);
        assert_eq!(e, 15);
        assert_eq!(f, 0);

        // Edge cases
        let mut g = i32::MIN;
        let mut h = i32::MAX;
        swap_int(&mut g, &mut h);
        assert_eq!(g, i32::MAX);
        assert_eq!(h, i32::MIN);
    }

    #[test]
    fn test_macro_expression_safety() {
        // Test with complex expressions
        assert_eq!(min!(2 + 3 * 2, 10 - 1), 8);
        assert_eq!(max!(2 + 3 * 2, 10 - 1), 9);

        // Test side effect safety
        let mut x = 5;
        let result = min!(
            {
                x += 1;
                x
            },
            7
        );
        assert_eq!(result, 6);
        assert_eq!(x, 6);
    }
}
