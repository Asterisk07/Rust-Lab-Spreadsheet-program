// graph.rs
use std::cell::{RefCell, RefMut};
use std::rc::Rc;

// External dependencies
use crate::info::{Info, VisitStatus};
use crate::list::{erase_list, push_front, Node};
use crate::sheet::{Sheet, GET_CELL, GET_COLUMN, GET_ROW_AND_COLUMN};
use crate::status::{set_status_code, StatusCode};
use crate::formulas::FPTR;

// Constants from original C code
const NOT_VISITED: VisitStatus = VisitStatus::NotVisited;
const IN_STACK: VisitStatus = VisitStatus::InStack;
const VISITED: VisitStatus = VisitStatus::Visited;

#[derive(Debug)]
pub struct AdjList {
    pub head: Option<Rc<RefCell<Node>>>,
    pub ptr: Option<Rc<RefCell<Node>>>,
}

pub struct GraphContext {
    stack: Vec<usize>,
    order_ptr: usize,
    stack_ptr: usize,
    adj: Vec<AdjList>,
    sheet: Rc<RefCell<Sheet>>,
    size: usize,
}

impl GraphContext {
    pub fn new(sheet: Rc<RefCell<Sheet>>, size: usize) -> Self {
        Self {
            stack: vec![0; size],
            order_ptr: size,
            stack_ptr: 0,
            adj: vec![AdjList { head: None, ptr: None }; size],
            sheet,
            size,
        }
    }

    fn sheet_mut(&self) -> RefMut<Sheet> {
        self.sheet.borrow_mut()
    }

    fn in_dependency(&self, cell: usize, info: &Info) -> bool {
        if (info.function_id >= 6 && info.function_id <= 10) {
            // Range function check
            let (x1, y1) = GET_ROW_AND_COLUMN(info.arg[0]);
            let (x2, y2) = GET_ROW_AND_COLUMN(info.arg[1]);
            let (cell_x, cell_y) = GET_ROW_AND_COLUMN(cell);
            cell_x >= x1 && cell_x <= x2 && cell_y >= y1 && cell_y <= y2
        } else {
            // Single argument check
            let arg1_cell = if info.arg_mask & 0b1 != 0 { info.arg[0] } else { usize::MAX };
            let arg2_cell = if info.arg_mask & 0b10 != 0 { info.arg[1] } else { usize::MAX };
            cell == arg1_cell || cell == arg2_cell
        }
    }

    fn modify_graph(&mut self, cell: usize, info: &Info, func: fn(&mut Option<Rc<RefCell<Node>>, usize)) {
        if info.function_id >= 6 && info.function_id <= 10 {
            // Range-based dependency
            let (x1, y1) = GET_ROW_AND_COLUMN(info.arg[0]);
            let (x2, y2) = GET_ROW_AND_COLUMN(info.arg[1]);
            
            for i in x1..=x2 {
                for j in y1..=y2 {
                    let x = GET_CELL(i, j);
                    func(&mut self.adj[x].head, cell);
                    self.adj[x].ptr = self.adj[x].head.clone();
                }
            }
        } else {
            // Single cell dependencies
            if info.arg_mask & 0b1 != 0 {
                let arg = info.arg[0];
                func(&mut self.adj[arg].head, cell);
                self.adj[arg].ptr = self.adj[arg].head.clone();
            }
            if info.arg_mask & 0b10 != 0 {
                let arg = info.arg[1];
                func(&mut self.adj[arg].head, cell);
                self.adj[arg].ptr = self.adj[arg].head.clone();
            }
        }
    }

    fn delete_expression(&mut self, cell: usize) {
        let info = &self.sheet.borrow().cells[cell].info;
        self.modify_graph(cell, info, erase_list);
    }

    fn add_expression(&mut self, cell: usize, new_info: &Info) {
        self.modify_graph(cell, new_info, push_front);
    }

    fn iterative_dfs(&mut self, cell: usize, new_info: &Info) -> bool {
        let mut sheet = self.sheet_mut();
        sheet.cells[cell].visit_status = IN_STACK;
        self.stack[self.stack_ptr] = cell;
        self.stack_ptr += 1;

        while self.stack_ptr > 0 {
            let u = self.stack[self.stack_ptr - 1];
            
            if self.in_dependency(u, new_info) {
                return false;
            }

            let adj_ptr = &mut self.adj[u].ptr;
            if let Some(node) = adj_ptr {
                let v = node.borrow().data;
                *adj_ptr = node.borrow().next.clone();
                
                match sheet.cells[v].visit_status {
                    IN_STACK => return false,
                    NOT_VISITED => {
                        sheet.cells[v].visit_status = IN_STACK;
                        self.stack[self.stack_ptr] = v;
                        self.stack_ptr += 1;
                    },
                    _ => {}
                }
            } else {
                sheet.cells[u].visit_status = VISITED;
                self.order_ptr -= 1;
                self.stack[self.order_ptr] = u;
                self.stack_ptr -= 1;
            }
        }
        true
    }

    fn reset(&mut self) {
        let mut sheet = self.sheet_mut();
        for i in 0..self.stack_ptr {
            let cell = self.stack[i];
            sheet.cells[cell].visit_status = NOT_VISITED;
            self.adj[cell].ptr = self.adj[cell].head.clone();
        }
        for i in self.order_ptr..self.size {
            let cell = self.stack[i];
            sheet.cells[cell].visit_status = NOT_VISITED;
            self.adj[cell].ptr = self.adj[cell].head.clone();
        }
        self.stack_ptr = 0;
        self.order_ptr = self.size;
    }

    fn update_values(&mut self) {
        let mut sheet = self.sheet_mut();
        for i in self.order_ptr..self.size {
            let cell = self.stack[i];
            let func_id = sheet.cells[cell].info.function_id;
            FPTR[func_id](&mut sheet.cells[cell]);
        }
    }

    pub fn update_expression(&mut self, cell: usize, new_info: &Info) -> Result<(), ()> {
        if !self.iterative_dfs(cell, new_info) {
            set_status_code(StatusCode::CyclicDependency);
            self.reset();
            return Err(());
        }

        self.delete_expression(cell);
        self.add_expression(cell, new_info);

        let mut sheet = self.sheet_mut();
        sheet.cells[cell].info = new_info.clone();

        self.update_values();
        self.reset();
        Ok(())
    }
}