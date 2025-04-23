// convert.rs

/// Converts a 1-based column number to Excel-style column letters
/// (e.g. 1 -> "A", 26 -> "Z", 27 -> "AA")
use crate::basic::swap_char;

pub fn num_to_alpha(num: u32) -> String {
    let mut n = num;
    let mut result = Vec::new();

    while n > 0 {
        n -= 1;
        let c = (b'A' + (n % 26) as u8) as char;
        result.push(c);
        n /= 26;
    }

    result.iter().rev().collect()
}

/// Converts Excel-style column letters to a 1-based number
/// Returns None for invalid input
pub fn alpha_to_num(letters: &str) -> Option<usize> {
    if letters.is_empty() {
        return None;
    }

    let mut res = 0;
    for c in letters.chars() {
        if !c.is_ascii_uppercase() {
            return None;
        }
        // result = result.checked_mul(26)?;
        // result = result.checked_add((c as usize) - ('A' as usize) + 1)?;

        // res *= 26;
        // res += (int) (buffer[i] - 'A' + 1);

        res *= 26;
        res += (c as usize) - ('A' as usize) + 1;
    }

    Some(res)
}
