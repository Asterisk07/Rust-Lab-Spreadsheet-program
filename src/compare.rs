//! A small CLI tool to compare two files line-by-line with a buffer size similar to C's `fgets`.
//!
//! It reads each file in chunks, compares lines, and reports the number of differences found.
//! It also includes tests using temporary files for validation.
use std::env;
use std::fs::File;
use std::io::{self, Read};
/// Maximum length of a buffer to read each line chunk (19 bytes + 1 for null in C).
const MAXLEN: usize = 19; // Read up to 19 bytes per chunk (like C's fgets with buffer size 20)
/// Compares two files line-by-line and returns the number of differences.
///
/// - Reports extra lines in either file.
/// - Prints mismatched lines with line numbers.
/// - Treats input as byte streams, splits lines on `\n`, and compares.
///
/// # Arguments
///
/// * `f1name` - Path to the first file.
/// * `f2name` - Path to the second file.
///
/// # Returns
///
/// * `Ok(differences)` - Number of lines that differ.
/// * `Err(e)` - I/O error occurred during file access or reading.
fn compare(f1name: &str, f2name: &str) -> io::Result<i32> {
    let mut diffs = 0;
    let mut line = 0;
    let mut print_header = true;

    let mut f1 = File::open(f1name)?;
    let mut f2 = File::open(f2name)?;

    let mut buf1 = vec![0; MAXLEN];
    let mut buf2 = vec![0; MAXLEN];

    loop {
        let bytes1 = f1.read(&mut buf1)?;
        let bytes2 = f2.read(&mut buf2)?;

        line += 1;

        // Both files exhausted
        if bytes1 == 0 && bytes2 == 0 {
            break;
        }

        // Process chunks
        let s1 = if bytes1 > 0 {
            let end = buf1[..bytes1]
                .iter()
                .position(|&c| c == b'\n')
                .unwrap_or(bytes1);
            String::from_utf8_lossy(&buf1[..end]).to_string()
        } else {
            String::new()
        };

        let s2 = if bytes2 > 0 {
            let end = buf2[..bytes2]
                .iter()
                .position(|&c| c == b'\n')
                .unwrap_or(bytes2);
            String::from_utf8_lossy(&buf2[..end]).to_string()
        } else {
            String::new()
        };

        // Handle file exhaustion or differences
        if bytes1 == 0 {
            if print_header {
                println!("Differences found:");
                print_header = false;
            }
            diffs += 1;
            println!("Line {line}: Extra in second file: {s2}");
            // Read remaining content from f2
            loop {
                let bytes = f2.read(&mut buf2)?;
                if bytes == 0 {
                    break;
                }
                line += 1;
                let end = buf2[..bytes]
                    .iter()
                    .position(|&c| c == b'\n')
                    .unwrap_or(bytes);
                println!(
                    "Line {line}: Extra in second file: {}",
                    String::from_utf8_lossy(&buf2[..end])
                );
                diffs += 1;
            }
            break;
        } else if bytes2 == 0 {
            if print_header {
                println!("Differences found:");
                print_header = false;
            }
            diffs += 1;
            println!("Line {line}: Extra in first file: {s1}");
            // Read remaining content from f1
            loop {
                let bytes = f1.read(&mut buf1)?;
                if bytes == 0 {
                    break;
                }
                line += 1;
                let end = buf1[..bytes]
                    .iter()
                    .position(|&c| c == b'\n')
                    .unwrap_or(bytes);
                println!(
                    "Line {line}: Extra in first file: {}",
                    String::from_utf8_lossy(&buf1[..end])
                );
                diffs += 1;
            }
            break;
        } else if s1 != s2 {
            if print_header {
                println!("Differences found:");
                print_header = false;
            }
            diffs += 1;
            println!("Line {line}: '{s1}' | '{s2}'");
        }
    }

    Ok(diffs)
}
/// Main function to execute from CLI.
///
/// Expects two file paths as command-line arguments. Compares them using `compare()`
/// and reports the result. Prints usage if incorrect arguments are given.
fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} file1 file2", args[0]);
        std::process::exit(1);
    }

    match compare(&args[1], &args[2]) {
        Ok(0) => println!("Files are identical"),
        Ok(diffs) => eprintln!("Total differences: {diffs}"),
        Err(e) => eprintln!("Error: {e}"),
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_tempfile(content: &[u8]) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content).unwrap();
        file
    }

    #[test]
    fn test_identical_files() {
        let file1 = create_tempfile(b"Hello World\nLine 2\n");
        let file2 = create_tempfile(b"Hello World\nLine 2\n");

        let result = compare(
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        );
        assert_eq!(result.unwrap(), 0);
    }
    #[test]
    fn test_extra_lines_in_first_file() {
        let file1 = create_tempfile(b"Line 1\nLine 2\nLine 3\n");
        let file2 = create_tempfile(b"Line 1\nLine 2\n");

        let result = compare(
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        );
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_long_lines_over_buffer() {
        let long_line = "a".repeat(25);
        let file1 = create_tempfile(format!("{long_line}\n").as_bytes());
        let file2 = create_tempfile(format!("{}b\n", &long_line[..24]).as_bytes());

        let result = compare(
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        );
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_empty_files() {
        let file1 = create_tempfile(b"");
        let file2 = create_tempfile(b"");

        let result = compare(
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        );
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_one_empty_file() {
        let file1 = create_tempfile(b"");
        let file2 = create_tempfile(b"Content\n");

        let result = compare(
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        );
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_mixed_newline_positions() {
        let file1 = create_tempfile(b"Line1\nLine2");
        let file2 = create_tempfile(b"Line1Line2\n");

        let result = compare(
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        );
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_file_not_found() {
        let result = compare("nonexistent1.txt", "nonexistent2.txt");
        assert!(result.is_err());
    }
}
