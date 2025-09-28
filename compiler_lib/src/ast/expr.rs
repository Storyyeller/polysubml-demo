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
