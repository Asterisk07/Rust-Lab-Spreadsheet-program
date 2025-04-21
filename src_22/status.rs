// status.rs
use lazy_static::lazy_static;
use std::sync::Mutex;
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StatusCode {
    Ok,
    InvalidCmd,
    Overflow,
    InvalidCell,
    InvalidRange,
    CyclicDep,
    OutOfBounds,
    InvalidValue,
}

lazy_static! {
    pub static ref STATUS_CODE: Mutex<StatusCode> = Mutex::new(StatusCode::Ok);
    static ref LAST_CMD_TIME: Mutex<SystemTime> = Mutex::new(SystemTime::now());
}

const STATUS_MSG: [&str; 8] = [
    "ok",
    "invalid command",
    "overflow occurred",
    "invalid cell",
    "invalid range",
    "cyclic dependency found",
    "scrolling out of sheet",
    "invalid value",
];

pub fn start_time() {
    *LAST_CMD_TIME.lock().unwrap() = SystemTime::now();
}

pub fn set_status_code(status: StatusCode) {
    *STATUS_CODE.lock().unwrap() = status;
}

pub fn get_status_code() -> StatusCode {
    *STATUS_CODE.lock().unwrap()
}

pub fn print_status() {
    let elapsed = LAST_CMD_TIME
        .lock()
        .unwrap()
        .elapsed()
        .unwrap_or(Duration::ZERO)
        .as_secs_f64();

    let status = *STATUS_CODE.lock().unwrap();
    let msg = STATUS_MSG[status as usize];

    println!("[{:.1}] ({}) >", elapsed, msg);
}
