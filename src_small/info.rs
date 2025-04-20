// ===============================
// info.rs
// ===============================
#[derive(Debug, Clone, Default)]
pub struct Info {
    pub arg_mask: u8,
    pub function_id: usize,
    pub arg: [i32; 2],
    pub visit: u8,
    pub invalid: bool,
}

#[derive(Debug, Clone, Default)]
pub struct Cell {
    pub value: i32,
    pub info: Info,
}
