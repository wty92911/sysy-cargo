use koopa::back::{self, NameManager};
use koopa::ir::entities::{FunctionData, ValueData};
use koopa::ir::layout::BasicBlockNode;
use koopa::ir::values::*;
use koopa::ir::{BasicBlock, Program, Type, TypeKind, Value, ValueKind};
use std::io::{Result, Write};

/// Visitor for generating the in-memory form Koopa IR program into the riscv
#[derive(Default)]
pub struct Visitor;

impl<W: Write> back::Visitor<W> for Visitor {
    type Output = ();

    fn visit(
        &mut self,
        w: &mut W,
        nm: &mut back::NameManager,
        program: &koopa::ir::Program,
    ) -> std::io::Result<Self::Output> {
        let mut visitor = VisitorImpl {
            w,
            nm,
            program,
            func: None,
        };
        visitor.visit()
    }
}

/// The implementation of riscv Koopa IR generator.
struct VisitorImpl<'a, W: Write> {
    w: &'a mut W,
    nm: &'a mut NameManager,
    program: &'a Program,
    func: Option<&'a FunctionData>,
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
            let value_data = self.func.unwrap().dfg().value(*inst);
            self.visit_local_inst(value_data)?;
        }
        Ok(())
    }

    /// Generates the given local instruction.
    fn visit_local_inst(&mut self, inst: &ValueData) -> Result<()> {

        match inst.kind() {
            ValueKind::Return(v) => self.visit_return(v)?,
            _ => unimplemented!(),
        };
        Ok(())
    }

    /// Generates function return.
    fn visit_return(&mut self, ret: &Return) -> Result<()> {
        if let Some(val) = ret.value() {
            self.visit_value(val)?;

        }
        writeln!(self.w, "  ret")?;
        Ok(())
    }

    /// Generates the given value.
    fn visit_value(&mut self, value: Value) -> Result<()> {
        let value = self.func.unwrap().dfg().value(value);
        match value.kind() {
            ValueKind::Integer(v) => {
                writeln!(self.w, "  li a0, {}", v.value())?;
            }
            _ => unimplemented!()
        };
        Ok(())
    }

}
