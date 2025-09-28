use std::mem::swap;

use crate::ast;
use crate::ast::StringId;
use crate::js;
use crate::unwindmap::UnwindMap;

pub struct ModuleBuilder {
    scope_expr: js::Expr,
    scope_counter: u64,
    param_counter: u64,
    // For choosing new var names
    var_counter: u64,
    // ML name -> JS expr for current scope
    bindings: UnwindMap<StringId, js::Expr>,
}
impl ModuleBuilder {
    pub fn new() -> Self {
        Self {
            scope_expr: js::var("$".to_string(), true),
            scope_counter: 0,
            param_counter: 0,
            var_counter: 0,
            bindings: UnwindMap::new(),
        }
    }

    fn set_binding(&mut self, k: StringId, v: js::Expr) {
        self.bindings.insert(k, v);
    }

    fn new_var_name(&mut self) -> String {
        let js_name = format!("v{}", self.var_counter);
        self.var_counter += 1;
        js_name
    }

    fn new_temp_var_assign(&mut self, rhs: js::Expr, out: &mut Vec<js::Expr>) -> js::Expr {
        if rhs.should_inline() {
            return rhs;
        }

        let js_name = format!("t{}", self.var_counter);
        self.var_counter += 1;

        let expr = js::field(self.scope_expr.clone(), js_name);
        out.push(js::assign(expr.clone(), rhs));
        expr
    }

    fn new_var(&mut self, ml_name: StringId) -> js::Expr {
        let js_name = self.new_var_name();
        let expr = js::field(self.scope_expr.clone(), js_name);
        self.set_binding(ml_name, expr.clone());
        expr
    }

    fn new_var_assign(&mut self, ml_name: StringId, rhs: js::Expr, out: &mut Vec<js::Expr>) -> js::Expr {
        if rhs.should_inline() {
            self.set_binding(ml_name, rhs.clone());
            return rhs;
        }

        let expr = self.new_var(ml_name);
        out.push(js::assign(expr.clone(), rhs));
        expr
    }

    fn new_scope_name(&mut self) -> String {
        let js_name = format!("s{}", self.scope_counter);
        self.scope_counter += 1;
        js_name
    }

    fn new_param_name(&mut self) -> String {
        let js_name = format!("p{}", self.param_counter);
        self.param_counter += 1;
        js_name
    }
}
pub struct Context<'a>(pub &'a mut ModuleBuilder, pub &'a lasso::Rodeo);
impl<'a> Context<'a> {
    fn ml_scope<T>(&mut self, cb: impl FnOnce(&mut Self) -> T) -> T {
        let n = self.bindings.unwind_point();
        let res = cb(self);
        self.bindings.unwind(n);
        res
    }

    fn fn_scope<T>(&mut self, cb: impl FnOnce(&mut Self) -> T) -> T {
        let old_var_counter = self.var_counter;
        let old_param_counter = self.param_counter;
        let old_scope_counter = self.scope_counter;
        self.var_counter = 0;

        let res = self.ml_scope(cb);

        self.var_counter = old_var_counter;
        self.param_counter = old_param_counter;
        self.scope_counter = old_scope_counter;
        res
    }

    fn get(&self, id: StringId) -> &'a str {
        self.1.resolve(&id)
    }

    fn get_new(&self, id: StringId) -> String {
        self.1.resolve(&id).to_owned()
    }
}
impl<'a> core::ops::Deref for Context<'a> {
    type Target = ModuleBuilder;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<'a> core::ops::DerefMut for Context<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

fn compile(ctx: &mut Context<'_>, expr: &ast::Expr) -> js::Expr {
    match expr {
        ast::Expr::BinOp(lhs_expr, lhs_span, rhs_expr, rhs_span, op_type, op, full_span) => {
            let lhs = compile(ctx, lhs_expr);
            let rhs = compile(ctx, rhs_expr);
            let jsop = match op {
                ast::Op::Add => js::Op::Add,
                ast::Op::Sub => js::Op::Sub,
                ast::Op::Mult => js::Op::Mult,
                ast::Op::Div => js::Op::Div,
                ast::Op::Rem => js::Op::Rem,

                ast::Op::Lt => js::Op::Lt,
                ast::Op::Lte => js::Op::Lte,
                ast::Op::Gt => js::Op::Gt,
                ast::Op::Gte => js::Op::Gte,

                ast::Op::Eq => js::Op::Eq,
                ast::Op::Neq => js::Op::Neq,
            };
            js::binop(lhs, rhs, jsop)
        }
        ast::Expr::Block(statements, rest_expr) => {
            ctx.ml_scope(|ctx| {
                let mut exprs = Vec::new(); // a list of assignments, followed by rest

                for stmt in statements {
                    compile_statement(ctx, &mut exprs, stmt);
                }

                exprs.push(compile(ctx, rest_expr));
                js::comma_list(exprs)
            })
        }
        ast::Expr::Call(func, arg, _) => {
            let lhs = compile(ctx, func);
            let rhs = compile(ctx, arg);
            js::call(lhs, rhs)
        }
        ast::Expr::Case((tag, _), expr) => {
            let tag = js::lit(format!("\"{}\"", ctx.get(*tag)));
            let expr = compile(ctx, expr);
            js::obj(vec![("$tag".to_string(), tag), ("$val".to_string(), expr)])
        }
        ast::Expr::FieldAccess(lhs_expr, (name, _), _) => {
            let lhs = compile(ctx, lhs_expr);
            js::field(lhs, ctx.get_new(*name))
        }
        ast::Expr::FieldSet(lhs_expr, (name, _), rhs_expr, _) => {
            let mut exprs = Vec::new();

            let lhs_compiled = compile(ctx, lhs_expr);
            let lhs_temp_var = ctx.new_temp_var_assign(lhs_compiled, &mut exprs);
            let lhs = js::field(lhs_temp_var, ctx.get_new(*name));

            let res_temp_var = ctx.new_temp_var_assign(lhs.clone(), &mut exprs);
            exprs.push(js::assign(lhs.clone(), compile(ctx, rhs_expr)));
            exprs.push(res_temp_var);

            js::comma_list(exprs)
        }
        ast::Expr::FuncDef(((_, (arg_pattern, _), _, body_expr), _)) => {
            ctx.fn_scope(|ctx| {
                let new_scope_name = ctx.new_scope_name();
                let mut scope_expr = js::var(new_scope_name.clone(), true);
                swap(&mut scope_expr, &mut ctx.scope_expr);

                //////////////////////////////////////////////////////
                let js_pattern = compile_let_pattern(ctx, arg_pattern).unwrap_or_else(|| js::var("_".to_string(), true));
                let body = compile(ctx, body_expr);
                //////////////////////////////////////////////////////

                swap(&mut scope_expr, &mut ctx.scope_expr);
                js::func(js_pattern, new_scope_name, body)
            })
        }
        ast::Expr::If((cond_expr, _), then_expr, else_expr) => {
            let cond_expr = compile(ctx, cond_expr);
            let then_expr = compile(ctx, then_expr);
            let else_expr = compile(ctx, else_expr);
            js::ternary(cond_expr, then_expr, else_expr)
        }
        ast::Expr::InstantiateExist(expr, _, _, _) => compile(ctx, expr),
        ast::Expr::InstantiateUni((expr, _), _, _, _) => compile(ctx, expr),
        ast::Expr::Literal(type_, (code, _)) => {
            let mut code = code.clone();
            if let ast::Literal::Int = type_ {
                code.push_str("n");
            }
            if code.starts_with("-") {
                js::unary_minus(js::lit(code[1..].to_string()))
            } else {
                js::lit(code)
            }
        }
        ast::Expr::Loop(body, _) => {
            let lhs = js::var("loop".to_string(), false);
            let rhs = compile(ctx, body);
            let rhs = js::func(js::var("_".to_string(), true), "_2".to_string(), rhs);
            js::call(lhs, rhs)
        }
        ast::Expr::Match((match_expr, _), cases, _) => {
            let mut exprs = Vec::new();
            let match_compiled = compile(ctx, match_expr);
            let temp_var = ctx.new_temp_var_assign(match_compiled, &mut exprs);

            let tag_expr = js::field(temp_var.clone(), "$tag".to_string());
            let val_expr = js::field(temp_var.clone(), "$val".to_string());

            let mut branches = Vec::new();
            let mut wildcard = None;
            for ((pattern, _), rhs_expr) in cases {
                use ast::LetPattern::*;
                match pattern {
                    Case((tag, _), sub_pattern) => {
                        ctx.ml_scope(|ctx| {
                            let mut exprs = Vec::new();
                            compile_let_pattern_flat(ctx, &mut exprs, sub_pattern, val_expr.clone());
                            exprs.push(compile(ctx, rhs_expr));
                            branches.push((ctx.get(*tag), js::comma_list(exprs)));
                        });
                    }
                    _ => {
                        wildcard = Some(ctx.ml_scope(|ctx| {
                            let mut exprs = Vec::new();
                            compile_let_pattern_flat(ctx, &mut exprs, pattern, temp_var.clone());
                            exprs.push(compile(ctx, rhs_expr));
                            js::comma_list(exprs)
                        }));
                    }
                }
            }

            let mut res = wildcard.unwrap_or_else(|| branches.pop().unwrap().1);
            while let Some((tag, rhs_expr)) = branches.pop() {
                assert!(tag.len() > 0);
                let cond = js::eqop(tag_expr.clone(), js::lit(format!("\"{}\"", tag)));
                res = js::ternary(cond, rhs_expr, res);
            }

            exprs.push(res);
            js::comma_list(exprs)
        }
        ast::Expr::Record(fields, span) => js::obj(
            fields
                .iter()
                .map(|((name, _), expr, _, _)| (ctx.get_new(*name), compile(ctx, expr)))
                .collect(),
        ),
        ast::Expr::Typed(expr, _) => compile(ctx, expr),
        ast::Expr::Variable((name, _)) => ctx.bindings.get(name).unwrap().clone(),
    }
}

fn compile_let_pattern_flat(ctx: &mut Context<'_>, out: &mut Vec<js::Expr>, pat: &ast::LetPattern, rhs: js::Expr) {
    use ast::LetPattern::*;
    match pat {
        Case(_, val_pat) => {
            // rhs.$val
            let rhs = js::field(rhs, "$val".to_string());
            compile_let_pattern_flat(ctx, out, val_pat, rhs);
        }
        Record(((_, pairs), _)) => {
            // Assign the rhs to a temporary value, and then do a = temp.foo for each field
            let lhs = ctx.new_temp_var_assign(rhs, out);

            for ((name, _), pat) in pairs.iter() {
                compile_let_pattern_flat(ctx, out, pat, js::field(lhs.clone(), ctx.get_new(*name)));
            }
        }

        Var((ml_name, _), _) => {
            if let Some(ml_name) = ml_name {
                ctx.new_var_assign(*ml_name, rhs, out);
            }
        }
    }
}

fn compile_let_pattern(ctx: &mut Context<'_>, pat: &ast::LetPattern) -> Option<js::Expr> {
    use ast::LetPattern::*;
    Some(match pat {
        Case(_, val_pat) => js::obj(vec![("$val".to_string(), compile_let_pattern(ctx, &*val_pat)?)]),
        Record(((_, pairs), _)) => js::obj(
            pairs
                .iter()
                .filter_map(|((name, _), pat)| Some((ctx.get_new(*name), compile_let_pattern(ctx, &*pat)?)))
                .collect(),
        ),

        Var((ml_name, _), _) => {
            let js_arg = js::var(ctx.new_param_name(), false);
            let ml_name = ml_name.as_ref()?;
            ctx.set_binding(*ml_name, js_arg.clone());
            js_arg
        }
    })
}

fn compile_statement(ctx: &mut Context<'_>, exprs: &mut Vec<js::Expr>, stmt: &ast::Statement) {
    use ast::Statement::*;
    match stmt {
        Empty => {}
        Expr(expr) => exprs.push(compile(ctx, expr)),
        LetDef((pat, var_expr)) => {
            let rhs = compile(ctx, var_expr);
            compile_let_pattern_flat(ctx, exprs, pat, rhs);
        }
        LetRecDef(defs) => {
            let mut vars = Vec::new();
            let mut rhs_exprs = Vec::new();
            for (name, _) in defs {
                vars.push(ctx.new_var(*name))
            }
            for (_, (expr, _)) in defs {
                rhs_exprs.push(compile(ctx, expr))
            }

            for (lhs, rhs) in vars.into_iter().zip(rhs_exprs) {
                exprs.push(js::assign(lhs, rhs));
            }
        }
        Println(args) => {
            let args = args.iter().map(|expr| compile(ctx, expr)).collect();
            exprs.push(js::println(args));
        }
    }
}

pub fn compile_script(ctx: &mut Context<'_>, parsed: &[ast::Statement]) -> js::Expr {
    let mut exprs = Vec::new();

    for item in parsed {
        compile_statement(ctx, &mut exprs, item);
    }

    js::comma_list(exprs)
}
