// parser.rs
use lazy_static::lazy_static;
use regex::Regex;
use std::str::FromStr;

use crate::convert;
use crate::info::{CommandInfo, Info, ValueInfo};
use crate::sheet::{get_cell, get_row_and_column, is_valid_cell, is_valid_range};
use crate::status::{StatusCode, set_status_code};

const INPUT_BUFFER_SIZE: usize = 64;
const MAX_MATCHES: usize = 4;
const RANGE_OFFSET: usize = 6;
const ARITHMETIC_OFFSET: usize = 2;

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

#[derive(Debug, PartialEq)]
pub enum ParseError {
    InvalidCommand,
    InvalidCell,
    InvalidRange,
    InvalidValue,
    ParseFailure,
}

pub struct ParserContext {
    pub px: usize,
    pub py: usize,
    pub output_enabled: bool,
}

impl ParserContext {
    pub fn new() -> Self {
        Self {
            px: 0,
            py: 0,
            output_enabled: true,
        }
    }
}

pub fn parse_sheet_dimensions(n_str: &str, m_str: &str) -> Result<(usize, usize), ParseError> {
    let n = n_str.parse().map_err(|_| ParseError::InvalidValue)?;
    let m = m_str.parse().map_err(|_| ParseError::InvalidValue)?;

    if n > crate::sheet::N_MAX() || m > crate::sheet::M_MAX() || n == 0 || m == 0 {
        Err(ParseError::InvalidValue)
    } else {
        Ok((n, m))
    }
}

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

fn handle_expression(caps: &regex::Captures, info: &mut Info) -> Result<(), ParseError> {
    let expr = caps.get(2).unwrap().as_str();
    expression_parser(expr, info)
}

fn handle_integer(caps: &regex::Captures, info: &mut Info) -> Result<(), ParseError> {
    let value =
        i32::from_str(caps.get(0).unwrap().as_str()).map_err(|_| ParseError::InvalidValue)?;
    info.arg_mask = 0; // Not a cell
    info.arg[0] = value;
    info.function_id = 0; // Assignment function
    Ok(())
}

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
    use crate::sheet::{M_MAX, N_MAX};

    fn setup_context() -> ParserContext {
        ParserContext {
            output_enabled: true,
            px: 0,
            py: 0,
        }
    }

    #[test]
    fn test_undo_command() {
        let mut ctx = setup_context();
        let result = handle_other_commands("undo", &mut ctx).unwrap();
        assert_eq!(result.lhs_cell, -2);
    }

    #[test]
    fn test_redo_command() {
        let mut ctx = setup_context();
        let result = handle_other_commands("redo", &mut ctx).unwrap();
        assert_eq!(result.lhs_cell, -3);
    }
    #[test]
    fn test_parse_sheet_dimensions_valid() {
        let (n, m) = parse_sheet_dimensions("10", "20").unwrap();
        assert_eq!(n, 10);
        assert_eq!(m, 20);
    }

    #[test]
    fn test_parse_sheet_dimensions_invalid() {
        assert!(parse_sheet_dimensions("0", "5").is_err());
        assert!(parse_sheet_dimensions("5", "0").is_err());
        assert!(parse_sheet_dimensions(&(N_MAX() + 1).to_string(), "5").is_err());
        assert!(parse_sheet_dimensions("5", &(M_MAX() + 1).to_string()).is_err());
    }

    #[test]
    fn test_assignment_cell() {
        let mut info = Info::default();
        expression_parser("A1", &mut info).unwrap();
        assert_eq!(info.function_id, 0);
        assert_eq!(info.arg_mask, 1);
    }

    #[test]
    fn test_assignment_number() {
        let mut info = Info::default();
        expression_parser("42", &mut info).unwrap();
        assert_eq!(info.function_id, 0);
        assert_eq!(info.arg_mask, 0);
        assert_eq!(info.arg[0], 42);
    }

    #[test]
    fn test_arithmetic_operations() {
        let mut info = Info::default();
        expression_parser("A1+B2", &mut info).unwrap();
        assert_eq!(info.function_id, 2);
        assert_eq!(info.arg_mask, 0b11);

        expression_parser("5-C3", &mut info).unwrap();
        assert_eq!(info.function_id, 3);
    }

    #[test]
    fn test_range_functions() {
        let mut info = Info::default();
        expression_parser("SUM(A1:B2)", &mut info).unwrap();
        assert_eq!(info.function_id, 6 + 2); // RANGE_OFFSET + SUM index
        assert!(is_valid_range(info.arg[0] as usize, info.arg[1] as usize));
    }

    #[test]
    fn test_nested_expression() {
        let mut cmd_info = CommandInfo::default();
        parse("A1=SUM(B2:C3)+5", &mut setup_context()).unwrap();
        // Verify nested parsing through command_info inspection
    }

    #[test]
    fn test_cell_parser_valid() {
        assert!(cell_parser("A1").is_ok());
        assert!(cell_parser("ZZ99").is_ok());
    }

    #[test]
    fn test_cell_parser_invalid() {
        assert_eq!(cell_parser("A0").unwrap_err(), ParseError::InvalidCell);
        assert_eq!(cell_parser("1A").unwrap_err(), ParseError::InvalidCell);
    }

    #[test]
    fn test_scroll_to_command() {
        let mut ctx = setup_context();
        parse("scroll_to D5", &mut ctx).unwrap();
        assert!(ctx.px > 0 || ctx.py > 0);
    }

    #[test]
    fn test_control_parser_movement() {
        let mut ctx = setup_context();
        control_parser("w", &mut ctx).unwrap();
        assert_eq!(ctx.px, 0); // Test boundary condition

        control_parser("s", &mut ctx).unwrap();
        assert!(ctx.px <= N_MAX() - 10);
    }

    #[test]
    fn test_output_commands() {
        let mut ctx = setup_context();
        parse("disable_output", &mut ctx).unwrap();
        assert!(!ctx.output_enabled);

        parse("enable_output", &mut ctx).unwrap();
        assert!(ctx.output_enabled);
    }

    #[test]
    fn test_invalid_commands() {
        let mut ctx = setup_context();
        assert_eq!(
            parse("invalid_cmd", &mut ctx).unwrap_err(),
            ParseError::InvalidCommand
        );
        assert_eq!(
            parse("A1=invalid", &mut ctx).unwrap_err(),
            ParseError::InvalidCommand
        );
    }

    #[test]
    fn test_sleep_command() {
        let mut info = Info::default();
        expression_parser("SLEEP(100)", &mut info).unwrap();
        assert_eq!(info.function_id, 1);
    }

    #[test]
    fn test_parse_empty_input() {
        assert_eq!(
            parse("", &mut setup_context()).unwrap_err(),
            ParseError::InvalidCommand
        );
    }

    #[test]
    fn test_control_parser_exit() {
        // This test would need to be handled specially as it exits the process
        // Can be omitted or run in a subprocess
    }
}
