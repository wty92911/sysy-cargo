use std::collections::HashMap;

use koopa::ir::Value;

pub enum Decl {
    Const(i32),
    Var(Value)
}
pub struct ValueManager {
    vm: HashMap<String, Decl>,
}

impl ValueManager {
    pub fn new() -> Self {
        ValueManager {
            vm: HashMap::new(),
        }
    }

    pub fn exist(&self, name: &str) -> bool {
        self.vm.contains_key(name)
    }

    pub fn insert_const(&mut self, name: &str, value: i32) {
        self.vm.insert(name.to_string(), Decl::Const(value));
    }

    pub fn get_const(&self, name: &str) -> Option<i32> {
        match self.vm.get(name) {
            Some(Decl::Const(value)) => Some(*value),
            _ => None,
        }
    }

    pub fn insert_var(&mut self, name: &str, value: Value) {
        self.vm.insert(name.to_string(), Decl::Var(value));
    }

    pub fn get(&self, name: &str) -> Option<&Decl> {
        self.vm.get(name)
    }
}