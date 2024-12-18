use crate::parser::ast::structs::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use koopa::ir::{builder_traits::*, *};

static CNT: AtomicUsize = AtomicUsize::new(0);

macro_rules! next_bb_id {
    ($prefix:expr) => {{
        let cnt = CNT.fetch_add(1, Ordering::SeqCst);
        Some(format!("{}_{}", $prefix, cnt))
    }};
}

impl Into<Program> for CompUnit {
    fn into(self) -> Program {
        let mut program = Program::new();

        // create func
        let main = program.new_func(FunctionData::new(
            format!("@{}", self.func_def.ident),
            Vec::new(),
            self.func_def.func_type.into(),
        ));

        // fill func
        let main_data = program.func_mut(main);
        let bb = main_data.dfg_mut().new_bb().basic_block(next_bb_id!("%main"));
        main_data.layout_mut().bbs_mut().push_key_back(bb).unwrap();

        let mut params = BuildParams {
            func: main,
            bb,
            v: None,
        };
        // parse exp
        self.func_def.block.build(&mut program, &mut params);
        program
    }
}

/// Build params.
struct BuildParams {
    func: Function,
    bb: BasicBlock,
    /// last value
    v: Option<Value>,
}

impl Into<Type> for FuncType {
    fn into(self) -> Type {
        match self {
            FuncType::Int => Type::get_i32(),
        }
    }
}

impl Block {
    fn build(&self, program: &mut Program, params: &mut BuildParams) {
        self.stmt.build(program, params);
    }
}

impl Stmt {
    fn build(&self, program: &mut Program, params: &mut BuildParams) {
        self.exp.build(program, params);
        
        // create a new basic block for ret
        let func_data = program.func_mut(params.func);
        let old_bb = params.bb;
        params.bb = func_data.dfg_mut().new_bb().basic_block(next_bb_id!("%ret"));
        func_data.layout_mut().bbs_mut().push_key_back(params.bb).unwrap();
        let ret = func_data.dfg_mut().new_value().ret(params.v);
        func_data.layout_mut().bb_mut(params.bb).insts_mut().extend([ret]);
        params.v = None; // clear
        params.bb = old_bb;
    }
}

impl Exp {
    /// build exp
    fn build(&self, program: &mut Program, params: &mut BuildParams) {
        // create a new basic block for exp
        let func_data = program.func_mut(params.func);
        

        match self {
            Exp::UnaryExp(exp) => exp.build(program, params),
        }
    }
}

impl UnaryExp {
    fn build(&self, program: &mut Program, params: &mut BuildParams) {
        match self {
            UnaryExp::PrimaryExp(exp) => exp.build(program, params),
            UnaryExp::UnaryOp(op, exp) => {
                // build next exp recursively
                exp.build(program, params);

                // op instruction
                let op = match op {
                    UnaryOp::Plus => return,
                    UnaryOp::Minus => BinaryOp::Sub,
                    UnaryOp::Not => BinaryOp::Eq,
                };
                
                // create a new basic block for exp
                let func_data = program.func_mut(params.func);
                let old_bb = params.bb;
                params.bb = func_data.dfg_mut().new_bb().basic_block(next_bb_id!("%exp"));
                func_data.layout_mut().bbs_mut().push_key_back(params.bb).unwrap();
                

                
                let zero = func_data.dfg_mut().new_value().integer(0);
                let op = func_data.dfg_mut().new_value().binary(op, zero, params.v.unwrap());
                params.v = Some(op);
                func_data.layout_mut().bb_mut(params.bb).insts_mut().extend([op]);
                
                // recover bb
                params.bb = old_bb;
            }
        }
    }
}

impl PrimaryExp {
    fn build(&self, program: &mut Program, params: &mut BuildParams) {
        match self {
            PrimaryExp::Exp(exp) => exp.build(program, params),
            PrimaryExp::Number(num) => {
                let func_data = program.func_mut(params.func);
                let value = func_data
                    .dfg_mut()
                    .new_value()
                    .integer(*num);
                params.v = Some(value);
                // just a number, don't need to create a instruction
                // func_data.layout_mut().bb_mut(params.bb).insts_mut().extend([value]);
            }
        }
    }
}
