use crate::parser::ast::structs::*;
use koopa::ir::{builder_traits::*, *};
use std::sync::atomic::{AtomicUsize, Ordering};

static CNT: AtomicUsize = AtomicUsize::new(0);

macro_rules! next_bb_id {
    ($prefix:expr) => {{
        let cnt = CNT.fetch_add(1, Ordering::SeqCst);
        Some(format!("{}_{}", $prefix, cnt))
    }};
}

macro_rules! insert_op {
    ($program:expr, $params:expr, $op:expr, $l:expr, $r:expr) => {
        {
            let func_data = $program.func_mut($params.func);
            let op = func_data
                .dfg_mut()
                .new_value()
                .binary($op, $l, $r);
            $params.v = Some(op);
            func_data
                .layout_mut()
                .bb_mut($params.bb)
                .insts_mut()
                .extend([op]);
            op
        }
    };
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
        let bb = main_data
            .dfg_mut()
            .new_bb()
            .basic_block(next_bb_id!("%main"));
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
       
        let ret = func_data.dfg_mut().new_value().ret(params.v);
        func_data
            .layout_mut()
            .bb_mut(params.bb)
            .insts_mut()
            .extend([ret]);
        params.v = None; // clear
    }
}

impl Exp {
    /// build exp
    fn build(&self, program: &mut Program, params: &mut BuildParams) {
        match self {
            Exp::Exp(exp) => exp.build(program, params),
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
                let unary_v = params.v.take().unwrap();
                // op instruction
                let op = match op {
                    UnaryOp::Plus => BinaryOp::Add,
                    UnaryOp::Minus => BinaryOp::Sub,
                    UnaryOp::Not => BinaryOp::Eq,
                };
                let func_data = program.func_mut(params.func);
                let zero = func_data.dfg_mut().new_value().integer(0);

                insert_op!(program, params, op, zero, unary_v);
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
                let value = func_data.dfg_mut().new_value().integer(*num);
                params.v = Some(value);
                // just a number, don't need to create a instruction
                // func_data.layout_mut().bb_mut(params.bb).insts_mut().extend([value]);
            }
        }
    }
}

impl AddExp {
    fn build(&self, program: &mut Program, params: &mut BuildParams) {
        match self {
            AddExp::MulExp(exp) => exp.build(program, params),
            AddExp::AddExp(add_exp, op, mul_exp) => {
                add_exp.build(program, params);
                let add_v = params.v.take().unwrap();

                mul_exp.build(program, params);
                let mul_v = params.v.take().unwrap();

                let op = match op {
                    AddOp::Add => BinaryOp::Add,
                    AddOp::Sub => BinaryOp::Sub,
                };

                insert_op!(program, params, op, add_v, mul_v);
                // let func_data = program.func_mut(params.func);
                // let op = func_data
                //     .dfg_mut()
                //     .new_value()
                //     .binary(op, add_v, mul_v);
                // params.v = Some(op);
                // func_data
                //     .layout_mut()
                //     .bb_mut(params.bb)
                //     .insts_mut()
                //     .extend([op]);

            }
        }
    }
}

impl MulExp {
    fn build(&self, program: &mut Program, params: &mut BuildParams) {
        match self {
            MulExp::UnaryExp(exp) => exp.build(program, params),
            MulExp::MulExp(mul_exp, op, unary_exp) => {
                mul_exp.build(program, params);
                let mul_v = params.v.take().unwrap();

                unary_exp.build(program, params);
                let unary_v = params.v.take().unwrap();

                let op = match op {
                    MulOp::Mul => BinaryOp::Mul,
                    MulOp::Div => BinaryOp::Div,
                    MulOp::Mod => BinaryOp::Mod,
                };
                insert_op!(program, params, op, mul_v, unary_v);
            }
        }
    }
}

impl LOrExp {
    fn build(&self, program: &mut Program, params: &mut BuildParams) {
        match self {
            LOrExp::LAndExp(exp) => exp.build(program, params),
            LOrExp::LOrExp(lor_exp, land_exp) => {
                lor_exp.build(program, params);
                let lor_v = params.v.take().unwrap();
                
                land_exp.build(program, params);
                let land_v = params.v.take().unwrap();
                

                let func_data = program.func_mut(params.func);
                let zero = func_data.dfg_mut().new_value().integer(0);
                let or_v = func_data.dfg_mut().new_value().binary(BinaryOp::Or, lor_v, land_v);
                let res = func_data.dfg_mut().new_value().binary(BinaryOp::NotEq, or_v, zero);

                params.v = Some(res);
                func_data.layout_mut().bb_mut(params.bb).insts_mut().extend([or_v, res]);
            }
        }
    }
}

impl LAndExp {
    fn build(&self, program: &mut Program, params: &mut BuildParams) {
        match self {
            LAndExp::EqExp(exp) => exp.build(program, params),
            LAndExp::LAndExp(land_exp, eq_exp) => {
                land_exp.build(program, params);
                let land_v = params.v.take().unwrap();
                
                eq_exp.build(program, params);
                let eq_v = params.v.take().unwrap();

                let func_data = program.func_mut(params.func);
                let zero = func_data.dfg_mut().new_value().integer(0);

                let l_v = func_data.dfg_mut().new_value().binary(BinaryOp::NotEq, land_v, zero);
                let r_v = func_data.dfg_mut().new_value().binary(BinaryOp::NotEq, eq_v, zero);
                let res = func_data.dfg_mut().new_value().binary(BinaryOp::And, l_v, r_v);
                params.v = Some(res);
                func_data.layout_mut().bb_mut(params.bb).insts_mut().extend([l_v, r_v, res]);
            }
        }
    }
}

impl EqExp {
    fn build(&self, program: &mut Program, params: &mut BuildParams) {
        match self {
            EqExp::RelExp(exp) => exp.build(program, params),
            EqExp::EqExp(eq_exp, eq_op, rel_exp) => {
                eq_exp.build(program, params);
                let eq_v = params.v.take().unwrap();
                
                rel_exp.build(program, params);
                let rel_v = params.v.take().unwrap();
                
                let op = match eq_op {
                    EqOp::Eq => BinaryOp::Eq,
                    EqOp::Ne => BinaryOp::NotEq,
                };
                insert_op!(program, params, op, eq_v, rel_v);
            }
        }
    }
}

impl RelExp {
    fn build(&self, program: &mut Program, params: &mut BuildParams) {
        match self {
            RelExp::AddExp(exp) => exp.build(program, params),
            RelExp::RelExp(rel_exp, rel_op, add_exp) => {
                rel_exp.build(program, params);
                let rel_v = params.v.take().unwrap();
                
                add_exp.build(program, params);
                let add_v = params.v.take().unwrap();

                let op = match rel_op {
                    RelOp::Lt => BinaryOp::Lt,
                    RelOp::Le => BinaryOp::Le,
                    RelOp::Gt => BinaryOp::Gt,
                    RelOp::Ge => BinaryOp::Ge,
                };
                insert_op!(program, params, op, rel_v, add_v);
            }
        }
    }
}