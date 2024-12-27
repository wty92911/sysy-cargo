#[derive(Debug)]
pub struct CompUnit {
    pub func_def: FuncDef,
}

#[derive(Debug)]
pub enum Decl {
    Const(ConstDecl),
    Var(VarDecl)
}


#[derive(Debug)]
pub struct ConstDecl {
    pub defs: Vec<ConstDef>,
}

#[derive(Debug)]
pub struct VarDecl {
    pub defs: Vec<VarDef>,
}


#[derive(Debug)]
pub struct ConstDef {
    pub ident: String,
    pub value: ConstInitVal,
}

#[derive(Debug)]
pub enum VarDef {
    Ident(Ident),
    InitVal(Ident, InitVal)
}

pub type ConstInitVal = ConstExp;

pub type ConstExp = Exp;

pub type Ident = String;

pub type InitVal = Exp;

#[derive(Debug)]
pub struct FuncDef {
    pub func_type: FuncType,
    pub ident: Ident,
    pub block: Block,
}
#[derive(Debug)]
pub enum FuncType {
    Int,
}
#[derive(Debug)]
pub struct Block {
    pub items: Vec<BlockItem>,
}

#[derive(Debug)]
pub enum BlockItem {
    Decl(Decl),
    Stmt(Stmt),
}

#[derive(Debug)]
pub enum Stmt {
    LVal(LVal, Exp),
    Ret(Exp)
}

#[derive(Debug)]
pub enum Exp {
    Exp(LOrExp),
}

pub type LVal = Ident;

#[derive(Debug)]
pub enum PrimaryExp {
    Exp(Box<Exp>),
    LVal(LVal),
    Number(Number),
}

pub type Number = i32;

#[derive(Debug)]
pub enum UnaryExp {
    PrimaryExp(PrimaryExp),
    UnaryOp(UnaryOp, Box<UnaryExp>),
}

#[derive(Debug)]
pub enum UnaryOp {
    Plus,
    Minus,
    Not,
}


#[derive(Debug)]
pub enum MulExp {
    UnaryExp(UnaryExp),
    MulExp(Box<MulExp>, MulOp, UnaryExp),
}

#[derive(Debug)]
pub enum MulOp {
    Mul,
    Div,
    Mod,
}

#[derive(Debug)]
pub enum AddExp {
    MulExp(MulExp),
    AddExp(Box<AddExp>, AddOp, MulExp),
}

#[derive(Debug)]
pub enum AddOp {
    Add,
    Sub,
}

#[derive(Debug)]
pub enum RelExp {
    AddExp(AddExp),
    RelExp(Box<RelExp>, RelOp, AddExp),
}

#[derive(Debug)]
pub enum RelOp {
    Lt,
    Le,
    Gt,
    Ge,
}

#[derive(Debug)]
pub enum EqExp {
    RelExp(RelExp),
    EqExp(Box<EqExp>, EqOp, RelExp),
}

#[derive(Debug)]
pub enum EqOp {
    Eq,
    Ne,
}

#[derive(Debug)]
pub enum LAndExp {
    EqExp(EqExp),
    LAndExp(Box<LAndExp>, EqExp),
}

#[derive(Debug)]
pub enum LOrExp {
    LAndExp(LAndExp),
    LOrExp(Box<LOrExp>, LAndExp),
}

