// status.rs
//! This module provides status code tracking and time-based feedback for command execution.
use lazy_static::lazy_static;
use std::io::{self, Write};
use std::sync::Mutex;
use std::time::{Duration, SystemTime};
/// Represents various status codes that indicate different system states.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StatusCode {
    /// Command executed successfully.
    Ok,
    /// Invalid command syntax or format.
    InvalidCmd,
    /// Overflow error occurred.
    Overflow,
    /// Invalid cell reference detected.
    InvalidCell,
    /// The specified range is invalid.
    InvalidRange,
    /// Cyclic dependency detected in processing.
    CyclicDep,
    /// No operation available to undo.
    NothingToUndo,
    /// No operation available to redo.
    NothingToRedo,
    /// The command exceeds the valid sheet boundaries.
    OutOfBounds,
    /// The provided value is not valid.
    InvalidValue,
    /// An internal error has occurred.
    InternalError,
}
/// Global mutex to hold the current system status code
lazy_static! {
    /// tells the commands
    pub static ref STATUS_CODE: Mutex<StatusCode> = Mutex::new(StatusCode::Ok);
     /// Tracks the last command execution time.
    static ref LAST_CMD_TIME: Mutex<SystemTime> = Mutex::new(SystemTime::now());
}
/// Status messages associated with each `StatusCode`.
const STATUS_MSG: [&str; 10] = [
    "ok",
    "invalid command",
    "overflow occurred",
    "invalid cell",
    "invalid range",
    "cyclic dependency found",
    "Nothing to undo",
    "Nothing to redo",
    "scrolling out of sheet",
    "invalid value",
];
/// Resets the start time to the current system time.
///
/// This is used to track the elapsed time since the last command execution.
pub fn start_time() {
    *LAST_CMD_TIME.lock().unwrap() = SystemTime::now();
}

/// Updates the global status code.
///
/// # Arguments
/// - `status`: The new `StatusCode` to set.
///
/// # Examples
/// ```
/// set_status_code(StatusCode::InvalidCmd);
/// assert_eq!(get_status_code(), StatusCode::InvalidCmd);
/// ```
pub fn set_status_code(status: StatusCode) {
    *STATUS_CODE.lock().unwrap() = status;
}
/// Retrieves the current system status code.
///
/// # Returns
/// The current `StatusCode`.
///
/// # Examples
/// ```
/// assert_eq!(get_status_code(), StatusCode::Ok);
/// ```
pub fn get_status_code() -> StatusCode {
    *STATUS_CODE.lock().unwrap()
}

/// Prints the current status message along with the elapsed time since the last command.
///
/// The format is: `[<elapsed_seconds>] (<status_message>) >`
///
/// # Examples
/// ```
/// start_time();
/// set_status_code(StatusCode::Overflow);
/// print_status();
/// ```
pub fn print_status() {
    let elapsed = LAST_CMD_TIME
        .lock()
        .unwrap()
        .elapsed()
        .unwrap_or(Duration::ZERO)
        .as_secs_f64();

    let status = *STATUS_CODE.lock().unwrap();
    let msg = STATUS_MSG[status as usize];

    print!("[{:.1}] ({}) >", elapsed, msg);
    io::stdout().flush().unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{self, Write};
    use std::thread::sleep;
    use std::time::{Duration, SystemTime};

    #[test]
    #[should_panic]
    fn test_print_status_internal_error() {
        // The STATUS_MSG array is defined with 10 elements (indices 0..9)
        // but StatusCode::InternalError, when cast as usize, equals 10.
        // This should cause an out-of-bound panic when attempting to index STATUS_MSG.
        set_status_code(StatusCode::InternalError);
        print_status();
    }
}
