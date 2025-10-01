pub mod expr;
pub use expr::Expr;
pub use expr::InstantiateSourceKind;
pub use expr::SExpr;

use crate::spans::Span;
use crate::spans::SpanMaker;
use crate::spans::Spanned;

pub struct ParserContext<'a, 'input> {
    pub span_maker: SpanMaker<'input>,
    pub strings: &'a mut lasso::Rodeo,
}
pub type StringId = lasso::Spur;

#[derive(Debug, Clone)]
pub enum Literal {
    Bool,
    Float,
    Int,
    Str,
}

#[derive(Debug, Clone)]
pub enum Op {
    Add,
    Sub,
    Mult,
    Div,
    Rem,

    Lt,
    Lte,
    Gt,
    Gte,

    Eq,
    Neq,
}

pub type OpType = (Option<Literal>, Literal);
pub const INT_OP: OpType = (Some(Literal::Int), Literal::Int);
pub const FLOAT_OP: OpType = (Some(Literal::Float), Literal::Float);
pub const STR_OP: OpType = (Some(Literal::Str), Literal::Str);
pub const INT_CMP: OpType = (Some(Literal::Int), Literal::Bool);
pub const FLOAT_CMP: OpType = (Some(Literal::Float), Literal::Bool);
pub const ANY_CMP: OpType = (None, Literal::Bool);

type LetDefinition = (LetPattern, Box<SExpr>);
pub type LetRecDefinition = (StringId, SExpr);

#[derive(Debug, Clone)]
pub enum LetPattern {
    Case(Spanned<StringId>, Box<LetPattern>),
    Record(Spanned<(Vec<TypeParam>, Vec<(Spanned<StringId>, Box<LetPattern>)>)>),
    Var((Option<StringId>, Span), Option<STypeExpr>),
}

#[derive(Debug, Clone, Copy)]
pub struct TypeParam {
    pub name: Spanned<StringId>,
    pub alias: Spanned<StringId>,
}
impl TypeParam {
    pub fn new(name: Spanned<StringId>, alias: Option<Spanned<StringId>>) -> Self {
        let alias = alias.unwrap_or(name);
        Self { name, alias }
    }
}

#[derive(Debug, Clone)]
pub enum FieldTypeDecl {
    Imm(STypeExpr),
    RWSame(STypeExpr),
    RWPair(STypeExpr, STypeExpr),
}
pub type KeyPairType = (Spanned<StringId>, FieldTypeDecl);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolyKind {
    Universal,
    Existential,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum JoinKind {
    Union,
    Intersect,
}

#[derive(Debug, Clone)]
pub enum TypeExpr {
    Bot,
    Case(Vec<(Spanned<StringId>, Box<STypeExpr>)>),
    Func(Box<STypeExpr>, Box<STypeExpr>),
    Hole,
    Ident(StringId),
    Poly(Vec<TypeParam>, Box<STypeExpr>, PolyKind),
    Record(Vec<KeyPairType>),
    RecursiveDef(StringId, Box<STypeExpr>),
    Top,
    VarJoin(JoinKind, Vec<STypeExpr>),
}
pub type STypeExpr = Spanned<TypeExpr>;

#[derive(Debug, Clone)]
pub enum Statement {
    Empty,
    Expr(SExpr),
    LetDef(LetDefinition),
    LetRecDef(Vec<LetRecDefinition>),
    Println(Vec<SExpr>),
}

fn enumerate_tuple_fields<T, R>(
    vals: impl IntoIterator<Item = (T, Span)>,
    strings: &mut lasso::Rodeo,
    mut make_field: impl FnMut(Spanned<StringId>, T) -> R,
) -> Vec<R> {
    vals.into_iter()
        .enumerate()
        .map(|(i, (val, span))| {
            let name = strings.get_or_intern(&format!("_{}", i));
            make_field((name, span), val)
        })
        .collect()
}

// TODO, cleanup
pub fn make_tuple_expr(mut vals: Vec<SExpr>, strings: &mut lasso::Rodeo) -> Expr {
    if vals.len() <= 1 {
        return vals.pop().unwrap().0;
    }

    // Tuple
    let fields = enumerate_tuple_fields(vals, strings, |name, val| (name, Box::new((val, name.1)), false, None));
    expr::record(fields)
}

pub fn make_tuple_pattern(vals: Spanned<Vec<Spanned<LetPattern>>>, strings: &mut lasso::Rodeo) -> LetPattern {
    let (mut vals, full_span) = vals;
    if vals.len() <= 1 {
        return vals.pop().unwrap().0;
    }

    let fields = enumerate_tuple_fields(vals, strings, |name, val| (name, Box::new(val)));
    LetPattern::Record(((vec![], fields), full_span))
}

pub fn make_tuple_type(mut vals: Vec<STypeExpr>, strings: &mut lasso::Rodeo) -> TypeExpr {
    if vals.len() <= 1 {
        return vals.pop().unwrap().0;
    }

    let fields = enumerate_tuple_fields(vals, strings, |name, val| (name, FieldTypeDecl::Imm((val, name.1))));
    TypeExpr::Record(fields)
}

pub fn make_join_ast(kind: JoinKind, mut children: Vec<STypeExpr>) -> TypeExpr {
    if children.len() <= 1 {
        children.pop().unwrap().0
    } else {
        TypeExpr::VarJoin(kind, children)
    }
}
