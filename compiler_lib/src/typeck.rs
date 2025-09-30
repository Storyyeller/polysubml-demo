use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::ast;
use crate::ast::StringId;
use crate::core::*;
use crate::parse_types::TreeMaterializerState;
use crate::parse_types::TypeParser;
use crate::spans::SpannedError as SyntaxError;
use crate::type_errors::HoleSrc;
use crate::unwindmap::UnwindMap;
use crate::unwindmap::UnwindPoint;

use UTypeHead::*;
use VTypeHead::*;

type Result<T> = std::result::Result<T, SyntaxError>;

type BindingsUnwindPoint = (UnwindPoint, UnwindPoint, ScopeLvl);
pub struct Bindings {
    pub vars: UnwindMap<StringId, Value>,
    pub types: UnwindMap<StringId, TypeCtorInd>,
    pub scopelvl: ScopeLvl,
}
impl Bindings {
    fn new() -> Self {
        Self {
            vars: UnwindMap::new(),
            types: UnwindMap::new(),
            scopelvl: ScopeLvl(0),
        }
    }

    fn unwind_point(&mut self) -> BindingsUnwindPoint {
        (self.vars.unwind_point(), self.types.unwind_point(), self.scopelvl)
    }

    fn unwind(&mut self, n: BindingsUnwindPoint) {
        self.vars.unwind(n.0);
        self.types.unwind(n.1);
        self.scopelvl = n.2;
    }

    fn make_permanent(&mut self, n: BindingsUnwindPoint) {
        self.vars.make_permanent(n.0);
        self.types.make_permanent(n.1);
    }
}

#[allow(non_snake_case)]
pub struct TypeckState {
    core: TypeCheckerCore,
    bindings: Bindings,

    TY_BOOL: TypeCtorInd,
    TY_FLOAT: TypeCtorInd,
    TY_INT: TypeCtorInd,
    TY_STR: TypeCtorInd,
}
impl TypeckState {
    #[allow(non_snake_case)]
    pub fn new(strings: &mut lasso::Rodeo) -> Self {
        let mut core = TypeCheckerCore::new();
        let TY_BOOL = core.add_builtin_type(strings.get_or_intern_static("bool"));
        let TY_FLOAT = core.add_builtin_type(strings.get_or_intern_static("float"));
        let TY_INT = core.add_builtin_type(strings.get_or_intern_static("int"));
        let TY_STR = core.add_builtin_type(strings.get_or_intern_static("str"));

        let mut new = Self {
            core,
            bindings: Bindings::new(),

            TY_BOOL,
            TY_FLOAT,
            TY_INT,
            TY_STR,
        };

        let n = new.bindings.unwind_point();
        for (i, ty) in new.core.type_ctors.iter().enumerate() {
            new.bindings.types.insert(ty.name, TypeCtorInd(i));
        }
        new.bindings.make_permanent(n);

        new
    }

    fn parse_type_signature(&mut self, tyexpr: &ast::STypeExpr) -> Result<(Value, Use)> {
        let temp = TypeParser::new(&self.bindings.types).parse_type(tyexpr)?;
        let mut mat = TreeMaterializerState::new(self.bindings.scopelvl);
        Ok(mat.with(&mut self.core).add_type(temp))
    }

    fn process_let_pattern(&mut self, pat: &ast::LetPattern, no_typed_var_allowed: bool) -> Result<Use> {
        let temp = TypeParser::new(&self.bindings.types).parse_let_pattern(pat, no_typed_var_allowed)?;
        let mut mat = TreeMaterializerState::new(self.bindings.scopelvl);
        Ok(mat.with(&mut self.core).add_pattern(temp, &mut self.bindings))
    }

    fn check_expr(&mut self, strings: &mut lasso::Rodeo, expr: &ast::SExpr, bound: Use) -> Result<()> {
        use ast::Expr::*;
        match &expr.0 {
            Block(e) => {
                assert!(e.statements.len() >= 1);
                let mark = self.bindings.unwind_point();

                for stmt in e.statements.iter() {
                    self.check_statement(strings, stmt, false)?;
                }

                self.check_expr(strings, &e.expr, bound)?;
                self.bindings.unwind(mark);
            }
            Call(e) => {
                let arg_type = self.infer_expr(strings, &e.arg)?;

                let bound = self.core.new_use(
                    UFunc {
                        arg: arg_type,
                        ret: bound,
                    },
                    e.span,
                    None,
                );
                self.check_expr(strings, &e.func, bound)?;
            }
            FieldAccess(e) => {
                let bound = self.core.obj_use(vec![(e.field.0, (bound, None, e.field.1))], e.field.1);
                self.check_expr(strings, &e.expr, bound)?;
            }
            FieldSet(e) => {
                let rhs_type = self.infer_expr(strings, &e.value)?;
                let bound = self
                    .core
                    .obj_use(vec![(e.field.0, (bound, Some(rhs_type), e.field.1))], e.field.1);
                self.check_expr(strings, &e.expr, bound)?;
            }
            If(e) => {
                let bool_use = self.core.simple_use(self.TY_BOOL, e.cond.1);
                self.check_expr(strings, &e.cond.0, bool_use)?;
                self.check_expr(strings, &e.then_expr, bound)?;
                self.check_expr(strings, &e.else_expr, bound)?;
            }
            InstantiateUni(e) => {
                let mut params = HashMap::new();
                for &(name, ref sig) in &e.types.0 {
                    params.insert(name, self.parse_type_signature(sig)?);
                }
                let bound = self.core.new_use(
                    UInstantiateUni {
                        params: Rc::new(RefCell::new(params)),
                        target: bound,
                        src_template: (e.types.1, e.source),
                    },
                    e.expr.1,
                    None,
                );
                self.check_expr(strings, &e.expr.0, bound)?;
            }
            Loop(e) => {
                let bound = self.core.case_use(
                    vec![
                        (strings.get_or_intern_static("Break"), bound),
                        (strings.get_or_intern_static("Continue"), self.core.top_use()),
                    ],
                    None,
                    e.span,
                );
                self.check_expr(strings, &e.body, bound)?;
            }
            Match(e) => {
                let (ref match_expr, arg_span) = e.expr;
                let ref cases = e.cases;
                let full_span = e.span;
                // Bounds from the match arms
                let mut case_type_pairs = Vec::with_capacity(cases.len());
                let mut wildcard_type = None;

                // Pattern reachability checking
                let mut case_names = HashMap::with_capacity(cases.len());
                let mut wildcard = None;

                for ((pattern, pattern_span), rhs_expr) in cases {
                    use ast::LetPattern::*;
                    match pattern {
                        Case(tag, val_pat) => {
                            if let Some(old_span) = case_names.insert(&tag.0, *pattern_span) {
                                return Err(SyntaxError::new2(
                                    "SyntaxError: Duplicate match pattern",
                                    *pattern_span,
                                    "Note: Variant already matched here:",
                                    old_span,
                                ));
                            }

                            let mark = self.bindings.unwind_point();
                            let pattern_bound = self.process_let_pattern(&*val_pat, true)?;
                            // Note: bound is bound for the result types, not the pattern
                            self.check_expr(strings, rhs_expr, bound)?;
                            case_type_pairs.push((tag.0, pattern_bound));
                            self.bindings.unwind(mark);
                        }
                        Record(..) => {
                            return Err(SyntaxError::new1(
                                "SyntaxError: Invalid wildcard match pattern",
                                *pattern_span,
                            ));
                        }
                        // Wildcard case - only Var patterns will actually work here.
                        // Any other pattern will result in a type error.
                        Var(..) => {
                            if let Some(old_span) = wildcard {
                                return Err(SyntaxError::new2(
                                    "SyntaxError: Duplicate match pattern",
                                    *pattern_span,
                                    "Note: Wildcard already matched here:",
                                    old_span,
                                ));
                            }

                            wildcard = Some(*pattern_span);

                            let mark = self.bindings.unwind_point();
                            let pattern_bound = self.process_let_pattern(pattern, true)?;
                            // Note: bound is bound for the result types, not the pattern
                            self.check_expr(strings, rhs_expr, bound)?;
                            wildcard_type = Some(pattern_bound);
                            self.bindings.unwind(mark);
                        }
                    }
                }

                let bound = self.core.case_use(case_type_pairs, wildcard_type, arg_span);
                self.check_expr(strings, match_expr, bound)?;
            }

            // Cases that should be inferred instead
            BinOp(_) | Case(_) | FuncDef(_) | Literal(_) | InstantiateExist(_) | Record(_) | Typed(_) | Variable(_) => {
                // Span is just an arbitrary span (usually that of the current expression) used
                // to help users diagnose cause of a type error that doesn't go through any holes.
                let t = self.infer_expr(strings, expr)?;
                self.core.flow(strings, t, bound, expr.1, self.bindings.scopelvl)?;
            }
        };
        Ok(())
    }

    fn infer_expr(&mut self, strings: &mut lasso::Rodeo, expr: &ast::SExpr) -> Result<Value> {
        use ast::Expr::*;

        match &expr.0 {
            BinOp(e) => {
                use ast::Literal::*;
                let (arg_class, ret_class) = &e.op_type;
                let (lhs_bound, rhs_bound) = match arg_class {
                    Some(arg_class) => {
                        let cls = match arg_class {
                            Bool => self.TY_BOOL,
                            Float => self.TY_FLOAT,
                            Int => self.TY_INT,
                            Str => self.TY_STR,
                        };

                        (self.core.simple_use(cls, e.lhs_span), self.core.simple_use(cls, e.rhs_span))
                    }
                    None => (self.core.top_use(), self.core.top_use()),
                };
                self.check_expr(strings, &e.lhs, lhs_bound)?;
                self.check_expr(strings, &e.rhs, rhs_bound)?;

                let cls = match ret_class {
                    Bool => self.TY_BOOL,
                    Float => self.TY_FLOAT,
                    Int => self.TY_INT,
                    Str => self.TY_STR,
                };
                Ok(self.core.simple_val(cls, e.span))
            }
            // Allow block expressions to be inferred as well as checked
            // TODO - deduplicate this code
            Block(e) => {
                assert!(e.statements.len() >= 1);
                let mark = self.bindings.unwind_point();

                for stmt in e.statements.iter() {
                    self.check_statement(strings, stmt, false)?;
                }

                let res = self.infer_expr(strings, &e.expr)?;
                self.bindings.unwind(mark);
                Ok(res)
            }
            Case(e) => {
                let val_type = self.infer_expr(strings, &e.expr)?;
                Ok(self.core.new_val(
                    VCase {
                        case: (e.tag.0, val_type),
                    },
                    e.tag.1,
                    None,
                ))
            }
            FuncDef(e) => {
                let ((ty_params, arg_pattern, ret_tyexpr, body_expr), span) = (&e.def.0, e.def.1);
                let parsed = TypeParser::new(&self.bindings.types).parse_func_sig(
                    ty_params,
                    arg_pattern,
                    ret_tyexpr.as_ref(),
                    span,
                )?;

                let mark = self.bindings.unwind_point();
                let mut mat = TreeMaterializerState::new(self.bindings.scopelvl);
                let mut mat = mat.with(&mut self.core);
                let func_type = mat.add_func_type(&parsed);
                let ret_bound = mat.add_func_sig(parsed, &mut self.bindings);

                self.check_expr(strings, body_expr, ret_bound)?;

                self.bindings.unwind(mark);
                Ok(func_type)
            }
            // Allow if expressions to be inferred as well as checked
            // TODO - deduplicate this code
            If(e) => {
                let (cond_expr, span) = (&e.cond.0, e.cond.1);
                let bool_use = self.core.simple_use(self.TY_BOOL, span);
                self.check_expr(strings, cond_expr, bool_use)?;
                let res1 = self.infer_expr(strings, &e.then_expr)?;
                let res2 = self.infer_expr(strings, &e.else_expr)?;
                if res1 == res2 {
                    Ok(res1)
                } else {
                    // spans for Union nodes don't matter, so just use whatever is handy
                    Ok(self.core.new_val(VUnion(vec![res1, res2]), span, None))
                }
            }
            InstantiateExist(e) => {
                let (ref sigs, sigs_span) = e.types;
                let src_kind = e.source;
                let full_span = e.span;
                let mut params = HashMap::new();
                for &(name, ref sig) in sigs {
                    params.insert(name, self.parse_type_signature(sig)?);
                }

                let target = self.infer_expr(strings, &e.expr)?;
                Ok(self.core.new_val(
                    VInstantiateExist {
                        params: Rc::new(RefCell::new(params)),
                        target,
                        src_template: (sigs_span, src_kind),
                    },
                    full_span,
                    None,
                ))
            }
            Literal(e) => {
                use ast::Literal::*;
                let span = e.value.1;

                let ty = match e.lit_type {
                    Bool => self.TY_BOOL,
                    Float => self.TY_FLOAT,
                    Int => self.TY_INT,
                    Str => self.TY_STR,
                };
                Ok(self.core.simple_val(ty, span))
            }
            Record(e) => {
                let mut field_names = HashMap::with_capacity(e.fields.len());
                let mut field_type_pairs = Vec::with_capacity(e.fields.len());
                for ((name, name_span), expr, mutable, type_annot) in &e.fields {
                    if let Some(old_span) = field_names.insert(&*name, *name_span) {
                        return Err(SyntaxError::new2(
                            "SyntaxError: Repeated field name",
                            *name_span,
                            "Note: Field was already defined here",
                            old_span,
                        ));
                    }

                    if *mutable {
                        let temp =
                            TypeParser::new(&self.bindings.types).parse_type_or_hole(type_annot.as_ref(), *name_span)?;
                        let mut mat = TreeMaterializerState::new(self.bindings.scopelvl);
                        let (v, u) = mat.with(&mut self.core).add_type(temp);

                        self.check_expr(strings, expr, u)?;
                        field_type_pairs.push((*name, (v, Some(u), *name_span)));
                    } else {
                        // For immutable fields, use the type annotation if one was supplied
                        // but do not create a hole (inference variable) if there wasn't,
                        let t = if let Some(ty) = type_annot {
                            let (v, u) = self.parse_type_signature(ty)?;
                            self.check_expr(strings, expr, u)?;
                            v
                        } else {
                            self.infer_expr(strings, expr)?
                        };

                        field_type_pairs.push((*name, (t, None, *name_span)));
                    }
                }
                let fields = field_type_pairs.into_iter().collect();
                Ok(self.core.new_val(VTypeHead::VObj { fields }, e.span, None))
            }
            Typed(e) => {
                let sig_type = self.parse_type_signature(&e.type_expr)?;
                self.check_expr(strings, &e.expr, sig_type.1)?;
                Ok(sig_type.0)
            }
            Variable(e) => {
                if let Some(v) = self.bindings.vars.get(&e.name.0) {
                    Ok(*v)
                } else {
                    Err(SyntaxError::new1(format!("SyntaxError: Undefined variable"), e.name.1))
                }
            }

            // Cases that have to be checked instead
            Call(_) | FieldAccess(_) | FieldSet(_) | Loop(_) | InstantiateUni(_) | Match(_) => {
                let (v, u) = self.core.var(HoleSrc::CheckedExpr(expr.1), self.bindings.scopelvl);
                self.check_expr(strings, expr, u)?;
                Ok(v)
            }
        }
    }

    fn check_let_def(&mut self, strings: &mut lasso::Rodeo, lhs: &ast::LetPattern, expr: &ast::SExpr) -> Result<()> {
        // Check if left hand side is a simple assignment with no type annotation
        if let &ast::LetPattern::Var((Some(name), _), None) = lhs {
            // If lefthand side is a simple assignment, avoid adding an inference var
            // (and hence the possibility of prompting the user to add a type annotation)
            // when the type is "obvious" or redundant from the right hand side.
            // For FuncDef, type annotations should be added on the function definition,
            // so don't prompt for redundant annotations on the assignment.
            use ast::Expr::*;
            match &expr.0 {
                FuncDef(..) | Literal(..) | Typed(..) | Variable(..) => {
                    let ty = self.infer_expr(strings, expr)?;
                    self.bindings.vars.insert(name, ty);
                    return Ok(());
                }
                _ => {}
            };
        }

        let parsed = TypeParser::new(&self.bindings.types).parse_let_pattern(lhs, false)?;
        let mut mat = TreeMaterializerState::new(self.bindings.scopelvl);

        // Important: The RHS of a let needs to be evaluated *before* we add the bindings from the LHS
        // However, we need to compute the bound (use type) of the lhs pattern so that we can check
        // the rhs against it. Therefore, materializing the pattern is split into two calls.
        // The first merely returns the bound while the second below actually adds the pattern bindings.
        let bound = mat.with(&mut self.core).add_pattern_bound(&parsed);
        self.check_expr(strings, expr, bound)?;

        // Now add the pattern bindings
        mat.with(&mut self.core).add_pattern(parsed, &mut self.bindings);
        Ok(())
    }

    fn check_let_rec_defs(&mut self, strings: &mut lasso::Rodeo, defs: &Vec<ast::LetRecDefinition>) -> Result<()> {
        // Important: Must use the same materializer state when materializing the outer and inner function types
        let mut mat = TreeMaterializerState::new(self.bindings.scopelvl);

        let mut temp = Vec::new();
        // Parse the function signatures
        // Materialize the outer function types and assign to bindings
        for &(name, (ref expr, span)) in defs.iter() {
            match expr {
                ast::Expr::FuncDef(func_def_expr) => {
                    let ((ty_params, arg_pattern, ret_tyexpr, body_expr), span) =
                        (&func_def_expr.def.0, func_def_expr.def.1);
                    let parsed = TypeParser::new(&self.bindings.types).parse_func_sig(
                        ty_params,
                        arg_pattern,
                        ret_tyexpr.as_ref(),
                        span,
                    )?;

                    self.bindings
                        .vars
                        .insert(name, mat.with(&mut self.core).add_func_type(&parsed));
                    temp.push((parsed, body_expr));
                }
                _ => {
                    return Err(SyntaxError::new1(
                        format!("SyntaxError: Let rec can only assign function definitions."),
                        span,
                    ));
                }
            }
        }

        // Now process the body of each function definition one by one
        for (parsed, body) in temp {
            let mark = self.bindings.unwind_point();

            let ret_bound = mat.with(&mut self.core).add_func_sig(parsed, &mut self.bindings);
            self.check_expr(strings, body, ret_bound)?;

            self.bindings.unwind(mark);
        }

        Ok(())
    }

    fn check_statement(
        &mut self,
        strings: &mut lasso::Rodeo,
        def: &ast::Statement,
        allow_useless_exprs: bool,
    ) -> Result<()> {
        use ast::Statement::*;
        match def {
            Empty => {}
            Expr(expr) => {
                if !allow_useless_exprs {
                    use ast::Expr::*;
                    match &expr.0 {
                        BinOp(_) | Case(_) | FieldAccess(_) | FuncDef(_) | InstantiateExist(_) | InstantiateUni(_)
                        | Literal(_) | Record(_) | Variable(_) => {
                            return Err(SyntaxError::new1(
                                format!(
                                    "SyntaxError: Only block, call, field set, if, loop, match, and typed expressions can appear in a sequence. The value of this expression will be ignored, which is likely unintentional. If you did intend to ignore the value of this expression, do so explicitly via let _ = ..."
                                ),
                                expr.1,
                            ));
                        }
                        _ => {}
                    };
                }

                self.check_expr(strings, expr, self.core.top_use())?;
            }
            LetDef((pattern, var_expr)) => {
                self.check_let_def(strings, pattern, var_expr)?;
            }
            LetRecDef(defs) => {
                self.check_let_rec_defs(strings, defs)?;
            }
            Println(exprs) => {
                for expr in exprs {
                    self.check_expr(strings, expr, self.core.top_use())?;
                }
            }
        };
        Ok(())
    }

    pub fn check_script(&mut self, strings: &mut lasso::Rodeo, parsed: &[ast::Statement]) -> Result<()> {
        // Tell type checker to start keeping track of changes to the type state so we can roll
        // back all the changes if the script contains an error.
        self.core.save();
        let mark = self.bindings.unwind_point();

        let len = parsed.len();
        for (i, item) in parsed.iter().enumerate() {
            let is_last = i == len - 1;
            if let Err(e) = self.check_statement(strings, item, is_last) {
                // println!("num type nodes {}", self.core.num_type_nodes());

                // Roll back changes to the type state and bindings
                self.core.revert();
                self.bindings.unwind(mark);
                return Err(e);
            }
        }

        // Now that script type-checked successfully, make the global definitions permanent
        // by removing them from the changes rollback list
        self.core.make_permanent();
        self.bindings.make_permanent(mark);
        // println!("num type nodes {}", self.core.num_type_nodes());
        // println!("{} vars {} flows", self.core.varcount, self.core.flowcount);
        Ok(())
    }
}
