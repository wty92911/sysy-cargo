use std::collections::HashMap;

use koopa::ir::Value;

pub enum Decl {
    Const(i32),
    Var(Value)
}
pub struct ValueManager {
    vm_stack: Vec<HashMap<String, Decl>>
}

impl ValueManager {
    pub fn new() -> Self {
        ValueManager {
            vm_stack: Vec::new(),
        }
    }

    pub fn cur_exist(&self, name: &str) -> bool {
        match self.vm_stack.last() {
            Some(vm) => vm.contains_key(name),
            None => false,
        }
    }

    pub fn push(&mut self) {
        self.vm_stack.push(HashMap::new());
    }

    pub fn pop(&mut self) {
        self.vm_stack.pop();
    }

    fn insert(&mut self, name: &str, value: Decl) {
        let vm = self.vm_stack.last_mut().unwrap();
        assert_eq!(vm.contains_key(name), false);
        vm.insert(name.to_string(), value);
    }

    pub fn insert_const(&mut self, name: &str, value: i32) {
        self.insert(name, Decl::Const(value));
    }

    pub fn insert_var(&mut self, name: &str, value: Value) {
        self.insert(name, Decl::Var(value));
    }

    pub fn get(&self, name: &str) -> Option<&Decl> {
        self.vm_stack.iter().rev().find_map(|vm| vm.get(name))
    }
}