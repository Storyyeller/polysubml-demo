#![allow(dead_code)]

use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy)]
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

/////////////////////////////////////////////////////////////////////////////////////////////
pub fn assign(lhs: Expr, rhs: Expr, keep_if_unused: bool) -> Expr {
    Expr(Expr2::Assignment(lhs.0.into(), rhs.0.into(), keep_if_unused))
}
pub fn binop(lhs: Expr, rhs: Expr, op: Op) -> Expr {
    Expr(Expr2::BinOp(lhs.0.into(), rhs.0.into(), op))
}
pub fn call(lhs: Expr, rhs: Expr) -> Expr {
    Expr(Expr2::Call(lhs.0.into(), rhs.0.into()))
}
pub fn unary_minus(rhs: Expr) -> Expr {
    Expr(Expr2::Minus(rhs.0.into()))
}
pub fn void() -> Expr {
    Expr(Expr2::Void)
}
pub fn eqop(lhs: Expr, rhs: Expr) -> Expr {
    Expr(Expr2::BinOp(lhs.0.into(), rhs.0.into(), Op::Eq))
}
pub fn field(lhs: Expr, rhs: String) -> Expr {
    Expr(Expr2::Field(lhs.0.into(), rhs))
}
pub fn scope_field(scope_var: &str, name: &str) -> Expr {
    Expr(Expr2::ScopeField(scope_var.to_string(), name.to_string()))
}
pub fn lit(code: String) -> Expr {
    Expr(Expr2::Literal(code))
}
pub fn ternary(cond: Expr, e1: Expr, e2: Expr) -> Expr {
    Expr(Expr2::Ternary(cond.0.into(), e1.0.into(), e2.0.into()))
}
pub fn var(s: String) -> Expr {
    Expr(Expr2::Var(s))
}

pub fn comma_list(exprs: Vec<Expr>) -> Expr {
    // Flatten any nested Comma expressions
    let mut flattened = Vec::new();
    for expr in exprs {
        match expr.0 {
            Expr2::Comma(nested_exprs) => {
                flattened.extend(nested_exprs);
            }
            _ => {
                flattened.push(expr.0);
            }
        }
    }

    Expr(comma_list_sub(flattened))
}
fn comma_list_sub(flattened: Vec<Expr2>) -> Expr2 {
    match flattened.len() {
        0 => Expr2::Void,
        1 => flattened.into_iter().next().unwrap(),
        _ => Expr2::Comma(flattened),
    }
}

pub fn println(exprs: Vec<Expr>) -> Expr {
    Expr(Expr2::Print(exprs.into_iter().map(|e| e.0).collect()))
}

pub fn func(arg: Expr, scope: String, body: Expr) -> Expr {
    Expr(Expr2::ArrowFunc(Box::new(arg.0), scope, Box::new(body.0)))
}

pub fn obj(fields: Vec<(String, Expr)>) -> Expr {
    let mut prop_defs = Vec::new();
    for (name, v) in fields {
        prop_defs.push(PropertyDefinition::Named(name, v.0.into()));
    }

    Expr(Expr2::Obj(prop_defs))
}

#[derive(Clone, Debug)]
pub struct Expr(Expr2);
impl Expr {
    pub fn to_source(mut self) -> String {
        self.0.add_parens();

        let mut s = "".to_string();
        self.0.write(&mut s);
        s
    }

    pub fn should_inline(&self) -> bool {
        self.0.should_inline()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Token {
    OTHER,
    BRACE,
    PAREN,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Precedence {
    PRIMARY = 0,
    MEMBER,
    CALL,
    LHS,
    UNARY,
    EXPONENT,
    MULTIPLICATIVE,
    ADDITIVE,
    SHIFT,
    RELATIONAL,
    EQUALITY,
    LOR,
    CONDITIONAL,
    ASSIGN,
    EXPR,
}

#[derive(Clone, Debug)]
enum PropertyDefinition {
    Named(String, Box<Expr2>),
}

#[derive(Clone, Debug)]
enum Expr2 {
    Paren(Box<Expr2>),
    Literal(String),
    Obj(Vec<PropertyDefinition>),

    Var(String),

    Field(Box<Expr2>, String),
    ScopeField(String, String),

    Call(Box<Expr2>, Box<Expr2>),

    Minus(Box<Expr2>),
    Void,

    BinOp(Box<Expr2>, Box<Expr2>, Op),

    Ternary(Box<Expr2>, Box<Expr2>, Box<Expr2>),

    // Last parameter indicates whether to keep the assignment even if the variable is unused
    // Used to prevent incorrect optimization of let rec bindings
    Assignment(Box<Expr2>, Box<Expr2>, bool),
    ArrowFunc(Box<Expr2>, String, Box<Expr2>),

    Comma(Vec<Expr2>),

    // Temp hack
    Print(Vec<Expr2>),
}
impl Expr2 {
    fn precedence(&self) -> Precedence {
        use Expr2::*;
        use Op::*;
        use Precedence::*;
        match self {
            Paren(..) => PRIMARY,
            Literal(..) => PRIMARY,
            Obj(..) => PRIMARY,
            Var(..) => PRIMARY,
            Field(..) => MEMBER,
            ScopeField(..) => MEMBER,
            Call(..) => CALL,
            Minus(..) => UNARY,
            Void => UNARY,
            BinOp(_, _, op) => match op {
                Mult | Div | Rem => MULTIPLICATIVE,
                Add | Sub => ADDITIVE,
                Lt | Lte | Gt | Gte => RELATIONAL,
                Eq | Neq => EQUALITY,
            },
            Ternary(..) => CONDITIONAL,
            Assignment(..) => ASSIGN,
            ArrowFunc(..) => ASSIGN,
            Comma(..) => EXPR,
            Print(..) => CALL,
        }
    }

    fn first(&self) -> Token {
        use Expr2::*;
        use Token::*;
        match self {
            Paren(..) => PAREN,
            Literal(..) => OTHER,
            Obj(..) => BRACE,
            Var(..) => OTHER,
            Field(lhs, ..) => lhs.first(),
            ScopeField(..) => OTHER,
            Call(lhs, ..) => lhs.first(),
            Minus(..) => OTHER,
            Void => OTHER,
            BinOp(lhs, ..) => lhs.first(),
            Ternary(lhs, ..) => lhs.first(),
            Assignment(lhs, ..) => lhs.first(),
            ArrowFunc(..) => PAREN,
            Comma(exprs) => exprs.first().map_or(OTHER, |e| e.first()),
            Print(..) => OTHER,
        }
    }

    fn write(&self, out: &mut String) {
        match self {
            Self::Paren(e) => {
                *out += "(";
                e.write(out);
                *out += ")";
            }
            Self::Literal(code) => {
                *out += code;
            }
            Self::Obj(fields) => {
                *out += "{";
                let mut cw = CommaListWrite::new(out);
                for prop_def in fields {
                    use PropertyDefinition::*;
                    match prop_def {
                        Named(name, val) => cw.write(|out| {
                            *out += "'";
                            *out += name;
                            *out += "': ";
                            val.write(out);
                        }),
                    }
                }
                *out += "}";
            }
            Self::Var(name) => {
                *out += name;
            }
            Self::Field(lhs, rhs) => {
                lhs.write(out);
                *out += ".";
                *out += rhs;
            }
            Self::ScopeField(s1, s2) => {
                *out += s1;
                *out += ".";
                *out += s2;
            }
            Self::Call(lhs, rhs) => {
                lhs.write(out);
                *out += "(";
                rhs.write(out);
                *out += ")";
            }
            Self::Minus(e) => {
                *out += "-";
                e.write(out);
            }
            Self::Void => {
                *out += "void 0";
            }
            Self::BinOp(lhs, rhs, op) => {
                use Op::*;
                let opstr = match op {
                    Add => "+",
                    Sub => "- ",
                    Mult => "*",
                    Div => "/",
                    Rem => "%",

                    Lt => "<",
                    Lte => "<=",
                    Gt => ">",
                    Gte => ">=",

                    Eq => "===",
                    Neq => "!==",
                };

                lhs.write(out);
                *out += opstr;
                rhs.write(out);
            }
            Self::Ternary(cond, e1, e2) => {
                cond.write(out);
                *out += " ? ";
                e1.write(out);
                *out += " : ";
                e2.write(out);
            }
            Self::Assignment(lhs, rhs, _) => {
                lhs.write(out);
                *out += " = ";
                rhs.write(out);
            }
            Self::ArrowFunc(arg, scope_arg, body) => {
                *out += "(";
                arg.write(out);
                *out += ", ";
                *out += scope_arg;
                *out += "={}) => ";
                body.write(out);
            }
            Self::Comma(exprs) => {
                let mut cw = CommaListWrite::new(out);
                for ex in exprs.iter() {
                    cw.write(|out| ex.write(out));
                }
            }
            Self::Print(exprs) => {
                *out += "p.println(";
                let mut cw = CommaListWrite::new(out);
                for ex in exprs {
                    cw.write(|out| ex.write(out));
                }
                *out += ")";
            }
        }
    }

    fn wrap_in_parens(&mut self) {
        use Expr2::*;
        let dummy = Literal("".to_string());
        let temp = std::mem::replace(self, dummy);
        *self = Paren(Box::new(temp));
    }

    /// Ensure that this expression has at most the given precedence. If it has lower precedence,
    /// wrap it in parentheses.
    fn ensure(&mut self, required: Precedence) {
        if self.precedence() > required {
            self.wrap_in_parens();
        }
    }

    fn add_parens(&mut self) {
        use Precedence::*;
        match self {
            Self::Paren(e) => {
                e.add_parens();
            }
            Self::Literal(code) => {}
            Self::Obj(fields) => {
                for prop_def in fields {
                    use PropertyDefinition::*;
                    match prop_def {
                        Named(name, val) => {
                            val.add_parens();
                            val.ensure(ASSIGN);
                        }
                    }
                }
            }
            Self::Var(name) => {}
            Self::Field(lhs, rhs) => {
                lhs.add_parens();
                lhs.ensure(MEMBER);
            }
            Self::ScopeField(..) => {}
            Self::Call(lhs, rhs) => {
                lhs.add_parens();
                lhs.ensure(MEMBER);
                rhs.add_parens();
                rhs.ensure(ASSIGN);
            }
            Self::Minus(e) => {
                e.add_parens();
                e.ensure(UNARY);
            }
            Self::Void => {}
            Self::BinOp(lhs, rhs, op) => {
                use Op::*;
                let req = match op {
                    Mult | Div | Rem => (MULTIPLICATIVE, EXPONENT),
                    Add | Sub => (ADDITIVE, MULTIPLICATIVE),
                    Lt | Lte | Gt | Gte => (RELATIONAL, SHIFT),
                    Eq | Neq => (EQUALITY, RELATIONAL),
                };

                lhs.add_parens();
                lhs.ensure(req.0);
                rhs.add_parens();
                rhs.ensure(req.1);
            }
            Self::Ternary(cond, e1, e2) => {
                cond.add_parens();
                e1.add_parens();
                e1.ensure(ASSIGN);
                e2.add_parens();
                e2.ensure(ASSIGN);
            }
            Self::Assignment(lhs, rhs, _) => {
                lhs.add_parens();
                lhs.ensure(LHS);
                rhs.add_parens();
                rhs.ensure(ASSIGN);
            }
            Self::ArrowFunc(arg, scope_arg, body) => {
                arg.add_parens();
                body.add_parens();
                body.ensure(ASSIGN);
                // body can't be an expression starting with "{"
                if body.first() == Token::BRACE {
                    body.wrap_in_parens();
                }
            }
            Self::Comma(exprs) => {
                for expr in exprs.iter_mut() {
                    expr.add_parens();
                }
                // All expressions except the first must have ASSIGN precedence
                for expr in exprs.iter_mut().skip(1) {
                    expr.ensure(ASSIGN);
                }
            }
            Self::Print(exprs) => {
                for ex in exprs {
                    ex.add_parens();
                    ex.ensure(PRIMARY);
                }
            }
        }
    }

    // Used by codegen for inlining decisions, not related to AST printing
    fn should_inline(&self) -> bool {
        use Expr2::*;
        match &self {
            Literal(s) => s.len() <= 10,
            Minus(e) => e.should_inline(),
            ScopeField(..) => true,
            Var(..) => true,
            _ => false,
        }
    }
}

/// Helper for writing comma-separated lists
struct CommaListWrite<'a> {
    out: &'a mut String,
    first: bool,
}
impl<'a> CommaListWrite<'a> {
    fn new(out: &'a mut String) -> Self {
        Self { out, first: true }
    }

    fn write(&mut self, f: impl FnOnce(&mut String)) {
        if self.first {
            self.first = false;
        } else {
            *self.out += ", ";
        }
        f(self.out);
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////
// Dead code elimination: removes unused variable assignments by tracking which scope variables
// are actually read. Works backwards through expressions to determine usage.
struct DeadCodeRemover {
    // For each scope in the current stack, the set of variables that are used so far
    used: HashMap<String, HashSet<String>>,
}
impl DeadCodeRemover {
    fn new() -> Self {
        Self { used: HashMap::new() }
    }

    fn add_var(&mut self, scope: &str, name: String) {
        self.used.get_mut(scope).unwrap().insert(name);
    }

    // Replace "s.x = rhs" with just "rhs" if s.x is never read
    fn remove_var_assign_if_unused(&self, expr: &mut Expr2) {
        use Expr2::*;
        if let Assignment(lhs, rhs, keep_if_unused) = expr {
            if !*keep_if_unused {
                if let ScopeField(ref s1, ref s2) = **lhs {
                    if !self.used.get(s1).unwrap().contains(s2) {
                        let rhs: Expr2 = std::mem::replace(&mut *rhs, Void);
                        *expr = rhs;
                    }
                }
            }
        }
    }

    // Process an expression whose value is used - traverses backwards, marking variables as used
    fn process_used_expr(&mut self, expr: &mut Expr2) {
        self.remove_var_assign_if_unused(expr);

        use Expr2::*;
        match expr {
            Paren(e) => {
                self.process_used_expr(e);
            }
            Literal(_) => {}
            Obj(fields) => {
                for prop_def in fields.iter_mut().rev() {
                    use PropertyDefinition::*;
                    match prop_def {
                        Named(_, val) => {
                            self.process_used_expr(val);
                        }
                    }
                }
            }
            Var(_) => {}
            Field(lhs, _) => {
                self.process_used_expr(lhs);
            }
            ScopeField(s1, s2) => {
                self.add_var(s1, s2.clone());
            }
            Call(lhs, rhs) => {
                self.process_used_expr(rhs);
                self.process_used_expr(lhs);
            }
            Minus(e) => {
                self.process_used_expr(e);
            }
            Void => {}
            BinOp(lhs, rhs, _) => {
                self.process_used_expr(rhs);
                self.process_used_expr(lhs);
            }
            Ternary(cond, e1, e2) => {
                self.process_used_expr(e2);
                self.process_used_expr(e1);
                self.process_used_expr(cond);
            }
            Assignment(lhs, rhs, _) => {
                self.process_used_expr(rhs);
                self.process_used_expr(lhs);
            }
            ArrowFunc(_, scope, body) => {
                self.used.insert(scope.clone(), HashSet::new());
                self.process_used_expr(body);
                self.used.remove(scope);
            }
            Comma(exprs) => {
                // In comma expressions, only the last value is used
                let mut last = exprs.pop().unwrap();
                self.process_used_expr(&mut last);
                let mut out = vec![last];

                while let Some(ex) = exprs.pop() {
                    self.process_unused_expr(ex, &mut out);
                }

                out.reverse();
                *expr = comma_list_sub(out);
            }
            Print(exprs) => {
                for ex in exprs.iter_mut().rev() {
                    self.process_used_expr(ex);
                }
            }
        }
    }

    // Process an expression whose value is discarded - only keeps side effects
    fn process_unused_expr(&mut self, mut expr: Expr2, out: &mut Vec<Expr2>) {
        self.remove_var_assign_if_unused(&mut expr);

        use Expr2::*;
        match expr {
            Paren(e) => {
                self.process_unused_expr(*e, out);
            }
            Literal(_) => {}
            Obj(fields) => {
                for prop_def in fields.into_iter().rev() {
                    use PropertyDefinition::*;
                    match prop_def {
                        Named(_, val) => {
                            self.process_unused_expr(*val, out);
                        }
                    }
                }
            }
            Var(_) => {}
            Field(lhs, _) => {
                self.process_unused_expr(*lhs, out);
            }
            ScopeField(..) => {}
            Minus(e) => {
                self.process_unused_expr(*e, out);
            }
            Void => {}
            BinOp(lhs, rhs, _) => {
                self.process_unused_expr(*rhs, out);
                self.process_unused_expr(*lhs, out);
            }
            ArrowFunc(..) => {}
            Comma(exprs) => {
                for ex in exprs.into_iter().rev() {
                    self.process_unused_expr(ex, out);
                }
            }
            Call(..) | Ternary(..) | Assignment(..) | Print(..) => {
                // These expression types inherently have side effects, so we keep them as is.
                self.process_used_expr(&mut expr);
                out.push(expr);
            }
        }
    }
}

pub fn optimize(expr: &mut Expr, main_scope_name: String, bindings: &HashMap<crate::ast::StringId, Expr>) {
    let mut optimizer = DeadCodeRemover::new();
    optimizer.used.insert(main_scope_name, HashSet::new());
    for expr in bindings.values() {
        if let Expr2::ScopeField(s1, s2) = &expr.0 {
            optimizer.add_var(s1, s2.clone());
        }
    }
    optimizer.process_used_expr(&mut expr.0);
}
