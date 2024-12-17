use crate::parser::ast::structs::*;
use koopa::ir::{builder_traits::*, *};

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
        let bb = main_data.dfg_mut().new_bb().basic_block(None);
        main_data.layout_mut().bbs_mut().push_key_back(bb).unwrap();

        let value = main_data
            .dfg_mut()
            .new_value()
            .integer(self.func_def.block.stmt.number);
        let ret = main_data.dfg_mut().new_value().ret(Some(value));
        main_data.layout_mut().bb_mut(bb).insts_mut().extend([ret]);
        program
    }
}

impl Into<Type> for FuncType {
    fn into(self) -> Type {
        match self {
            FuncType::Int => Type::get_i32(),
        }
    }
}
