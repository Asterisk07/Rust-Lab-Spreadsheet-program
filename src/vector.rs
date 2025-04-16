// vector.rs
#[derive(Debug)]
pub struct Vector {
    container: Vec<i32>,
}

impl Vector {
    pub fn new() -> Self {
        Vector {
            container: Vec::new(),
        }
    }

    pub fn resize(&mut self, new_capacity: usize) -> bool {
        if new_capacity < self.container.len() {
            return false;
        }
        self.container.reserve_exact(new_capacity - self.container.capacity());
        true
    }

    pub fn push_back(&mut self, value: i32) -> bool {
        self.container.push(value);
        true
    }

    pub fn back(&self) -> Option<&i32> {
        self.container.last()
    }

    pub fn pop_back(&mut self) {
        self.container.pop();
        if self.container.len() <= self.container.capacity() / 4 {
            self.container.shrink_to_fit();
        }
    }

    pub fn erase(&mut self, key: i32) -> bool {
        if let Some(pos) = self.container.iter().position(|&x| x == key) {
            self.container.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn print(&self) {
        print!("(");
        for (i, val) in self.container.iter().enumerate() {
            if i == self.container.len() - 1 {
                print!("{}", val);
            } else {
                print!("{}, ", val);
            }
        }
        println!(")");
    }
}

impl Drop for Vector {
    fn drop(&mut self) {
       
    }
}