use std::collections::HashMap;

enum Value {
    Const(i32),
}
pub struct ValueManager {
    vm: HashMap<String, Value>,
}

impl ValueManager {
    pub fn new() -> Self {
        ValueManager {
            vm: HashMap::new(),
        }
    }

    pub fn insert_const(&mut self, name: &str, value: i32) {
        self.vm.insert(name.to_string(), Value::Const(value));
    }

    pub fn get_const(&self, name: &str) -> Option<i32> {
        match self.vm.get(name) {
            Some(Value::Const(value)) => Some(*value),
            _ => None,
        }
    }
}