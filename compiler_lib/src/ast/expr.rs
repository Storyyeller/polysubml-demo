use crate::ast::LetPattern;
use crate::ast::Literal;
use crate::ast::Op;
use crate::ast::OpType;
use crate::ast::STypeExpr;
use crate::ast::Statement;
use crate::ast::StringId;
use crate::ast::TypeParam;
use crate::spans::Spanned;

pub type KeyPair = (Spanned<StringId>, Box<SExpr>, bool, Option<STypeExpr>);

#[derive(Debug, Clone, Copy)]
pub enum InstantiateSourceKind {
    ImplicitCall,
    ImplicitRecord,
    ExplicitParams(bool),
}

// Struct types for each Expr variant
#[derive(Debug, Clone)]
pub struct BinOpExpr {
    pub lhs: Box<SExpr>,
    pub rhs: Box<SExpr>,
    pub op_type: OpType,
    pub op: Op,
}

#[derive(Debug, Clone)]
pub struct BlockExpr {
    pub statements: Vec<Statement>,
    pub expr: Box<SExpr>,
}

#[derive(Debug, Clone)]
pub struct CallExpr {
    pub func: Box<SExpr>,
    pub arg: Box<SExpr>,
}

#[derive(Debug, Clone)]
pub struct CaseExpr {
    pub tag: Spanned<StringId>,
    pub expr: Box<SExpr>,
}

#[derive(Debug, Clone)]
pub struct FieldAccessExpr {
    pub expr: Box<SExpr>,
    pub field: Spanned<StringId>,
}

#[derive(Debug, Clone)]
pub struct FieldSetExpr {
    pub expr: Box<SExpr>,
    pub field: Spanned<StringId>,
    pub value: Box<SExpr>,
}

#[derive(Debug, Clone)]
pub struct FuncDefExpr {
    pub type_params: Option<Vec<TypeParam>>,
    pub param: Spanned<LetPattern>,
    pub return_type: Option<STypeExpr>,
    pub body: Box<SExpr>,
}

#[derive(Debug, Clone)]
pub struct IfExpr {
    pub cond: Spanned<Box<SExpr>>,
    pub then_expr: Box<SExpr>,
    pub else_expr: Box<SExpr>,
}

#[derive(Debug, Clone)]
pub struct InstantiateExistExpr {
    pub expr: Box<SExpr>,
    pub types: Spanned<Vec<(StringId, STypeExpr)>>,
    pub source: InstantiateSourceKind,
}

#[derive(Debug, Clone)]
pub struct InstantiateUniExpr {
    pub expr: Box<SExpr>,
    pub types: Spanned<Vec<(StringId, STypeExpr)>>,
    pub source: InstantiateSourceKind,
}

#[derive(Debug, Clone)]
pub struct LiteralExpr {
    pub lit_type: Literal,
    pub value: Spanned<String>,
}

#[derive(Debug, Clone)]
pub struct LoopExpr {
    pub body: Box<SExpr>,
}

#[derive(Debug, Clone)]
pub struct MatchExpr {
    pub expr: Spanned<Box<SExpr>>,
    pub cases: Vec<(Spanned<LetPattern>, Box<SExpr>)>,
}

#[derive(Debug, Clone)]
pub struct RecordExpr {
    pub fields: Vec<KeyPair>,
}

#[derive(Debug, Clone)]
pub struct TypedExpr {
    pub expr: Box<SExpr>,
    pub type_expr: STypeExpr,
}

#[derive(Debug, Clone)]
pub struct VariableExpr {
    pub name: StringId,
}

#[derive(Debug, Clone)]
pub enum Expr {
    BinOp(BinOpExpr),
    Block(BlockExpr),
    Call(CallExpr),
    Case(CaseExpr),
    FieldAccess(FieldAccessExpr),
    FieldSet(FieldSetExpr),
    FuncDef(FuncDefExpr),
    If(IfExpr),
    InstantiateExist(InstantiateExistExpr),
    InstantiateUni(InstantiateUniExpr),
    Literal(LiteralExpr),
    Loop(LoopExpr),
    Match(MatchExpr),
    Record(RecordExpr),
    Typed(TypedExpr),
    Variable(VariableExpr),
}
pub type SExpr = Spanned<Expr>;

// Constructor functions for Expr variants
pub fn binop(lhs: Box<SExpr>, rhs: Box<SExpr>, op_type: OpType, op: Op) -> Expr {
    Expr::BinOp(BinOpExpr { lhs, rhs, op_type, op })
}

pub fn block(statements: Vec<Statement>, expr: Box<SExpr>) -> Expr {
    Expr::Block(BlockExpr { statements, expr })
}

pub fn call(func: Box<SExpr>, arg: Box<SExpr>) -> Expr {
    let func = match &func.0 {
        Expr::InstantiateUni(e) => func,
        _ => {
            let span = func.1;
            Box::new((
                instantiate_uni(Box::new(*func), (vec![], span), InstantiateSourceKind::ImplicitCall),
                span,
            ))
        }
    };

    Expr::Call(CallExpr { func, arg })
}

pub fn case(tag: Spanned<StringId>, expr: Box<SExpr>) -> Expr {
    Expr::Case(CaseExpr { tag, expr })
}

pub fn field_access(expr: Box<SExpr>, field: Spanned<StringId>) -> Expr {
    Expr::FieldAccess(FieldAccessExpr { expr, field })
}

pub fn field_set(expr: Box<SExpr>, field: Spanned<StringId>, value: Box<SExpr>) -> Expr {
    Expr::FieldSet(FieldSetExpr { expr, field, value })
}

pub fn func_def(
    type_params: Option<Vec<TypeParam>>,
    param: Spanned<LetPattern>,
    return_type: Option<STypeExpr>,
    body: Box<SExpr>,
) -> Expr {
    Expr::FuncDef(FuncDefExpr {
        type_params,
        param,
        return_type,
        body,
    })
}

pub fn if_expr(cond: Spanned<Box<SExpr>>, then_expr: Box<SExpr>, else_expr: Box<SExpr>) -> Expr {
    Expr::If(IfExpr {
        cond,
        then_expr,
        else_expr,
    })
}

pub fn instantiate_exist(
    expr: Box<SExpr>,
    types: Spanned<Vec<(StringId, STypeExpr)>>,
    source: InstantiateSourceKind,
) -> Expr {
    Expr::InstantiateExist(InstantiateExistExpr { expr, types, source })
}

pub fn instantiate_uni(expr: Box<SExpr>, types: Spanned<Vec<(StringId, STypeExpr)>>, source: InstantiateSourceKind) -> Expr {
    Expr::InstantiateUni(InstantiateUniExpr { expr, types, source })
}

pub fn literal(lit_type: Literal, value: Spanned<String>) -> Expr {
    Expr::Literal(LiteralExpr { lit_type, value })
}

pub fn loop_expr(body: Box<SExpr>) -> Expr {
    Expr::Loop(LoopExpr { body })
}

pub fn match_expr(expr: Spanned<Box<SExpr>>, cases: Vec<(Spanned<LetPattern>, Box<SExpr>)>) -> Expr {
    Expr::Match(MatchExpr { expr, cases })
}

pub fn record(fields: Vec<KeyPair>) -> Expr {
    Expr::Record(RecordExpr { fields })
}

pub fn typed(expr: Box<SExpr>, type_expr: STypeExpr) -> Expr {
    Expr::Typed(TypedExpr { expr, type_expr })
}

pub fn variable(name: StringId) -> Expr {
    Expr::Variable(VariableExpr { name })
}
