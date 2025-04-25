// info.rs
//! This module defines various structs for handling command execution and cell data.

/// Stores metadata for a command or operation.
#[derive(Debug, Clone, Copy, Default)]
pub struct Info {
    /// Number of times this operation has been visited (used for graph traversal).
    pub visit: u8,
    /// Bitmask representing whether arguments are cells.
    pub arg_mask: u8,
    /// Indicates if the command is invalid.
    pub invalid: bool,
    /// The function identifier.
    pub function_id: u8,
    /// Arguments related to the command.
    pub arg: [i32; 2],
}
impl Info {
    /// Checks if the first argument is a cell reference
    pub fn is_cell_arg1(&self) -> bool {
        self.arg_mask & 0b1 != 0
    }
    /// Checks if the second argument is a cell reference.
    pub fn is_cell_arg2(&self) -> bool {
        (self.arg_mask >> 1) & 0b1 != 0
    }
    /// Checks if both arguments are cell references.
    pub fn is_cell_both(&self) -> bool {
        self.arg_mask == 0b11
    }
}
/// Represents information stored in a spreadsheet cell.
#[derive(Debug, Clone, Copy, Default)]
pub struct CellInfo {
    pub info: Info,
    pub value: i32,
    pub literal_mode: bool,
}
/// Represents a value and whether it's a cell reference.
#[derive(Debug, Clone, Copy, Default)]
pub struct ValueInfo {
    pub is_cell: bool,
    pub value: i32,
}
/// Represents a parsed command in the spreadsheet system.
#[derive(Debug, Clone, Copy, Default)]
pub struct CommandInfo {
    pub lhs_cell: i32,
    pub info: Info,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_info_default() {
        let info = Info::default();
        // Default values should be zero/false.
        assert_eq!(info.visit, 0);
        assert_eq!(info.arg_mask, 0);
        assert!(!info.invalid);
        assert_eq!(info.function_id, 0);
        assert_eq!(info.arg, [0, 0]);
    }

    #[test]
    fn test_is_cell_arg1() {
        let mut info = Info::default();
        // With no bits set, first arg is false.
        info.arg_mask = 0;
        assert!(!info.is_cell_arg1());

        // With only the lowest bit set.
        info.arg_mask = 0b00000001;
        assert!(info.is_cell_arg1());

        // With both bits set.
        info.arg_mask = 0b00000011;
        assert!(info.is_cell_arg1());
    }

    #[test]
    fn test_is_cell_arg2() {
        let mut info = Info::default();
        info.arg_mask = 0;
        assert!(!info.is_cell_arg2());

        // Only the second bit set.
        info.arg_mask = 0b00000010;
        assert!(info.is_cell_arg2());

        // Both bits set.
        info.arg_mask = 0b00000011;
        assert!(info.is_cell_arg2());

        // Only first bit set should return false for arg2.
        info.arg_mask = 0b00000001;
        assert!(!info.is_cell_arg2());
    }

    #[test]
    fn test_is_cell_both() {
        let mut info = Info::default();
        info.arg_mask = 0;
        assert!(!info.is_cell_both());

        info.arg_mask = 0b00000001;
        assert!(!info.is_cell_both());

        info.arg_mask = 0b00000010;
        assert!(!info.is_cell_both());

        info.arg_mask = 0b00000011;
        assert!(info.is_cell_both());
    }

    #[test]
    fn test_cellinfo_debug_clone_copy() {
        let info = Info {
            visit: 5,
            arg_mask: 0b11,
            invalid: true,
            function_id: 10,
            arg: [42, -1],
        };
        let cell1 = CellInfo {
            info,
            value: 100,
            literal_mode: false,
        };

        // Test Debug formatting is non-empty.
        let debug_str = format!("{:?}", cell1);
        assert!(!debug_str.is_empty());

        // Test that Copy and Clone work as expected.
        let cell2 = cell1; // Copy occurs.
        assert_eq!(cell1.info.visit, cell2.info.visit);
        assert_eq!(cell1.value, cell2.value);
        assert_eq!(cell1.literal_mode, cell2.literal_mode);

        let cell3 = cell1.clone();
        assert_eq!(cell1.info.arg, cell3.info.arg);
    }

    #[test]
    fn test_valueinfo_default_and_manual() {
        let val_info = ValueInfo::default();
        assert!(!val_info.is_cell);
        assert_eq!(val_info.value, 0);

        let val_info_custom = ValueInfo {
            is_cell: true,
            value: 123,
        };
        assert!(val_info_custom.is_cell);
        assert_eq!(val_info_custom.value, 123);
    }

    #[test]
    fn test_commandinfo_default_and_manual() {
        let cmd_info = CommandInfo::default();
        assert_eq!(cmd_info.lhs_cell, 0);
        assert_eq!(cmd_info.info.visit, 0);

        let new_info = Info {
            visit: 3,
            arg_mask: 1,
            invalid: false,
            function_id: 7,
            arg: [10, 20],
        };
        let cmd_info_custom = CommandInfo {
            lhs_cell: 42,
            info: new_info,
        };
        assert_eq!(cmd_info_custom.lhs_cell, 42);
        assert_eq!(cmd_info_custom.info.function_id, 7);
        assert_eq!(cmd_info_custom.info.arg, [10, 20]);
    }
}
