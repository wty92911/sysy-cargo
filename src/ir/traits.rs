use crate::ir::structs::*;
use koopa::ir::{builder_traits::*, *};
use core::panic;
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::ir::vm::{self, ValueManager};
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

macro_rules! jump_ifn_ret {
    ($program:expr, $params:expr, $end_bb:expr) => {
        let func_data = $program.func($params.func);
        match func_data.layout().bbs().node(&$params.bb).unwrap().insts().back_key() {
            Some(v) if matches!(func_data.dfg().value(*v).kind(), ValueKind::Return(_)) => {},
            _ => {
                let func_data = $program.func_mut($params.func);
                let jump = func_data.dfg_mut().new_value().jump($end_bb);
                func_data.layout_mut().bb_mut($params.bb).insts_mut().extend([jump]);
            }
        }
    }
}
macro_rules! if_else {
    ($program:expr, $params:expr, $exp:expr, $true_stmt:expr, $false_stmt:expr) => {
        $exp.build($program, $params);
        let exp_v = $params.v.take().unwrap();
        let func_data = $program.func_mut($params.func);
        let true_bb = func_data.dfg_mut().new_bb().basic_block(next_bb_id!("%if"));
        let false_bb = func_data.dfg_mut().new_bb().basic_block(next_bb_id!("%else"));
        let end_bb = func_data.dfg_mut().new_bb().basic_block(next_bb_id!("%if_end"));
        func_data.layout_mut().bbs_mut().extend([true_bb, false_bb, end_bb]);

        let branch = func_data.dfg_mut().new_value().branch(exp_v, true_bb, false_bb);
        func_data.layout_mut().bb_mut($params.bb).insts_mut().extend([branch]);

        $params.bb = true_bb;
        $params.vm.push();
        $true_stmt.build($program, $params);
        jump_ifn_ret!($program, $params, end_bb);
        $params.vm.pop();


        $params.bb = false_bb;
        if let Some(false_stmt) = $false_stmt {
            $params.vm.push();
            false_stmt.build($program, $params);
            $params.vm.pop();
        }
        jump_ifn_ret!($program, $params, end_bb);


        $params.bb = end_bb;
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
        let bb = main_data.dfg_mut().new_bb().basic_block(next_bb_id!("%main"));
        main_data.layout_mut().bbs_mut().push_key_back(bb).unwrap();

        let mut params = BuildParams {
            func: main,
            bb,
            v: None,
            vm: ValueManager::new(),
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

    /// variable manager
    vm: ValueManager,
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
        params.vm.push();
        for item in self.items.iter() {
            item.build(program, params);
        }
        params.vm.pop();
    }
}

impl BlockItem {
    fn build(&self, program: &mut Program, params: &mut BuildParams) {
        match self {
            BlockItem::Stmt(stmt) => stmt.build(program, params),
            BlockItem::Decl(decl) => decl.build(program, params),
        }
    }
}

impl Stmt {
    fn build(&self, program: &mut Program, params: &mut BuildParams) {
        match self {
            Stmt::Open(stmt) => stmt.build(program, params),
            Stmt::Closed(stmt) => stmt.build(program, params),
        }
    }
}

impl OpenStmt {
    fn build(&self, program: &mut Program, params: &mut BuildParams) {
        match self {
            OpenStmt::If(exp, stmt) => {
                if_else!(program, params, exp, stmt, Option::<OpenStmt>::None);
            }
            OpenStmt::IfElse(exp, stmt1, stmt2) => {
               if_else!(program, params, exp, stmt1, Some(stmt2));
            }
        }
    }
}

impl ClosedStmt {
    fn build(&self, program: &mut Program, params: &mut BuildParams) {
        match self {
            ClosedStmt::Simple(stmt) => stmt.build(program, params),
            ClosedStmt::IfElse(exp, stmt1, stmt2) => {
                if_else!(program, params, exp, stmt1, Some(stmt2));
            }
        }
    }
}

impl SimpleStmt {
    fn build(&self, program: &mut Program, params: &mut BuildParams) {
        match self {
            SimpleStmt::LVal(lval, exp) => {
                exp.build(program, params);
                let v = params.v.take().unwrap();
                let func_data = program.func_mut(params.func);
                let lv = params.vm.get(&lval).unwrap();
                match *lv {
                    vm::Decl::Var(lv) => {
                        let s = func_data.dfg_mut().new_value().store(v, lv);

                        func_data.layout_mut().bb_mut(params.bb).insts_mut().extend([s]);
                    }
                    _ => panic!()
                }
            }
            SimpleStmt::Exp(_exp) => {} // todo?
            SimpleStmt::Block(block) => {
                block.build(program, params)
            }
            SimpleStmt::Ret(exp) => {
                match exp {
                    Some(exp) => {
                        let func_data = program.func_mut(params.func);

                        match func_data.layout().bbs().node(&params.bb).unwrap().insts().back_key() {
                            Some(v) if matches!(func_data.dfg().value(*v).kind(), ValueKind::Return(_)) => {
                                let ret_bb = func_data.dfg_mut().new_bb().basic_block(next_bb_id!("%ret"));
                                func_data.layout_mut().bbs_mut().extend([ret_bb]);
                                params.bb = ret_bb;
                            }
                            _ => {}
                        }
                        exp.build(program, params);
                        
                        // create a new basic block for ret
                        let func_data = program.func_mut(params.func);
                        let ret = func_data.dfg_mut().new_value().ret(params.v);
                        func_data.layout_mut().bb_mut(params.bb).insts_mut().extend([ret]);
                        params.v = None; // clear
                    }
                    None => {}
                }
            }
        }
    }
}

impl Decl {
    fn build(&self, program: &mut Program, params: &mut BuildParams) {
        match self {
            Decl::Const(decl) => decl.build(program, params),
            Decl::Var(decl) => decl.build(program, params),
        }
    }
}

impl ConstDecl {
    fn build(&self, program: &mut Program, params: &mut BuildParams) {
        for def in self.defs.iter() {
            def.build(program, params);
        }
    }
}

impl VarDecl {
    fn build(&self, program: &mut Program, params: &mut BuildParams) {
        for def in self.defs.iter() {
            def.build(program, params);
        }
    }
}

impl ConstDef {
    fn build(&self, _program: &mut Program, params: &mut BuildParams) {
        let v = self.value.calc(params);
        params.vm.insert_const(self.ident.as_str(), v);
    }
}

impl VarDef {
    fn build(&self, program: &mut Program, params: &mut BuildParams) {
        match self {
            VarDef::Ident(ident) => {
                let func_data = program.func_mut(params.func);
                let v = func_data.dfg_mut().new_value().alloc(Type::get_i32());
                func_data.layout_mut().bb_mut(params.bb).insts_mut().extend([v]);
                params.vm.insert_var(ident.as_str(), v);
            }
            VarDef::InitVal(ident,exp ) => {
                exp.build(program, params);

                let func_data = program.func_mut(params.func);
                let v = func_data.dfg_mut().new_value().alloc(Type::get_i32());
                params.vm.insert_var(ident.as_str(), v);
                let exp_v = params.v.take().unwrap();
                let s = func_data.dfg_mut().new_value().store(exp_v, v);
                func_data.layout_mut().bb_mut(params.bb).insts_mut().extend([v, s]);
            }
        }
    }
}
impl Exp {
    /// build exp
    fn build(&self, program: &mut Program, params: &mut BuildParams) {
        match self {
            Exp::Exp(exp) => exp.build(program, params),
        }
    }

    fn calc(&self, params: &mut BuildParams) -> i32 {
        match self {
            Exp::Exp(exp) => exp.calc(params),
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
            }
        }
    }

    fn calc(&self, params: &mut BuildParams) -> i32 {
        match self {
            AddExp::MulExp(exp) => exp.calc(params),
            AddExp::AddExp(add_exp, op, mul_exp) => match op {
                AddOp::Add => add_exp.calc(params) + mul_exp.calc(params),
                AddOp::Sub => add_exp.calc(params) - mul_exp.calc(params),
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

    fn calc(&self, params: &mut BuildParams) -> i32 {
        match self {
            MulExp::UnaryExp(exp) => exp.calc(params),
            MulExp::MulExp(mul_exp, op, unary_exp) => match op {
                MulOp::Mul => mul_exp.calc(params) * unary_exp.calc(params),
                MulOp::Div => mul_exp.calc(params) / unary_exp.calc(params),
                MulOp::Mod => mul_exp.calc(params) % unary_exp.calc(params),
            }
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

    fn calc(&self, params: &mut BuildParams) -> i32 {
        match self {
            UnaryExp::PrimaryExp(exp) => exp.calc(params),
            UnaryExp::UnaryOp(op, exp) => match op {
                UnaryOp::Plus => exp.calc(params),
                UnaryOp::Minus => -exp.calc(params),
                UnaryOp::Not => !exp.calc(params),
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
            PrimaryExp::LVal(lval) => {
                let func_data = program.func_mut(params.func);
                let v = params.vm.get(&lval).unwrap();
                match *v {
                    vm::Decl::Const(v) => {
                        let value = func_data.dfg_mut().new_value().integer(v);
                        params.v = Some(value);
                    }
                    vm::Decl::Var(v) => {
                        let l = func_data.dfg_mut().new_value().load(v);
                        func_data.layout_mut().bb_mut(params.bb).insts_mut().extend([l]);
                        params.v = Some(l);
                    }
                }
               
            }
        }
    }

    fn calc(&self, params: &mut BuildParams) -> i32 {
        match self {
            PrimaryExp::Exp(exp) => exp.calc(params),
            PrimaryExp::Number(num) => *num,
            PrimaryExp::LVal(lval) => {
                let decl = params.vm.get(lval).unwrap();
                match decl {
                    vm::Decl::Const(v) => *v,
                    vm::Decl::Var(_) => panic!()
                }
            },
        }
    }
}

impl LOrExp {
    fn build(&self, program: &mut Program, params: &mut BuildParams) {
        match self {
            LOrExp::LAndExp(exp) => exp.build(program, params),
            LOrExp::LOrExp(lor_exp, land_exp) => {
                let func_data = program.func_mut(params.func);
                let result = func_data.dfg_mut().new_value().alloc(Type::get_i32());
                func_data.layout_mut().bb_mut(params.bb).insts_mut().extend([result]);

                let r_bb = func_data.dfg_mut().new_bb().basic_block(next_bb_id!("%or_r"));
                let end_bb = func_data.dfg_mut().new_bb().basic_block(next_bb_id!("%or_end"));
                func_data.layout_mut().bbs_mut().extend([r_bb, end_bb]);

                lor_exp.build(program, params);

                let lor_v = params.v.take().unwrap();
                let func_data = program.func_mut(params.func);
                let l_store = func_data.dfg_mut().new_value().store(lor_v, result);
                let l_br = func_data.dfg_mut().new_value().branch(lor_v, end_bb, r_bb);
                func_data.layout_mut().bb_mut(params.bb).insts_mut().extend([l_store, l_br]);

                params.bb = r_bb;
                land_exp.build(program, params);
                let land_v = params.v.take().unwrap();
                let func_data = program.func_mut(params.func);
                let r_store = func_data.dfg_mut().new_value().store(land_v, result);
                let r_br = func_data.dfg_mut().new_value().jump(end_bb);
                func_data.layout_mut().bb_mut(params.bb).insts_mut().extend([r_store, r_br]);

                params.bb = end_bb;
                let func_data = program.func_mut(params.func);
                let load = func_data.dfg_mut().new_value().load(result);
                let zero = func_data.dfg_mut().new_value().integer(0);
                let res = func_data.dfg_mut().new_value().binary(BinaryOp::NotEq, load, zero);

                params.v = Some(res);
                func_data.layout_mut().bb_mut(params.bb).insts_mut().extend([load, res]);
            }
        }
    }

    fn calc(&self, params: &mut BuildParams) -> i32 {
        match self {
            LOrExp::LAndExp(exp) => exp.calc(params),
            LOrExp::LOrExp(lor_exp, land_exp) => 
                (lor_exp.calc(params) != 0 || land_exp.calc(params) != 0).into()
        }
    }
}

impl LAndExp {
    fn build(&self, program: &mut Program, params: &mut BuildParams) {
        match self {
            LAndExp::EqExp(exp) => exp.build(program, params),
            LAndExp::LAndExp(land_exp, eq_exp) => {
                let func_data = program.func_mut(params.func);
                let result = func_data.dfg_mut().new_value().alloc(Type::get_i32());
                func_data.layout_mut().bb_mut(params.bb).insts_mut().extend([result]);

                let r_bb = func_data.dfg_mut().new_bb().basic_block(next_bb_id!("%and_r"));
                let end_bb = func_data.dfg_mut().new_bb().basic_block(next_bb_id!("%and_end"));
                func_data.layout_mut().bbs_mut().extend([r_bb, end_bb]);

                land_exp.build(program, params);

                let land_v = params.v.take().unwrap();
                let func_data = program.func_mut(params.func);
                let l_store = func_data.dfg_mut().new_value().store(land_v, result);
                let zero = func_data.dfg_mut().new_value().integer(0);
                let eq_zero = func_data.dfg_mut().new_value().binary(BinaryOp::Eq, land_v, zero);
                let l_br = func_data.dfg_mut().new_value().branch(eq_zero, end_bb, r_bb);
                func_data.layout_mut().bb_mut(params.bb).insts_mut().extend([l_store, eq_zero, l_br]);

                params.bb = r_bb;
                eq_exp.build(program, params);
                let eq_v = params.v.take().unwrap();
                let func_data = program.func_mut(params.func);
                let r_store = func_data.dfg_mut().new_value().store(eq_v, result);
                let r_br = func_data.dfg_mut().new_value().jump(end_bb);
                func_data.layout_mut().bb_mut(params.bb).insts_mut().extend([r_store, r_br]);

                params.bb = end_bb;
                let func_data = program.func_mut(params.func);
                let load = func_data.dfg_mut().new_value().load(result);
                let res = func_data.dfg_mut().new_value().binary(BinaryOp::NotEq, load, zero);

                params.v = Some(res);
                func_data.layout_mut().bb_mut(params.bb).insts_mut().extend([load, res]);
            }
        }
    }

    fn calc(&self, params: &mut BuildParams) -> i32 {
        match self {
            LAndExp::EqExp(exp) => exp.calc(params),
            LAndExp::LAndExp(land_exp, eq_exp) => 
               (land_exp.calc(params) != 0 && eq_exp.calc(params) != 0).into()
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

    fn calc(&self, params: &mut BuildParams) -> i32 {
        match self {
            EqExp::RelExp(exp) => exp.calc(params),
            EqExp::EqExp(eq_exp, eq_op, rel_exp) => match eq_op {
                EqOp::Eq => eq_exp.calc(params) == rel_exp.calc(params) ,
                EqOp::Ne => eq_exp.calc(params) != rel_exp.calc(params),
            }.into()
            
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

    fn calc(&self, params: &mut BuildParams) -> i32 {
        match self {
            RelExp::AddExp(exp) => exp.calc(params),
            RelExp::RelExp(rel_exp, rel_op, add_exp) => match rel_op {
                RelOp::Lt => rel_exp.calc(params) < add_exp.calc(params),
                RelOp::Le => rel_exp.calc(params) <= add_exp.calc(params),
                RelOp::Gt => rel_exp.calc(params) > add_exp.calc(params),
                RelOp::Ge => rel_exp.calc(params) >= add_exp.calc(params),
            }.into()
        }
    }
}