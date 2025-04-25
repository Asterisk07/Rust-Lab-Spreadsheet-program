// convert.rs
use crate::basic::swap_char;
/// Converts a 1-based column number to Excel-style column letters
/// (e.g. 1 -> "A", 26 -> "Z", 27 -> "AA")

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_num_to_alpha_basic() {
        assert_eq!(num_to_alpha(1), "A");
        assert_eq!(num_to_alpha(26), "Z");
        assert_eq!(num_to_alpha(27), "AA");
        assert_eq!(num_to_alpha(28), "AB");
        assert_eq!(num_to_alpha(52), "AZ");
        assert_eq!(num_to_alpha(53), "BA");
        assert_eq!(num_to_alpha(702), "ZZ");
        assert_eq!(num_to_alpha(703), "AAA");
    }

    #[test]
    fn test_num_to_alpha_edge_cases() {
        assert_eq!(num_to_alpha(0), ""); // Though 0 is invalid input
        assert_eq!(num_to_alpha(16384), "XFD");
        assert_eq!(num_to_alpha(1353), "AZA");
    }

    #[test]
    fn test_alpha_to_num_valid() {
        assert_eq!(alpha_to_num("A"), Some(1));
        assert_eq!(alpha_to_num("Z"), Some(26));
        assert_eq!(alpha_to_num("AA"), Some(27));
        assert_eq!(alpha_to_num("AB"), Some(28));
        assert_eq!(alpha_to_num("AZ"), Some(52));
        assert_eq!(alpha_to_num("BA"), Some(53));
        assert_eq!(alpha_to_num("ZZ"), Some(702));
        assert_eq!(alpha_to_num("AAA"), Some(703));
        assert_eq!(alpha_to_num("XFD"), Some(16384));
    }

    #[test]
    fn test_alpha_to_num_invalid() {
        assert_eq!(alpha_to_num(""), None);
        assert_eq!(alpha_to_num("a"), None);
        assert_eq!(alpha_to_num("aA"), None);
        assert_eq!(alpha_to_num("A1"), None);
        assert_eq!(alpha_to_num("A!"), None);
        assert_eq!(alpha_to_num(" "), None);
        assert_eq!(alpha_to_num("Ä"), None); // Non-ASCII uppercase
        assert_eq!(alpha_to_num("AB CD"), None);
    }

    #[test]
    fn test_round_trip_conversion() {
        // Test numbers 1-1000
        for n in 1..=1000 {
            let alpha = num_to_alpha(n);
            let converted = alpha_to_num(&alpha).unwrap();
            assert_eq!(n, converted as u32);
        }

        // Test specific large values
        let test_values = vec![16384, 12345, 9999, 703, 702];
        for &n in &test_values {
            let alpha = num_to_alpha(n);
            let converted = alpha_to_num(&alpha).unwrap();
            assert_eq!(n, converted as u32);
        }
    }
}
