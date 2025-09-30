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
    pub lhs_span: Span,
    pub rhs: Box<SExpr>,
    pub rhs_span: Span,
    pub op_type: OpType,
    pub op: Op,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct BlockExpr {
    pub statements: Vec<Statement>,
    pub expr: Box<SExpr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct CallExpr {
    pub func: Box<SExpr>,
    pub arg: Box<SExpr>,
    pub span: Span,
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
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct FieldSetExpr {
    pub expr: Box<SExpr>,
    pub field: Spanned<StringId>,
    pub value: Box<SExpr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct FuncDefExpr {
    pub def: Spanned<(Option<Vec<TypeParam>>, Spanned<LetPattern>, Option<STypeExpr>, Box<SExpr>)>,
}

#[derive(Debug, Clone)]
pub struct IfExpr {
    pub cond: Spanned<Box<SExpr>>,
    pub then_expr: Box<SExpr>,
    pub else_expr: Box<SExpr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct InstantiateExistExpr {
    pub expr: Box<SExpr>,
    pub types: Spanned<Vec<(StringId, STypeExpr)>>,
    pub source: InstantiateSourceKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct InstantiateUniExpr {
    pub expr: Spanned<Box<SExpr>>,
    pub types: Spanned<Vec<(StringId, STypeExpr)>>,
    pub source: InstantiateSourceKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct LiteralExpr {
    pub lit_type: Literal,
    pub value: Spanned<String>,
}

#[derive(Debug, Clone)]
pub struct LoopExpr {
    pub body: Box<SExpr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct MatchExpr {
    pub expr: Spanned<Box<SExpr>>,
    pub cases: Vec<(Spanned<LetPattern>, Box<SExpr>)>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct RecordExpr {
    pub fields: Vec<KeyPair>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct TypedExpr {
    pub expr: Box<SExpr>,
    pub type_expr: STypeExpr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct VariableExpr {
    pub name: Spanned<StringId>,
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
impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::BinOp(e) => e.span,
            Expr::Block(e) => e.span,
            Expr::Call(e) => e.span,
            Expr::Case(e) => e.tag.1,
            Expr::FieldAccess(e) => e.span,
            Expr::FieldSet(e) => e.span,
            Expr::FuncDef(e) => e.def.1,
            Expr::If(e) => e.span,
            Expr::InstantiateExist(e) => e.span,
            Expr::InstantiateUni(e) => e.span,
            Expr::Literal(e) => e.value.1,
            Expr::Loop(e) => e.span,
            Expr::Match(e) => e.span,
            Expr::Record(e) => e.span,
            Expr::Typed(e) => e.span,
            Expr::Variable(e) => e.name.1,
        }
    }
}
pub type SExpr = Expr;

// Constructor functions for Expr variants
pub fn binop(
    lhs: Box<SExpr>,
    lhs_span: Span,
    rhs: Box<SExpr>,
    rhs_span: Span,
    op_type: OpType,
    op: Op,
    op_span: Span,
) -> Expr {
    Expr::BinOp(BinOpExpr {
        lhs,
        lhs_span,
        rhs,
        rhs_span,
        op_type,
        op,
        span: op_span,
    })
}

pub fn block(statements: Vec<Statement>, expr: Box<SExpr>, span: Span) -> Expr {
    Expr::Block(BlockExpr { statements, expr, span })
}

pub fn call(func: Box<SExpr>, arg: Box<SExpr>, span: Span) -> Expr {
    Expr::Call(CallExpr { func, arg, span })
}

pub fn case(tag: Spanned<StringId>, expr: Box<SExpr>) -> Expr {
    Expr::Case(CaseExpr { tag, expr })
}

pub fn field_access(expr: Box<SExpr>, field: Spanned<StringId>, span: Span) -> Expr {
    Expr::FieldAccess(FieldAccessExpr { expr, field, span })
}

pub fn field_set(expr: Box<SExpr>, field: Spanned<StringId>, value: Box<SExpr>, span: Span) -> Expr {
    Expr::FieldSet(FieldSetExpr {
        expr,
        field,
        value,
        span,
    })
}

pub fn func_def(def: Spanned<(Option<Vec<TypeParam>>, Spanned<LetPattern>, Option<STypeExpr>, Box<SExpr>)>) -> Expr {
    Expr::FuncDef(FuncDefExpr { def })
}

pub fn if_expr(cond: Spanned<Box<SExpr>>, then_expr: Box<SExpr>, else_expr: Box<SExpr>, span: Span) -> Expr {
    Expr::If(IfExpr {
        cond,
        then_expr,
        else_expr,
        span,
    })
}

pub fn instantiate_exist(
    expr: Box<SExpr>,
    types: Spanned<Vec<(StringId, STypeExpr)>>,
    source: InstantiateSourceKind,
    span: Span,
) -> Expr {
    Expr::InstantiateExist(InstantiateExistExpr {
        expr,
        types,
        source,
        span,
    })
}

pub fn instantiate_uni(
    expr: Spanned<Box<SExpr>>,
    types: Spanned<Vec<(StringId, STypeExpr)>>,
    source: InstantiateSourceKind,
    span: Span,
) -> Expr {
    Expr::InstantiateUni(InstantiateUniExpr {
        expr,
        types,
        source,
        span,
    })
}

pub fn literal(lit_type: Literal, value: Spanned<String>) -> Expr {
    Expr::Literal(LiteralExpr { lit_type, value })
}

pub fn loop_expr(body: Box<SExpr>, span: Span) -> Expr {
    Expr::Loop(LoopExpr { body, span })
}

pub fn match_expr(expr: Spanned<Box<SExpr>>, cases: Vec<(Spanned<LetPattern>, Box<SExpr>)>, span: Span) -> Expr {
    Expr::Match(MatchExpr { expr, cases, span })
}

pub fn record(fields: Vec<KeyPair>, span: Span) -> Expr {
    Expr::Record(RecordExpr { fields, span })
}

pub fn typed(expr: Box<SExpr>, type_expr: STypeExpr, span: Span) -> Expr {
    Expr::Typed(TypedExpr { expr, type_expr, span })
}

pub fn variable(name: Spanned<StringId>) -> Expr {
    Expr::Variable(VariableExpr { name })
}
