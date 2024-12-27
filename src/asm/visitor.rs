use crate::asm::vm::*;
use koopa::ir::entities::{FunctionData, ValueData};
use koopa::ir::layout::BasicBlockNode;
use koopa::ir::values::*;
use koopa::ir::{BasicBlock, Program, TypeKind, Value, ValueKind};
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
        // calc stack size
        let mut stack_size = 0;
        node.insts().iter().for_each(|(value,_)| {
            match self.func.unwrap().dfg().value(*value).ty().kind() {
                TypeKind::Int32 => stack_size += 4,
                TypeKind::Pointer(_) => stack_size += 4,
                _ => {}
            }
        });
        stack_size = (stack_size + 15) / 16 * 16;
        if stack_size > 2048 {
            todo!();
        }
        writeln!(self.w, "  addi sp, sp, -{}", stack_size)?;
        self.vm.set_max_offset(stack_size as u32);
        for inst in node.insts().keys() {
            self.visit_local_inst(inst)?;
        }
        Ok(())
    }

    /// Generates the given local instruction.
    fn visit_local_inst(&mut self, inst: &Value) -> Result<()> {
        let value_data = self.func.unwrap().dfg().value(*inst);
        match value_data.kind() {
            ValueKind::Alloc(_) => {
                self.vm.alloc(inst, 4)?;
            }
            ValueKind::Load(l) => {
                self.vm.alloc(inst, 4)?;
                // warn! must be sure the src() value is loaded from memory,
                // or the value of l.src never use again.

                // to make sure of this, we must store the value of reg to memory
                // when the value is changed

                let reg = self.vm.load_to_reg(l.src(), None, self.w)?;
                self.vm.set_value_store(*inst, ValueStore::Reg(reg));
                // copy value of reg to memory
                self.vm.copy_to_mem(reg, self.w);
            }
            ValueKind::Store(s) => {
                self.visit_const(s.value())?;
                let reg = self.vm.load_to_reg(s.value(), None, self.w)?;
                self.vm.set_value_store(s.dest(), ValueStore::Reg(reg));
                // here we use copy to avoid load again when the value of reg is used
                self.vm.copy_to_mem(reg, self.w);
            }
            ValueKind::Binary(b) => {
                self.vm.alloc(inst, 4)?;
                self.visit_binary(inst, b)?;
                let reg = self.vm.load_to_reg(*inst, None, self.w)?;
                self.vm.copy_to_mem(reg, self.w)
            }
            ValueKind::Return(v) => self.visit_return(v)?,
            _ => unimplemented!(),
        };
        Ok(())
    }



    /// Generates function return.
    fn visit_return(&mut self, ret: &Return) -> Result<()> {
        if let Some(val) = ret.value() {
            self.visit_const(val)?;
            self.vm.load_to_reg(val, Some(A0), self.w)?;
        }
        writeln!(self.w, "  ret")?;
        Ok(())
    }

    /// Generates the given binary operation._
    fn visit_binary(&mut self, value: &Value, b: &Binary) -> Result<()> {
        self.visit_const(b.lhs())?;
        self.visit_const(b.rhs())?;

        // deal reg, for now load all const to reg
        let rd = self.vm.alloc_reg(None, self.w);
        self.vm.set_value_store(*value, ValueStore::Reg(rd));
        self.vm.lock_reg(rd);
        let rd_name = self.vm.get_reg_name(rd);

        let lvs = self.vm.load_to_reg(b.lhs(), None, self.w)?;
        self.vm.lock_reg(lvs);
        let rvs = self.vm.load_to_reg(b.rhs(), None, self.w)?;
        self.vm.unlock_reg(lvs);
        self.vm.unlock_reg(rd);
        let (lvs, rvs) = (self.vm.get_reg_name(lvs), self.vm.get_reg_name(rvs));
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
            BinaryOp::And => {
                writeln!(self.w, "  and {}, {}, {}", rd_name, lvs, rvs)?;
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

        Ok(())
    }

    /// check if const, add it to vm
    fn visit_const(&mut self, v: Value) -> Result<()> {
        let data = self.func.unwrap().dfg().value(v);
        match data.kind() {
            ValueKind::Integer(i) => {
                self.vm.set_value_store(v, ValueStore::Mem(Mem::Const(i.value())));
            }
            _ => {}
        }
        Ok(())
    }
}
