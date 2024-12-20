use crate::parser::asm::gen::*;
use koopa::ir::entities::FunctionData;
use koopa::ir::layout::BasicBlockNode;
use koopa::ir::values::*;
use koopa::ir::{BasicBlock, Program, Value, ValueKind};
use std::io::{Result, Write};

/// Visitor for generating the in-memory form Koopa IR program into the riscv
#[derive(Default)]
pub struct Visitor;

impl Visitor {
    pub fn visit<W: Write>(
        &mut self,
        w: &mut W,
        program: &koopa::ir::Program,
    ) -> std::io::Result<()> {
        let mut visitor = VisitorImpl {
            w,
            program,
            func: None,
            vm: ValueManager::new(),
        };
        visitor.visit()
    }
}

/// The implementation of riscv Koopa IR generator.
struct VisitorImpl<'a, W: Write> {
    w: &'a mut W,
    program: &'a Program,
    func: Option<&'a FunctionData>,
    vm: ValueManager,
}

impl<W: Write> VisitorImpl<'_, W> {
    /// Visits the program
    fn visit(&mut self) -> Result<()> {
        writeln!(self.w, "  .text")?;
        writeln!(self.w, "  .global main")?;

        for func in self.program.func_layout().iter() {
            let func = self.program.func(*func);
            self.func = Some(func);
            self.visit_func(func)?;
        }
        Ok(())
    }

    /// Generates the given function
    fn visit_func(&mut self, func: &FunctionData) -> Result<()> {
        writeln!(self.w, "{}:", &func.name()[1..])?;

        for (i, (bb, node)) in func.layout().bbs().iter().enumerate() {
            self.visit_bb(*bb, node)?;
        }
        Ok(())
    }

    /// Generates the given basic block.
    fn visit_bb(&mut self, bb: BasicBlock, node: &BasicBlockNode) -> Result<()> {
        for inst in node.insts().keys() {
            self.visit_local_inst(inst)?;
        }
        Ok(())
    }

    /// Generates the given local instruction.
    fn visit_local_inst(&mut self, inst: &Value) -> Result<()> {
        let value_data = self.func.unwrap().dfg().value(*inst);
        match value_data.kind() {
            ValueKind::Binary(b) => {
                self.visit_binary(inst, b)?;
            }
            ValueKind::Return(v) => self.visit_return(v)?,
            _ => unimplemented!(),
        };
        Ok(())
    }

    /// Generates function return.
    fn visit_return(&mut self, ret: &Return) -> Result<()> {
        if let Some(val) = ret.value() {
            self.visit_value(val)?;
            let val = self
                .vm
                .get_value(val)
                .expect(&format!("value {:#?} not found", val));

            match *val {
                ValueStore::Const(v) => {
                    if let Some(inst) = self.vm.reset_reg(15) {
                        writeln!(self.w, "  {}", inst)?;
                    }
                    writeln!(self.w, "  li {}, {}", self.vm.get_reg_name(15), v)?;
                }
                ValueStore::Reg(r) => {
                    if r != 15 {
                        if let Some(inst) = self.vm.reset_reg(15) {
                            writeln!(self.w, "  {}", inst)?;
                        }
                        write!(
                            self.w,
                            "  mv {}, {}",
                            self.vm.get_reg_name(15),
                            self.vm.get_reg_name(r)
                        )?;
                    }
                }
            }
        }
        writeln!(self.w, "  ret")?;
        Ok(())
    }

    /// Generates the given binary operation._
    fn visit_binary(&mut self, value: &Value, b: &Binary) -> Result<()> {
        self.visit_value(b.lhs())?;
        self.visit_value(b.rhs())?;

        let (lvs, rvs) = (
            *self.vm.get_value(b.lhs()).unwrap(),
            *self.vm.get_value(b.rhs()).unwrap(),
        );
        // let (lvs, rvs) = (self.vm.get_store_name(lvs), self.vm.get_store_name(rvs));
        // deal const val
        if let (ValueStore::Const(lv), ValueStore::Const(rv)) = (lvs, rvs) {
            dbg!("const: ", lv, rv);
            let bv = ValueStore::Const(match b.op() {
                BinaryOp::Eq => (lv == rv) as i32,
                BinaryOp::NotEq => (lv != rv) as i32,
                BinaryOp::Lt => (lv < rv) as i32,
                BinaryOp::Gt => (lv > rv) as i32,
                BinaryOp::Le => (lv <= rv) as i32,
                BinaryOp::Ge => (lv >= rv) as i32,
                BinaryOp::And => lv & rv,
                BinaryOp::Or => lv | rv,
                BinaryOp::Add => lv + rv,
                BinaryOp::Sub => lv - rv,
                BinaryOp::Mul => lv * rv,
                BinaryOp::Div => lv / rv,
                BinaryOp::Mod => lv % rv,
                _ => unimplemented!("not implemented"),
            });
            self.vm.set_value(*value, bv);
            return Ok(());
        }

        // deal reg, for now load all const to reg
        let rd = self.vm.alloc_reg();
        let rd_name = self.vm.get_reg_name(rd);

        if let Some(inst) = self.vm.load_reg(b.lhs()) {
            writeln!(self.w, "  {}", inst)?;
        }
        if let Some(inst) = self.vm.load_reg(b.rhs()) {
            writeln!(self.w, "  {}", inst)?;
        }
        let (lvs, rvs) = (
            *self.vm.get_value(b.lhs()).unwrap(),
            *self.vm.get_value(b.rhs()).unwrap(),
        );
        let (lvs, rvs) = (self.vm.get_store_name(lvs), self.vm.get_store_name(rvs));

        match b.op() {
            BinaryOp::Eq => {
                writeln!(self.w, "  sub {}, {}, {}", rd_name, lvs, rvs)?;
                writeln!(self.w, "  seqz {}, {}", rd_name, rd_name)?;
            }
            BinaryOp::NotEq => {
                writeln!(self.w, "  sub {}, {}, {}", rd_name, lvs, rvs)?;
                writeln!(self.w, "  snez {}, {}", rd_name, rd_name)?;
            }
            BinaryOp::Lt => {
                writeln!(self.w, "  slt {}, {}, {}", rd_name, lvs, rvs)?;
            }
            BinaryOp::Gt => {
                writeln!(self.w, "  slt {}, {}, {}", rd_name, rvs, lvs)?;
            }
            BinaryOp::Le => {
                writeln!(self.w, "  slt {}, {}, {}", rd_name, rvs, lvs)?;
                writeln!(self.w, "  seqz {}, {}", rd_name, rd_name)?;
            }
            BinaryOp::Ge => {
                writeln!(self.w, "  slt {}, {}, {}", rd_name, lvs, rvs)?;
                writeln!(self.w, "  seqz {}, {}", rd_name, rd_name)?;
            }
            BinaryOp::Or => {
                writeln!(self.w, "  or {}, {}, {}", rd_name, lvs, rvs)?;
            }
            BinaryOp::Add => {
                writeln!(self.w, "  add {}, {}, {}", rd_name, lvs, rvs)?;
            }
            BinaryOp::Sub => {
                writeln!(self.w, "  sub {}, {}, {}", rd_name, lvs, rvs)?;
            }
            BinaryOp::Mul => {
                writeln!(self.w, "  mul {}, {}, {}", rd_name, lvs, rvs)?;
            }
            BinaryOp::Div => {
                writeln!(self.w, "  div {}, {}, {}", rd_name, lvs, rvs)?;
            }
            BinaryOp::Mod => {
                writeln!(self.w, "  rem {}, {}, {}", rd_name, lvs, rvs)?;
            }
            _ => unimplemented!("not implemented"),
        }

        self.vm.set_value(*value, ValueStore::Reg(rd));
        Ok(())
    }

    /// check if const, add it to vm
    fn visit_value(&mut self, v: Value) -> Result<()> {
        let data = self.func.unwrap().dfg().value(v);
        match data.kind() {
            ValueKind::Integer(i) => {
                self.vm.set_value(v, ValueStore::Const(i.value()));
            }
            ValueKind::Binary(_) => {
                // do nothing
            }
            _ => unimplemented!("not implemented"),
        }

        Ok(())
    }
}
