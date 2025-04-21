// info.rs

#[derive(Debug, Clone, Copy, Default)]
pub struct Info {
    pub visit: u8,
    pub arg_mask: u8,
    pub invalid: bool,
    pub function_id: u8,
    pub arg: [i32; 2],
}

impl Info {
    pub fn is_cell_arg1(&self) -> bool {
        self.arg_mask & 0b1 != 0
    }

    pub fn is_cell_arg2(&self) -> bool {
        (self.arg_mask >> 1) & 0b1 != 0
    }

    pub fn is_cell_both(&self) -> bool {
        self.arg_mask == 0b11
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct CellInfo {
    pub info: Info,
    pub value: i32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ValueInfo {
    pub is_cell: bool,
    pub value: i32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct CommandInfo {
    pub lhs_cell: i32,
    pub info: Info,
}
