// parser.rs
use lazy_static::lazy_static;
use regex::Regex;
use std::str::FromStr;

// mod convert;
// mod sheet;
// mod status;
// mod info;

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

    if n > sheet::N_MAX || m > sheet::M_MAX || n == 0 || m == 0 {
        Err(ParseError::InvalidValue)
    } else {
        Ok((n, m))
    }
}

pub fn expression_parser(expr: &str, info: &mut Info) -> Result<(), ParseError> {
    for (match_type, re) in PATTERNS.iter().enumerate() {
        if let Some(caps) = re.captures(expr) {
            return match match_type {
                0 | 1 => handle_assignment(&caps, info, match_type),
                2 => handle_arithmetic(&caps, info),
                3 => handle_range(&caps, info),
                4 => handle_expression(&caps, info),
                5 => handle_scroll(&caps, info),
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
    let func_index = ["MIN", "MAX", "AVG", "SUM", "STDEV"]
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

    if !is_valid_cell(row, col) {
        Err(ParseError::InvalidCell)
    } else {
        Ok(get_cell(row, col))
    }
}

pub fn parser(input: &str, context: &mut ParserContext) -> Result<CommandInfo, ParseError> {
    let mut cmd_info = CommandInfo::default();

    if input.len() == 1 {
        cmd_info.lhs_cell = -1;
        control_parser(input, context)?;
        return Ok(cmd_info);
    }

    if let Some(caps) = PATTERNS[4].captures(input) {
        let lhs_str = caps.get(1).unwrap().as_str();
        cmd_info.lhs_cell = cell_parser(lhs_str)? as i32;

        let expr = caps.get(2).unwrap().as_str();
        expression_parser(expr, &mut cmd_info.info)?;

        Ok(cmd_info)
    } else {
        handle_other_commands(input, context)
    }
}

fn handle_other_commands(
    input: &str,
    context: &mut ParserContext,
) -> Result<CommandInfo, ParseError> {
    match input {
        "disable_output" => {
            context.output_enabled = false;
            Ok(CommandInfo::default())
        }
        "enable_output" => {
            context.output_enabled = true;
            Ok(CommandInfo::default())
        }
        _ => {
            if let Some(caps) = PATTERNS[5].captures(input) {
                let cell_str = caps.get(1).unwrap().as_str();
                let cell = cell_parser(cell_str)?;
                let (row, col) = get_row_and_column(cell);
                context.px = row;
                context.py = col;
                Ok(CommandInfo::default())
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
            let delta = match input {
                "w" => (-1, 0),
                "a" => (0, -1),
                "s" => (1, 0),
                "d" => (0, 1),
                _ => unreachable!(),
            };

            context.px = context.px.saturating_add_signed(delta.0);
            context.py = context.py.saturating_add_signed(delta.1);
            Ok(())
        }
        _ => Err(ParseError::InvalidCommand),
    }
}
