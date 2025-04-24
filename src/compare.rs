use std::env;
use std::fs::File;
use std::io::{self, Read};

const MAXLEN: usize = 19; // Read up to 19 bytes per chunk (like C's fgets with buffer size 20)

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
