use koopa::ir::Value;
use std::collections::{HashMap, HashSet};
use std::io::{Result, Write};
use rand::prelude::IteratorRandom;

#[derive(Copy, Clone, Debug)]
pub enum Mem {
    Stack(u32),
    Const(i32)
}

#[derive(Copy, Clone, Debug)]
pub enum ValueStore {
    Reg(Reg),
    Mem(Mem),
}
// todo last store
pub type Reg = u8;
pub const A0: Reg = 15;

#[derive(Copy, Clone, Debug)]
pub struct RegNode {
    pub value: Option<Value>,
    pub name: &'static str,
}

pub struct ValueManager {
    cur_offset: u32,
    max_offset: u32,
    regs: HashMap<Reg, RegNode>,
    avail_regs: HashSet<Reg>,
    value_reg: HashMap<Value, Reg>,
    value_mem: HashMap<Value, Mem>,
}

impl ValueManager {
    pub fn new() -> Self {
        let mut regs = HashMap::new();
        let mut avail_regs = HashSet::new();
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
            // ("t2", 3),
            // ("t3", 4),
            // ("t4", 5),
            // ("t5", 6),
            // ("t6", 7),
        ];

        for &(name, num) in &temp_regs {
            regs.insert(num, RegNode { value: None, name });
            avail_regs.insert(num);
        }

        let arg_regs = [
            // ("a7", 8),
            // ("a1", 9),
            // ("a2", 10),
            // ("a3", 11),
            // ("a4", 12),
            // ("a5", 13),
            // ("a6", 14),
            ("a0", 15), // avoid use first
        ];

        for &(name, num) in &arg_regs {
            regs.insert(num, RegNode { value: None, name });
            avail_regs.insert(num);
        }

        ValueManager {
            cur_offset: 0,
            max_offset: 0,
            regs,
            // avail_regs: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15], // except x0
            avail_regs, // except x0
            value_reg: HashMap::new(),
            value_mem: HashMap::new(),
        }
    }

    pub fn set_max_offset(&mut self, max_offset: u32) {
        self.max_offset = max_offset;
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

    pub fn lock_reg(&mut self, reg: Reg) {
        self.avail_regs.remove(&reg);
    }

    pub fn unlock_reg(&mut self, reg: Reg) {
        self.avail_regs.insert(reg);
    }

    pub fn alloc_reg<W: Write>(&mut self, specific: Option<Reg>, w: &mut W) -> Reg {
        let reg = specific.unwrap_or({
            let mut rng = rand::rng();
            let mut r = *self.avail_regs.iter().choose(&mut rng).unwrap();
            for reg in self.avail_regs.iter() {
                let reg_node = self.regs.get(reg).unwrap();
                if reg_node.value.is_none() {
                   r = *reg
                }
            }
            // random choose one
           r
        });

        if self.regs.get(&reg).unwrap().value.is_some() {
            self.store_to_mem(reg, w);
        }
        reg
    }

    pub fn get_value_mem(&self, value: &Value) -> Option<&Mem> {
        self.value_mem.get(value)
    }

    pub fn get_value_store(&self, value: &Value) -> Option<ValueStore> {
        if let Some(reg) = self.value_reg.get(value) {
            return Some(ValueStore::Reg(*reg));
        } else if let Some(mem) = self.value_mem.get(value) {
            return Some(ValueStore::Mem(*mem));
        }
        None
    }

    /// bind value and store reg or set mem
    pub fn set_value_store(&mut self, value: Value, store: ValueStore) {
        match store {
            ValueStore::Reg(r) => {
                let node = self.regs.get_mut(&r).unwrap();
                // clear old value
                if let Some(v) = node.value {
                    self.value_reg.remove(&v);
                }
                self.value_reg.insert(value, r);
                node.value = Some(value);
            }
            ValueStore::Mem(m) => {
                self.value_mem.insert(value, m);
            }
        }
    }

    pub fn load_to_reg<W: Write>(&mut self, value: Value, specific: Option<Reg>, w: &mut W) -> Result<Reg> {
        let store = self.get_value_store(&value).unwrap();
        match store {
            ValueStore::Mem(m) => {
                let reg = self.alloc_reg(specific, w);
                self.set_value_store(value, ValueStore::Reg(reg));
                let name =  self.regs.get(&reg).unwrap().name;
                match m {
                    Mem::Const(i) => {
                        writeln!(w, "  li {}, {}", name, i)?;
                    }
                    Mem::Stack(s) => {
                        writeln!(w, "  lw {}, {}(sp)", name, s)?;
                    }
                }
                Ok(reg)
            }
            ValueStore::Reg(r) => {
                match specific {
                    Some(_) => {
                        let reg = self.alloc_reg(specific, w);
                        self.set_value_store(value, ValueStore::Reg(reg));
                        let name =  self.regs.get(&reg).unwrap().name;
                        if r != reg {
                            writeln!(w, "  mv {}, {}", name, self.get_reg_name(r))?;
                        }
                        Ok(reg)
                    }
                    None => {
                        Ok(r)
                    }
                }
            },
        }
    }

    // alloc on stack
    pub fn alloc(&mut self, value: &Value, size: u32) -> Result<()> {
        self.set_value_store(*value, ValueStore::Mem(Mem::Stack(self.cur_offset)));
        self.cur_offset += size;
        Ok(())
    }

    /// store value of reg to mem, and change reg
    fn store_to_mem<W: Write>(&mut self, reg: Reg, w: &mut W) {
        let node = self.regs.get_mut(&reg).unwrap();
        if let Some(value) = node.value.take() {
            self.value_reg.remove(&value);
            let mem = self.value_mem.get(&value).unwrap();
            match mem {
                Mem::Const(_) => {} // do nothing
                Mem::Stack(s) => {
                    writeln!(w, "  sw {}, {}(sp)", node.name, s).unwrap();
                }
            }
        }
    }

    pub fn copy_to_mem<W: Write>(&mut self, reg: Reg, w: &mut W) {
        let node = self.regs.get(&reg).unwrap();
        if let Some(value) = node.value {
            let mem = self.value_mem.get(&value).unwrap();
            match mem {
                Mem::Const(_) => {} // do nothing
                Mem::Stack(s) => {
                    writeln!(w, "  sw {}, {}(sp)", self.get_reg_name(reg), s).unwrap();
                }
            }
        }

    }

}
