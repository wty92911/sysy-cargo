use koopa::ir::Value;
use std::{collections::HashMap, fmt::Write};

#[derive(Copy, Clone, Debug)]
pub enum ValueStore {
    Const(i32),
    Reg(Reg),
}

pub type Reg = u8;

pub struct RegNode {
    pub value: Option<Value>,
    pub name: &'static str,
}

pub struct ValueManager {
    regs: HashMap<Reg, RegNode>,
    values: HashMap<Value, ValueStore>,
}

impl ValueManager {
    pub fn new() -> ValueManager {
        let mut regs = HashMap::new();
        regs.insert(
            0,
            RegNode {
                value: None,
                name: "x0",
            },
        );

        let temp_regs = [
            ("t0", 1),
            ("t1", 2),
            ("t2", 3),
            ("t3", 4),
            ("t4", 5),
            ("t5", 6),
            ("t6", 7),
        ];

        for &(name, num) in &temp_regs {
            regs.insert(num, RegNode { value: None, name });
        }

        let arg_regs = [
            ("a7", 8),
            ("a1", 9),
            ("a2", 10),
            ("a3", 11),
            ("a4", 12),
            ("a5", 13),
            ("a6", 14),
            ("a0", 15), // avoid use first
        ];

        for &(name, num) in &arg_regs {
            regs.insert(num, RegNode { value: None, name });
        }

        ValueManager {
            regs,
            values: HashMap::new(),
        }
    }

    pub fn get_reg(&self, reg: Reg) -> Option<&RegNode> {
        self.regs.get(&reg)
    }

    pub fn get_reg_name(&self, reg: Reg) -> &'static str {
        self.regs.get(&reg).unwrap().name
    }
    pub fn get_reg_mut(&mut self, reg: Reg) -> Option<&mut RegNode> {
        self.regs.get_mut(&reg)
    }

    pub fn alloc_reg(&mut self) -> Reg {
        for (reg, reg_node) in self.regs.iter() {
            if reg_node.value.is_none() {
                return *reg;
            }
        }
        panic!("Out of registers!");
    }

    pub fn reset_reg(&mut self, reg: Reg) -> Option<String> {
        let reg_node = self.regs.get_mut(&reg).unwrap();
        let v = reg_node.value.take();
        let name = reg_node.name;
        match v {
            Some(v) => {
                for (r, rn) in self.regs.iter_mut() {
                    if rn.value.is_none() {
                        self.values.insert(v, ValueStore::Reg(*r));
                        rn.value = Some(v);
                        return Some(format!("mv {}, {}", rn.name, name));
                    }
                }
                unimplemented!()
            }
            None => None,
        }
        //
    }
    pub fn get_value(&self, value: Value) -> Option<&ValueStore> {
        self.values.get(&value)
    }

    pub fn set_value(&mut self, value: Value, store: ValueStore) {
        self.values.insert(value, store);
    }

    pub fn get_store_name(&self, store: ValueStore) -> String {
        match store {
            ValueStore::Const(i) => i.to_string(),
            ValueStore::Reg(r) => self.get_reg_name(r).to_string(),
        }
    }

    pub fn load_reg(&mut self, value: Value) -> Option<String> {
        let store = *self.get_value(value).unwrap();
        match store {
            ValueStore::Const(i) => {
                let reg = self.alloc_reg();
                self.set_value(value, ValueStore::Reg(reg));
                Some(format!("li {}, {}", self.get_reg_name(reg), i))
            }
            ValueStore::Reg(_) => None,
        }
    }
}
