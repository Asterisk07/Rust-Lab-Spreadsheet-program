// parser.rs
//! This module handles parsing commands and expressions for the spreadsheet system.
use lazy_static::lazy_static;
use regex::Regex;
use std::str::FromStr;

use crate::convert;
use crate::info::{CommandInfo, Info, ValueInfo};
use crate::sheet::{get_cell, get_row_and_column, is_valid_cell, is_valid_range};
use crate::status::{StatusCode, set_status_code};
/// Input buffer size constant.
const INPUT_BUFFER_SIZE: usize = 64;
/// Maximum regex match count.
const MAX_MATCHES: usize = 4;
/// Offset for range-based functions.
const RANGE_OFFSET: usize = 6;
/// Offset for arithmetic operations.
const ARITHMETIC_OFFSET: usize = 2;
/// Regular expressions used for parsing different command types.
lazy_static! {
    static ref PATTERNS: [Regex; 7] = [
        Regex::new(r"^([A-Z]{1,3}[1-9][0-9]{0,2}|[+-]?[0-9]+)$").unwrap(),         // ASSIGNMENT
        Regex::new(r"^SLEEP\(([A-Z]{1,3}[1-9][0-9]{0,2}|[+-]?[0-9]+)\)$").unwrap(), // SLEEP
        Regex::new(r"^([A-Z]{1,3}[1-9][0-9]{0,2}|[+-]?[0-9]+)([-+*/])([A-Z]{1,3}[1-9][0-9]{0,2}|[+-]?[0-9]+)$").unwrap(), // ARITHMETIC
        Regex::new(r"^(MAX|MIN|SUM|AVG|STDEV)\(([A-Z]{1,3}[1-9][0-9]{0,2}):([A-Z]{1,3}[1-9][0-9]{0,2})\)$").unwrap(), // RANGE
        Regex::new(r"^([A-Z]{1,3}[1-9][0-9]{0,2})=(.+)$").unwrap(),                 // EXPRESSION
        Regex::new(r"^scroll_to ([A-Z]{1,3}[1-9][0-9]{0,2})$").unwrap(),            // SCROLL_TO
        Regex::new(r"^[+-]?[0-9]+$").unwrap(),                                      // INTEGER
    ];
}
/// Represents different types of parsing errors.
#[derive(Debug, PartialEq)]
pub enum ParseError {
    /// Invalid command format.
    InvalidCommand,
    /// Invalid cell reference.
    InvalidCell,
    /// Invalid range reference.
    InvalidRange,
    /// Invalid value in an expression.
    InvalidValue,
    /// Failed to parse.
    ParseFailure,
}

/// Stores parser context information.
pub struct ParserContext {
    /// Current row position in the sheet.
    pub px: usize,
    /// Current column position in the sheet.
    pub py: usize,
    /// Controls whether output is enabled.
    pub output_enabled: bool,
}

impl ParserContext {
    /// Creates a new parser context.
    ///
    /// # Examples
    /// ```
    /// let context = ParserContext::new();
    /// ```
    pub fn new() -> Self {
        Self {
            px: 0,
            py: 0,
            output_enabled: true,
        }
    }
}
/// Parses sheet dimensions from string input.
///
/// # Arguments
/// - `n_str`: Rows count string.
/// - `m_str`: Columns count string.
///
/// # Returns
/// A tuple of `(rows, columns)` if valid.
///
/// # Errors
/// Returns `ParseError::InvalidValue` if values are out of bounds.
///
/// # Examples
/// ```
/// let dims = parse_sheet_dimensions("10", "5").unwrap();
/// ```
pub fn parse_sheet_dimensions(n_str: &str, m_str: &str) -> Result<(usize, usize), ParseError> {
    let n = n_str.parse().map_err(|_| ParseError::InvalidValue)?;
    let m = m_str.parse().map_err(|_| ParseError::InvalidValue)?;

    if n > crate::sheet::N_MAX() || m > crate::sheet::M_MAX() || n == 0 || m == 0 {
        Err(ParseError::InvalidValue)
    } else {
        Ok((n, m))
    }
}
/// Parses an expression and stores the result in `Info`.
///
/// # Arguments
/// - `expr`: Expression string.
/// - `info`: Storage for parsed information.
///
/// # Returns
/// Returns `Ok(())` if parsed successfully, otherwise `ParseError`.
pub fn expression_parser(expr: &str, info: &mut Info) -> Result<(), ParseError> {
    for (match_type, re) in PATTERNS.iter().enumerate() {
        // Skip the SCROLL_TO pattern (index 5) as it's handled by handle_other_commands

        if match_type == 5 {
            continue;
        }
        if let Some(caps) = re.captures(expr) {
            return match match_type {
                0 | 1 => handle_assignment(&caps, info, match_type),
                2 => handle_arithmetic(&caps, info),
                3 => handle_range(&caps, info),
                4 => handle_expression(&caps, info),
                6 => handle_integer(&caps, info),
                _ => Err(ParseError::InvalidCommand),
            };
        }
    }
    Err(ParseError::InvalidCommand)
}
/// Handles assignment expressions like `A1` or `42`, storing parsed result in `info`.
///
/// # Arguments
/// - `caps`: Regex captures from the matched assignment expression.
/// - `info`: Target `Info` structure to populate.
/// - `match_type`: Type of assignment pattern (0 for literal, 1 for SLEEP).
///
/// # Returns
/// `Ok(())` if parsed successfully, otherwise a `ParseError`.

fn handle_assignment(
    caps: &regex::Captures,
    info: &mut Info,
    match_type: usize,
) -> Result<(), ParseError> {
    let value_str = caps.get(1).unwrap().as_str();
    let mut value_info = ValueInfo::default();
    value_parser(value_str, &mut value_info)?;

    info.arg_mask = value_info.is_cell as u8;
    info.arg[0] = value_info.value as i32;
    info.function_id = match_type as u8;
    Ok(())
}
/// Parses arithmetic expressions like `A1+10` or `20/B3`, filling in the `Info` struct.
///
/// # Arguments
/// - `caps`: Captured groups from the arithmetic regex.
/// - `info`: Target `Info` structure to populate.
///
/// # Returns
/// `Ok(())` if parsing was successful, or `ParseError`.

fn handle_arithmetic(caps: &regex::Captures, info: &mut Info) -> Result<(), ParseError> {
    let op = caps.get(2).unwrap().as_str();
    let op_index = "+-*/".find(op).ok_or(ParseError::InvalidCommand)?;

    info.function_id = (ARITHMETIC_OFFSET + op_index) as u8;

    for j in 0..=1 {
        let value_str = caps.get(j * 2 + 1).unwrap().as_str();
        let mut value_info = ValueInfo::default();
        value_parser(value_str, &mut value_info)?;
        info.arg_mask |= (value_info.is_cell as u8) << j;
        info.arg[j] = value_info.value as i32;
    }
    Ok(())
}
/// Parses range-based function calls like `SUM(A1:B2)` into `Info`.
///
/// # Arguments
/// - `caps`: Captured groups from the range function regex.
/// - `info`: Target `Info` structure to populate.
///
/// # Returns
/// `Ok(())` if the range is valid, else `ParseError::InvalidRange`.

fn handle_range(caps: &regex::Captures, info: &mut Info) -> Result<(), ParseError> {
    let func_name = caps.get(1).unwrap().as_str();
    let func_index = ["MAX", "MIN", "SUM", "AVG", "STDEV"]
        .iter()
        .position(|&s| s == func_name)
        .ok_or(ParseError::InvalidCommand)?;

    info.function_id = (RANGE_OFFSET + func_index) as u8;
    info.arg_mask = 0b11;

    for j in 0..=1 {
        let cell_str = caps.get(j + 2).unwrap().as_str();
        let cell = cell_parser(cell_str)?;
        info.arg[j] = cell as i32;
    }

    if !is_valid_range(info.arg[0] as usize, info.arg[1] as usize) {
        Err(ParseError::InvalidRange)
    } else {
        Ok(())
    }
}
/// Handles recursive parsing of expressions of the form `A1=SUM(A1:A2)`.
///
/// # Arguments
/// - `caps`: Regex captures from expression assignment.
/// - `info`: Target `Info` to store parsed result.
///
/// # Returns
/// `Ok(())` if successfully parsed, otherwise `ParseError`.

fn handle_expression(caps: &regex::Captures, info: &mut Info) -> Result<(), ParseError> {
    let expr = caps.get(2).unwrap().as_str();
    expression_parser(expr, info)
}
/// Parses a numeric literal into a simple assignment function.
///
/// # Arguments
/// - `caps`: Regex match result containing the integer.
/// - `info`: Target `Info` structure to populate.
///
/// # Returns
/// `Ok(())` if valid integer, else `ParseError::InvalidValue`.

fn handle_integer(caps: &regex::Captures, info: &mut Info) -> Result<(), ParseError> {
    let value =
        i32::from_str(caps.get(0).unwrap().as_str()).map_err(|_| ParseError::InvalidValue)?;
    info.arg_mask = 0; // Not a cell
    info.arg[0] = value;
    info.function_id = 0; // Assignment function
    Ok(())
}
/// Parses a string as either a cell reference or an integer literal.
///
/// # Arguments
/// * `value_str` - The input string to parse.
/// * `value_info` - The structure to populate with the parsed result.
///
/// # Returns
/// `Ok(())` if parsing was successful, otherwise a `ParseError`.
///
/// # Example
/// ```
/// let mut vi = ValueInfo::default();
/// value_parser("A1", &mut vi).unwrap();
/// ```
pub fn value_parser(value_str: &str, value_info: &mut ValueInfo) -> Result<(), ParseError> {
    if value_str.chars().next().unwrap().is_ascii_uppercase() {
        value_info.is_cell = true;
        value_info.value = cell_parser(value_str)? as i32;
    } else {
        value_info.is_cell = false;
        value_info.value = i32::from_str(value_str).map_err(|_| ParseError::InvalidValue)?;
    }
    Ok(())
}
/// Parses a spreadsheet-style cell reference like "A1" into its linear index.
///
/// # Arguments
/// * `cell_str` - The cell reference string to parse.
///
/// # Returns
/// The linear index of the cell, or `ParseError::InvalidCell` if parsing fails.
///
/// # Example
/// ```
/// let index = cell_parser("B2").unwrap();
/// ```
pub fn cell_parser(cell_str: &str) -> Result<usize, ParseError> {
    let split_pos = cell_str
        .find(|c: char| c.is_ascii_digit())
        .ok_or(ParseError::InvalidCell)?;
    let (col_str, row_str) = cell_str.split_at(split_pos);

    let col = convert::alpha_to_num(col_str).ok_or(ParseError::InvalidCell)?;
    let row = usize::from_str(row_str).map_err(|_| ParseError::InvalidCell)? - 1;

    if !is_valid_cell(row, col - 1) {
        Err(ParseError::InvalidCell)
    } else {
        Ok(get_cell(row, col - 1))
    }
}
/// Parses an input command and converts it into `CommandInfo`.
///
/// # Arguments
/// - `input`: User command string.
/// - `context`: The parser context.
///
/// # Returns
/// Parsed command info if valid.
pub fn parse(input: &str, context: &mut ParserContext) -> Result<CommandInfo, ParseError> {
    if input.is_empty() {
        return Err(ParseError::InvalidCommand);
    }

    if input.len() == 1 {
        let mut cmd_info = CommandInfo::default();
        cmd_info.lhs_cell = -1;
        control_parser(input, context)?;
        return Ok(cmd_info);
    }
    // Check for special commands first

    let result = handle_other_commands(input, context);

    if result.is_ok() {
        return result;
    }

    if let Some(caps) = PATTERNS[4].captures(input) {
        let lhs_str = caps.get(1).unwrap().as_str();
        let cell = cell_parser(lhs_str)?;
        let mut cmd_info = CommandInfo::default();
        cmd_info.lhs_cell = cell as i32;

        let expr = caps.get(2).unwrap().as_str();
        expression_parser(expr, &mut cmd_info.info)?;

        Ok(cmd_info)
    } else {
        Err(ParseError::InvalidCommand)
    }
}
/// Handles special keywords like `undo`, `redo`, `scroll_to A1`, `enable_output`, etc.
///
/// # Arguments
/// - `input`: Command string.
/// - `context`: Current parser context for tracking position and output state.
///
/// # Returns
/// A populated `CommandInfo` or `ParseError::InvalidCommand` if unrecognized.

fn handle_other_commands(
    input: &str,
    context: &mut ParserContext,
) -> Result<CommandInfo, ParseError> {
    match input {
        "undo" => {
            let mut cmd_info = CommandInfo::default();
            cmd_info.lhs_cell = -2; // Special value for undo
            Ok(cmd_info)
        }
        "redo" => {
            let mut cmd_info = CommandInfo::default();
            cmd_info.lhs_cell = -3; // Special value for redo
            Ok(cmd_info)
        }
        "disable_output" => {
            context.output_enabled = false;
            let mut cmd_info = CommandInfo::default();
            cmd_info.lhs_cell = -1;
            Ok(cmd_info)
        }
        "enable_output" => {
            context.output_enabled = true;
            let mut cmd_info = CommandInfo::default();
            cmd_info.lhs_cell = -1;
            Ok(cmd_info)
        }
        _ => {
            if let Some(caps) = PATTERNS[5].captures(input) {
                let cell_str = caps.get(1).unwrap().as_str();
                let cell = cell_parser(cell_str)?;
                let (row, col) = get_row_and_column(cell);
                context.px = row;
                context.py = col;
                let mut cmd_info = CommandInfo::default();
                cmd_info.lhs_cell = -1;
                Ok(cmd_info)
            } else {
                Err(ParseError::InvalidCommand)
            }
        }
    }
}
/// Handles navigation commands like `w`, `a`, `s`, `d`, and exits on `q`.
///
/// # Arguments
/// - `input`: A single character command.
/// - `context`: Parser context, updated if scrolling is valid.
///
/// # Returns
/// `Ok(())` if command is valid and executed, or `ParseError::InvalidCommand`.

fn control_parser(input: &str, context: &mut ParserContext) -> Result<(), ParseError> {
    match input {
        "q" => std::process::exit(0),
        "w" | "a" | "s" | "d" => {
            // Get sheet dimensions
            let n = crate::sheet::N_MAX();
            let m = crate::sheet::M_MAX();
            let viewport_size = 10; // Assuming 10x10 viewport

            // Calculate max valid scroll positions
            let max_px = n.saturating_sub(viewport_size);
            let max_py = m.saturating_sub(viewport_size);

            // Calculate delta with boundary checks
            let (new_px, new_py) = match input {
                "w" => (
                    // Up
                    context.px.saturating_sub(10),
                    context.py,
                ),
                "s" => (
                    // Down
                    context.px.saturating_add(10).min(max_px),
                    context.py,
                ),
                "a" => (
                    // Left
                    context.px,
                    context.py.saturating_sub(10),
                ),
                "d" => (
                    // Right
                    context.px,
                    context.py.saturating_add(10).min(max_py),
                ),
                _ => unreachable!(),
            };

            // Only update if position changed
            if new_px != context.px || new_py != context.py {
                context.px = new_px;
                context.py = new_py;
            }

            Ok(())
        }
        _ => Err(ParseError::InvalidCommand),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::info::{CommandInfo, Info, ValueInfo};
    use crate::sheet::{get_cell, get_row_and_column};
    use std::panic;
    use std::str::FromStr;

    // Instead of checking the private static INIT_DONE, we attempt to initialize dimensions.
    // If they were already initialized, init_dimensions will panic, so we ignore it.
    fn ensure_sheet_dimensions() {
        unsafe {
            let _ = panic::catch_unwind(|| {
                crate::sheet::init_dimensions(100, 100);
            });
        }
    }

    // --- Tests for dimension parsing ---
    #[test]
    fn test_parse_sheet_dimensions_valid() {
        ensure_sheet_dimensions();
        let res = parse_sheet_dimensions("10", "20");
        assert!(res.is_ok(), "Expected valid dimensions");
        let (n, m) = res.unwrap();
        assert_eq!(n, 10);
        assert_eq!(m, 20);
    }

    #[test]
    fn test_parse_sheet_dimensions_invalid() {
        let res = parse_sheet_dimensions("0", "20");
        assert!(res.is_err());
        let res2 = parse_sheet_dimensions("10", "0");
        assert!(res2.is_err());
        let res3 = parse_sheet_dimensions("abc", "20");
        assert!(res3.is_err());
    }

    #[test]
    fn test_value_parser() {
        let mut vi = ValueInfo::default();
        // For a cell reference, value_parser should flag is_cell true.
        assert!(value_parser("A1", &mut vi).is_ok());
        assert!(vi.is_cell);
        // For a numeric string, is_cell remains false.
        let mut vi2 = ValueInfo::default();
        assert!(value_parser("123", &mut vi2).is_ok());
        assert!(!vi2.is_cell);
        assert_eq!(vi2.value, 123);
    }

    // --- Tests for expression_parser for various patterns ---
    #[test]
    fn test_expression_parser_assignment_number() {
        let mut info = Info::default();
        // "456" matches pattern 0 (assignment) as a number.
        let res = expression_parser("456", &mut info);
        assert!(res.is_ok());
        // For assignment, function_id is set to the match type (0 in this case).
        assert_eq!(info.function_id, 0);
        assert_eq!(info.arg[0], 456);
    }

    #[test]
    fn test_expression_parser_assignment_cell() {
        let mut info = Info::default();
        // "A1" interpreted as assignment.
        let res = expression_parser("A1", &mut info);
        assert!(res.is_ok());
        assert_eq!(info.function_id, 0);
        let expected_cell = cell_parser("A1").unwrap();
        assert_eq!(info.arg[0] as usize, expected_cell);
    }

    #[test]
    fn test_expression_parser_sleep() {
        let mut info = Info::default();
        // Pattern index 1: SLEEP
        let res = expression_parser("SLEEP(100)", &mut info);
        assert!(res.is_ok());
        // For sleep, function_id should be 1.
        assert_eq!(info.function_id, 1);
        assert_eq!(info.arg[0], 100);
    }

    #[test]
    fn test_expression_parser_arithmetic() {
        let mut info = Info::default();
        // Test arithmetic expression: pattern index 2.
        let res = expression_parser("10+20", &mut info);
        assert!(res.is_ok());
        // ARITHMETIC_OFFSET is 2; for '+' index is 0 so function_id = 2.
        assert_eq!(info.function_id, 2);
        assert_eq!(info.arg[0], 10);
        assert_eq!(info.arg[1], 20);
    }

    #[test]
    fn test_expression_parser_range_valid() {
        // Test a valid range expression.
        let mut info = Info::default();
        let res = expression_parser("SUM(A1:B2)", &mut info);
        assert!(res.is_ok());
        // With RANGE_OFFSET = 6 and "SUM" index=2 (from ["MAX", "MIN", "SUM", "AVG", "STDEV"]),
        // function_id should be 8.
        assert_eq!(info.function_id, 8);
        // It also sets arg_mask to 0b11.
        assert_eq!(info.arg_mask, 0b11);
        let a1 = cell_parser("A1").unwrap();
        let b2 = cell_parser("B2").unwrap();
        assert_eq!(info.arg[0] as usize, a1);
        assert_eq!(info.arg[1] as usize, b2);
    }

    #[test]
    fn test_expression_parser_range_invalid() {
        let mut info = Info::default();
        // Provide an invalid range, e.g., starting cell is greater than ending cell.
        let res = expression_parser("SUM(B2:A1)", &mut info);
        assert!(res.is_err());
        assert_eq!(res.err().unwrap(), ParseError::InvalidRange);
    }

    #[test]
    fn test_expression_parser_expression() {
        // Test pattern 4: expression of the form "A1=10+20"
        let res = parse("A1=10+20", &mut ParserContext::new());
        assert!(res.is_ok());
        let cmd_info = res.unwrap();
        let expected_cell = cell_parser("A1").unwrap();
        assert_eq!(cmd_info.lhs_cell, expected_cell as i32);
    }

    // --- Tests for the control/other commands via parse() ---
    #[test]
    fn test_parse_control_commands() {
        let mut context = ParserContext::new();
        // "undo" command should set lhs_cell to -2.
        let res_undo = parse("undo", &mut context);
        assert!(res_undo.is_ok());
        assert_eq!(res_undo.unwrap().lhs_cell, -2);

        // "redo" command should set lhs_cell to -3.
        let res_redo = parse("redo", &mut context);
        assert!(res_redo.is_ok());
        assert_eq!(res_redo.unwrap().lhs_cell, -3);

        // "disable_output" should turn off output.
        let res_disable = parse("disable_output", &mut context);
        assert!(res_disable.is_ok());
        assert_eq!(res_disable.unwrap().lhs_cell, -1);
        assert!(!context.output_enabled);

        // "enable_output" should re-enable output.
        context.output_enabled = false;
        let res_enable = parse("enable_output", &mut context);
        assert!(res_enable.is_ok());
        assert_eq!(res_enable.unwrap().lhs_cell, -1);
        assert!(context.output_enabled);

        // "scroll_to A1" should update context's px and py.
        let mut scroll_context = ParserContext::new();
        let res_scroll = parse("scroll_to A1", &mut scroll_context);
        assert!(res_scroll.is_ok());
        let cell = cell_parser("A1").unwrap();
        let (row, col) = get_row_and_column(cell);
        assert_eq!(scroll_context.px, row);
        assert_eq!(scroll_context.py, col);
    }

    #[test]
    fn test_parse_single_character_valid() {
        // Single-character commands: "w", "a", "s", "d" are valid control commands.
        for &cmd in &["w", "a", "s", "d"] {
            let res = parse(cmd, &mut ParserContext::new());
            assert!(res.is_ok());
            // For control commands, lhs_cell remains -1.
            assert_eq!(res.unwrap().lhs_cell, -1);
        }
    }

    #[test]
    fn test_parse_single_character_invalid() {
        // A single character not recognized (e.g., "x") should result in an InvalidCommand.
        let res = parse("x", &mut ParserContext::new());
        assert!(res.is_err());
        assert_eq!(res.err().unwrap(), ParseError::InvalidCommand);
    }

    #[test]
    fn test_parse_invalid_command() {
        // A command that doesn't match any known pattern returns InvalidCommand.
        let res = parse("foobar", &mut ParserContext::new());
        assert!(res.is_err());
        assert_eq!(res.err().unwrap(), ParseError::InvalidCommand);
    }
}
