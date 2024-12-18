#[derive(Debug)]
pub struct CompUnit {
    pub func_def: FuncDef,
}
#[derive(Debug)]
pub struct FuncDef {
    pub func_type: FuncType,
    pub ident: String,
    pub block: Block,
}
#[derive(Debug)]
pub enum FuncType {
    Int,
}
#[derive(Debug)]
pub struct Block {
    pub stmt: Stmt,
}
#[derive(Debug)]
pub struct Stmt {
    pub exp: Exp,
}

#[derive(Debug)]
pub enum Exp {
    UnaryExp(UnaryExp),
}

#[derive(Debug)]
pub enum UnaryExp{
    PrimaryExp(PrimaryExp),
    UnaryOp(UnaryOp, Box<UnaryExp>)
}

#[derive(Debug)]
pub enum UnaryOp{
    Plus,
    Minus,
    Not,
}

#[derive(Debug)]
pub enum PrimaryExp{
    Exp(Box<Exp>), // bracket
    Number(Number),
}

pub type Number = i32;
