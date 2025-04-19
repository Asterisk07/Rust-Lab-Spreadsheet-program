// convert.rs

/// Converts a 1-based column number to Excel-style column letters into provided buffer
/// (e.g. 1 -> "A", 26 -> "Z", 27 -> "AA") 
mod basic;
mod sheet;
use basic::*;
use sheet :: *;

pub fn num_to_alpha(buffer: &mut String, num: i32) -> Option<()> {
    let mut n = num;
    buffer.clear(); // Clear the `String` for safety

    while n > 0 {
        n -= 1;
        let ch = (b'A' + (n % 26) as u8) as char; // Convert to a UTF-8 character
        buffer.push(ch); // Append the character to the string
        n /= 26;
    }

    let reversed: String = buffer.chars().rev().collect();
    buffer.clear(); // Clear the buffer
    buffer.push_str(&reversed); // Update buffer with reversed characters

    Some(())
}

/// Returns None for invalid input
pub fn alpha_to_num(letters: &str) -> Option<i32> {
    let mut res = 0;

    for c in letters.chars() {
        if !c.is_ascii_uppercase() {
            return None; // Return None if the character is not an uppercase ASCII letter
        }
        res = res.checked_mul(26)?; // Checked multiplication to prevent overflow
        res = res.checked_add((c as u8 - b'A' + 1) as i32)?; // Convert char to ASCII index
    }

    Some(res)
}
