use crate::ast::LetPattern;
use crate::ast::Literal;
use crate::ast::Op;
use crate::ast::OpType;
use crate::ast::STypeExpr;
use crate::ast::Statement;
use crate::ast::StringId;
use crate::ast::TypeParam;
use crate::spans::Span;
use crate::spans::Spanned;

pub type KeyPair = (Spanned<StringId>, Box<Expr>, bool, Option<STypeExpr>);

#[derive(Debug, Clone, Copy)]
pub enum InstantiateSourceKind {
    ImplicitCall,
    ImplicitRecord,
    ExplicitParams(bool),
}

#[derive(Debug, Clone)]
pub enum Expr {
    BinOp(Box<Expr>, Span, Box<Expr>, Span, OpType, Op, Span),
    Block(Vec<Statement>, Box<Expr>),
    Call(Box<Expr>, Box<Expr>, Span),
    Case(Spanned<StringId>, Box<Expr>),
    FieldAccess(Box<Expr>, Spanned<StringId>, Span),
    FieldSet(Box<Expr>, Spanned<StringId>, Box<Expr>, Span),
    FuncDef(Spanned<(Option<Vec<TypeParam>>, Spanned<LetPattern>, Option<STypeExpr>, Box<Expr>)>),
    If(Spanned<Box<Expr>>, Box<Expr>, Box<Expr>),
    InstantiateExist(Box<Expr>, Spanned<Vec<(StringId, STypeExpr)>>, InstantiateSourceKind, Span),
    InstantiateUni(
        Spanned<Box<Expr>>,
        Spanned<Vec<(StringId, STypeExpr)>>,
        InstantiateSourceKind,
        Span,
    ),
    Literal(Literal, Spanned<String>),
    Loop(Box<Expr>, Span),
    Match(Spanned<Box<Expr>>, Vec<(Spanned<LetPattern>, Box<Expr>)>, Span),
    Record(Vec<KeyPair>, Span),
    Typed(Box<Expr>, STypeExpr),
    Variable(Spanned<StringId>),
}

// Constructor functions for Expr variants
pub fn binop(
    lhs: Box<Expr>,
    lhs_span: Span,
    rhs: Box<Expr>,
    rhs_span: Span,
    op_type: OpType,
    op: Op,
    op_span: Span,
) -> Expr {
    Expr::BinOp(lhs, lhs_span, rhs, rhs_span, op_type, op, op_span)
}

pub fn block(statements: Vec<Statement>, expr: Box<Expr>) -> Expr {
    Expr::Block(statements, expr)
}

pub fn call(func: Box<Expr>, arg: Box<Expr>, span: Span) -> Expr {
    Expr::Call(func, arg, span)
}

pub fn case(tag: Spanned<StringId>, expr: Box<Expr>) -> Expr {
    Expr::Case(tag, expr)
}

pub fn field_access(expr: Box<Expr>, field: Spanned<StringId>, span: Span) -> Expr {
    Expr::FieldAccess(expr, field, span)
}

pub fn field_set(expr: Box<Expr>, field: Spanned<StringId>, value: Box<Expr>, span: Span) -> Expr {
    Expr::FieldSet(expr, field, value, span)
}

pub fn func_def(def: Spanned<(Option<Vec<TypeParam>>, Spanned<LetPattern>, Option<STypeExpr>, Box<Expr>)>) -> Expr {
    Expr::FuncDef(def)
}

pub fn if_expr(cond: Spanned<Box<Expr>>, then_expr: Box<Expr>, else_expr: Box<Expr>) -> Expr {
    Expr::If(cond, then_expr, else_expr)
}

pub fn instantiate_exist(
    expr: Box<Expr>,
    types: Spanned<Vec<(StringId, STypeExpr)>>,
    source: InstantiateSourceKind,
    span: Span,
) -> Expr {
    Expr::InstantiateExist(expr, types, source, span)
}

pub fn instantiate_uni(
    expr: Spanned<Box<Expr>>,
    types: Spanned<Vec<(StringId, STypeExpr)>>,
    source: InstantiateSourceKind,
    span: Span,
) -> Expr {
    Expr::InstantiateUni(expr, types, source, span)
}

pub fn literal(lit_type: Literal, value: Spanned<String>) -> Expr {
    Expr::Literal(lit_type, value)
}

pub fn loop_expr(body: Box<Expr>, span: Span) -> Expr {
    Expr::Loop(body, span)
}

pub fn match_expr(expr: Spanned<Box<Expr>>, cases: Vec<(Spanned<LetPattern>, Box<Expr>)>, span: Span) -> Expr {
    Expr::Match(expr, cases, span)
}

pub fn record(fields: Vec<KeyPair>, span: Span) -> Expr {
    Expr::Record(fields, span)
}

pub fn typed(expr: Box<Expr>, type_expr: STypeExpr) -> Expr {
    Expr::Typed(expr, type_expr)
}

pub fn variable(name: Spanned<StringId>) -> Expr {
    Expr::Variable(name)
}
