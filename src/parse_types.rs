use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::Hash;
use std::rc::Rc;

use crate::ast;
use crate::ast::JoinKind;
use crate::ast::PolyKind;
use crate::ast::StringId;
use crate::ast::TypeParam;
use crate::core::*;
use crate::instantiate::InstantionContext;
use crate::instantiate::Substitutions;
use crate::reachability::TypeNodeInd;
use crate::spans::Span;
use crate::spans::SpannedError as SyntaxError;
use crate::spans::SpannedError;
use crate::type_errors::HoleSrc;
use crate::typeck::Bindings;
use crate::typeck::TypeckState;
use crate::unwindmap::UnwindMap;
use crate::unwindmap::sorted;

use UTypeHead::*;
use VTypeHead::*;

type Result<T> = std::result::Result<T, SyntaxError>;

// Represent distinct declarations of polymorphic/existential types in the source code
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceLoc(Span);

enum ParsedTypeHead {
    Case(HashMap<StringId, (Span, RcParsedType)>),
    Func(RcParsedType, RcParsedType),
    Record(HashMap<StringId, (Span, RcParsedType, Option<RcParsedType>)>),

    PolyHead(Rc<PolyHeadData>, RcParsedType),
    PolyVar(VarSpec),
    RecHead(SourceLoc, RcParsedType),
    RecVar(SourceLoc),

    VarJoin(JoinKind, HashMap<VarSpec, Span>, Option<RcParsedType>),

    Bot,
    Top,
    Hole(HoleSrc),
    Simple(TypeCtorInd),
}
type ParsedType = (PolyAndRecDeps, Span, ParsedTypeHead);
type RcParsedType = Rc<ParsedType>;

#[derive(Debug, Default, Clone)]
pub struct PolyDeps(HashSet<SourceLoc>);
impl PolyDeps {
    pub fn single(loc: SourceLoc) -> Self {
        Self(vec![loc].into_iter().collect())
    }

    fn extend(&mut self, other: &Self) {
        self.0.extend(&other.0);
    }

    pub fn get(&self, key: SourceLoc) -> bool {
        self.0.contains(&key)
    }

    pub fn remove(&mut self, key: SourceLoc) {
        self.0.remove(&key);
    }
}

#[derive(Default, Clone)]
struct PolyAndRecDeps {
    poly: PolyDeps,
    rec: PolyDeps,
}
impl PolyAndRecDeps {
    fn add(&mut self, child: RcParsedType) -> RcParsedType {
        self.poly.extend(&child.0.poly);
        self.rec.extend(&child.0.rec);
        child
    }
}

pub struct ParsedTypeSig(RcParsedType);
pub struct ParsedLetPattern(RcParsedType, ParsedBindings);
pub struct ParsedFuncSig {
    bindings: ParsedBindings,
    ret_type: RcParsedType,
    func_type: RcParsedType,
}

pub struct TreeMaterializerState {
    cache: HashMap<*const ParsedType, (Value, Use)>,
    rec_types: HashMap<SourceLoc, ((Value, Use), PolyDeps)>,
    /// ScopeLvl to use when creating holes - note that this will be lower than for abstract types that are added
    scopelvl: ScopeLvl,
}
impl TreeMaterializerState {
    pub fn new(scopelvl: ScopeLvl) -> Self {
        Self {
            cache: HashMap::new(),
            rec_types: HashMap::new(),
            scopelvl,
        }
    }

    pub fn with<'a>(&'a mut self, core: &'a mut TypeCheckerCore) -> TreeMaterializer<'a> {
        TreeMaterializer {
            core,
            cache: &mut self.cache,
            rec_types: &mut self.rec_types,
            scopelvl: self.scopelvl,
        }
    }
}

pub struct TreeMaterializer<'a> {
    core: &'a mut TypeCheckerCore,
    cache: &'a mut HashMap<*const ParsedType, (Value, Use)>,
    rec_types: &'a mut HashMap<SourceLoc, ((Value, Use), PolyDeps)>,
    scopelvl: ScopeLvl,
}
impl<'a> TreeMaterializer<'a> {
    fn eval(&self, deps: &PolyAndRecDeps) -> PolyDeps {
        let mut res = deps.poly.clone();
        for loc in deps.rec.0.iter().copied() {
            res.extend(&self.rec_types.get(&loc).unwrap().1);
        }
        res
    }

    fn materialize_tree_sub_into_ph(&mut self, ty: &ParsedType, ph: (Value, Use)) {
        use ParsedTypeHead::*;

        let deps = self.eval(&ty.0);
        let (vhead, uhead) = match &ty.2 {
            &Case(ref cases) => {
                let mut utype_case_arms = HashMap::new();
                let mut vtype_case_arms = Vec::new();
                for (&tag, &(tag_span, ref ty)) in cases {
                    let (v, u) = self.materialize_tree(ty);

                    vtype_case_arms.push(((tag, v), tag_span));
                    utype_case_arms.insert(tag, u);
                }

                // Grammar ensures that cases is nonempty
                let vhead = if cases.len() <= 1 {
                    VCase {
                        case: vtype_case_arms[0].0,
                    }
                } else {
                    VUnion(
                        vtype_case_arms
                            .into_iter()
                            // deps is overly coarse here, but oh well
                            // This will lead to extra copies of the VCase nodes, but the underlying
                            // "case" node will still have correct deps, and so won't be copied.
                            .map(|(case, span)| self.core.new_val(VCase { case }, span, Some(deps.clone())))
                            .collect(),
                    )
                };
                let uhead = UCase {
                    cases: utype_case_arms,
                    wildcard: None,
                };

                (vhead, uhead)
            }
            &Func(ref arg, ref ret) => {
                let arg = self.materialize_tree(arg);
                let ret = self.materialize_tree(ret);
                (VFunc { arg: arg.1, ret: ret.0 }, UFunc { arg: arg.0, ret: ret.1 })
            }
            &Record(ref fields) => {
                let mut vtype_fields = HashMap::with_capacity(fields.len());
                let mut utype_fields = HashMap::with_capacity(fields.len());
                for (&name, &(span, ref rty, ref wty)) in fields {
                    let rty = self.materialize_tree(rty);
                    let wty = wty.as_ref().map(|wty| self.materialize_tree(wty));

                    vtype_fields.insert(name, (rty.0, wty.map(|w| w.1), span));
                    utype_fields.insert(name, (rty.1, wty.map(|w| w.0), span));
                }

                (VObj { fields: vtype_fields }, UObj { fields: utype_fields })
            }
            &PolyHead(ref data, ref sub) => {
                let sub = self.materialize_tree(sub);
                (VPolyHead(data.clone(), sub.0, false), UPolyHead(data.clone(), sub.1, false))
            }
            &PolyVar(spec) => (VTypeVar(spec), UTypeVar(spec)),
            &RecHead(loc, ref sub) => {
                self.rec_types.insert(loc, (ph, deps));
                self.materialize_tree_sub_into_ph(sub, ph);
                self.rec_types.remove(&loc);
                return;
            }

            &VarJoin(kind, ref vars, ref sub) => {
                let sub = sub.as_deref().map(|ty| self.materialize_tree(ty));
                let var_set = vars.iter().map(|(&vs, &span)| vs).collect();

                match kind {
                    JoinKind::Union => {
                        let mut vals: Vec<_> = vars
                            .iter()
                            .map(|(&vs, &span)| {
                                let deps = PolyDeps::single(vs.loc);
                                self.core.new_val(VTypeVar(vs), span, Some(deps))
                            })
                            .collect();
                        if let Some(t) = sub {
                            vals.push(t.0);
                        }
                        (VUnion(vals), UDisjointUnion(var_set, sub.map(|t| t.1)))
                    }
                    JoinKind::Intersect => {
                        let mut uses: Vec<_> = vars
                            .iter()
                            .map(|(&vs, &span)| {
                                let deps = PolyDeps::single(vs.loc);
                                self.core.new_use(UTypeVar(vs), span, Some(deps))
                            })
                            .collect();
                        if let Some(t) = sub {
                            uses.push(t.1);
                        }
                        (VDisjointIntersect(var_set, sub.map(|t| t.0)), UIntersection(uses))
                    }
                }
            }

            RecVar(..) | Simple(..) | Bot | Top | Hole(_) => unreachable!(),
        };
        let span = ty.1;
        self.core.set_val(ph.0, vhead, span, Some(deps.clone()));
        self.core.set_use(ph.1, uhead, span, Some(deps));
    }

    fn materialize_tree_sub(&mut self, ty: &ParsedType) -> (Value, Use) {
        use ParsedTypeHead::*;
        match &ty.2 {
            Case(..) | Func(..) | Record(..) | PolyHead(..) | PolyVar(..) | RecHead(..) | VarJoin(..) => {
                let vredirect = self.core.val_placeholder();
                let uredirect = self.core.use_placeholder();
                let ph = (vredirect, uredirect);
                self.materialize_tree_sub_into_ph(ty, ph);
                ph
            }
            &RecVar(loc) => self.rec_types.get(&loc).unwrap().0,
            &Simple(ty_con) => self.core.custom(ty_con, ty.1),

            &Bot => (self.core.bot(), self.core.new_use(UTypeHead::UBot, ty.1, None)),
            &Top => (self.core.new_val(VTypeHead::VTop, ty.1, None), self.core.top_use()),
            &Hole(src) => self.core.var(src, self.scopelvl),
        }
    }

    fn materialize_tree(&mut self, ty: &ParsedType) -> (Value, Use) {
        let key = ty as *const _;
        if let Some(t) = self.cache.get(&key) {
            return *t;
        }

        let t = self.materialize_tree_sub(ty);
        self.cache.insert(key, t);
        t
    }

    pub fn add_type(&mut self, parsed: ParsedTypeSig) -> (Value, Use) {
        self.materialize_tree(&parsed.0)
    }

    fn materialize_and_instantiate_bindings(
        &mut self,
        parsed: ParsedBindings,
        ret_type: RcParsedType,
        should_instantiate_ret: bool,
        bindings: &mut Bindings,
    ) -> Use {
        // First materialize all type trees
        let mut new_vars: Vec<_> = parsed
            .vars
            .into_iter()
            .map(|(name, (_, ty))| (name, self.materialize_tree(&ty).0))
            .collect();
        let mut ret_type = self.materialize_tree(&ret_type).1;

        if !parsed.types.is_empty() {
            bindings.scopelvl.0 += 1;
        }

        let mut new_types = HashMap::new();
        // Now see if we have to instantiate type parameters to local abstract types
        for spec in parsed.poly_heads {
            let subs = spec
                .params
                .iter()
                .copied()
                .map(|(name, span)| (name, self.core.add_abstract_type(name, span, bindings.scopelvl)))
                .collect();

            let mut ctx = InstantionContext::new(self.core, Substitutions::Abs(&subs), spec.loc);
            for (name, v) in new_vars.iter_mut() {
                let old = *v;
                *v = ctx.instantiate_val(*v);
                // println!("{}: inst {}->{}", name.into_inner(), old.0.0, v.0.0);
            }
            if should_instantiate_ret {
                ret_type = ctx.instantiate_use(ret_type);
            }

            // Add the new types to new_types
            for (name, tycon) in subs {
                new_types.insert((spec.loc, name), tycon);
            }
        }

        for (name, ty) in new_vars {
            // println!("var {}: {}", name.into_inner(), ty.0.0);
            bindings.vars.insert(name, ty);
        }
        for (alias, loc, name) in parsed.types {
            // println!("type {}: tycon {}", alias.into_inner(), new_types.get(&(loc, name)).unwrap().0);
            bindings.types.insert(alias, *new_types.get(&(loc, name)).unwrap());
        }
        ret_type
    }

    pub fn add_pattern_bound(&mut self, parsed: &ParsedLetPattern) -> Use {
        self.materialize_tree(&parsed.0).1
    }

    pub fn add_pattern(&mut self, parsed: ParsedLetPattern, bindings: &mut Bindings) -> Use {
        // println!("add pat {} {}", parsed.1.types.len(), parsed.1.poly_heads.len());
        self.materialize_and_instantiate_bindings(parsed.1, parsed.0, false, bindings)
    }

    pub fn add_func_type(&mut self, parsed: &ParsedFuncSig) -> Value {
        self.materialize_tree(&parsed.func_type).0
    }

    pub fn add_func_sig(&mut self, parsed: ParsedFuncSig, bindings: &mut Bindings) -> Use {
        self.materialize_and_instantiate_bindings(parsed.bindings, parsed.ret_type, true, bindings)
    }
}

#[derive(Default)]
struct ParsedBindings {
    vars: HashMap<StringId, (Span, RcParsedType)>,
    types: Vec<(StringId, SourceLoc, StringId)>,
    poly_heads: Vec<Rc<PolyHeadData>>,
}
impl ParsedBindings {
    fn insert_var(&mut self, name: StringId, span: Span, ty: RcParsedType) -> Result<()> {
        if let Some((old_span, _)) = self.vars.insert(name, (span, ty)) {
            Err(SyntaxError::new2(
                "SyntaxError: Repeated variable binding in pattern",
                span,
                "Note: Name was already bound here",
                old_span,
            ))
        } else {
            Ok(())
        }
    }
}

fn flip(k: &JoinKind) -> JoinKind {
    match k {
        JoinKind::Union => JoinKind::Intersect,
        JoinKind::Intersect => JoinKind::Union,
    }
}

enum TypeVar {
    Rec(SourceLoc),
    Param(VarSpec),
}
pub struct TypeParser<'a> {
    global_types: &'a UnwindMap<StringId, TypeCtorInd>,
    local_types: UnwindMap<StringId, TypeVar>,

    // If loc isn't allowed in either kind, remove it from the map
    // Can use regular hashmap here becuase each loc will only be processed
    // at most once in a given tree, so no need for unwinding.
    join_allowed: HashMap<SourceLoc, JoinKind>,
}
impl<'a> TypeParser<'a> {
    pub fn new(global_types: &'a UnwindMap<StringId, TypeCtorInd>) -> Self {
        Self {
            global_types,
            local_types: UnwindMap::new(),
            join_allowed: HashMap::new(),
        }
    }

    fn parse_union_or_intersect_type(
        &mut self,
        deps: &mut PolyAndRecDeps,
        kind: JoinKind,
        exprs: &[ast::STypeExpr],
    ) -> Result<ParsedTypeHead> {
        let mut vars = HashMap::new();
        let mut default = None;

        use ParsedTypeHead::*;
        for expr in exprs.iter() {
            let sub = deps.add(self.parse_type_sub(expr)?);

            match sub.2 {
                PolyVar(spec) => {
                    if self.join_allowed.get(&spec.loc) == Some(&kind) {
                        vars.insert(spec, sub.1);
                        continue;
                    }
                }
                Top | Bot => {
                    return Err(SyntaxError::new1(
                        "SyntaxError: Any and never are not allowed in union or intersection types.",
                        sub.1,
                    ));
                }

                _ => (),
            };

            match default {
                None => default = Some(sub),
                Some(old) => {
                    return Err(SyntaxError::new2(
                        "SyntaxError: Repeated ineligible join type",
                        sub.1,
                        "Note: Previous ineligible type here",
                        old.1,
                    ));
                }
            }
        }

        Ok(VarJoin(kind, vars, default))
    }

    fn parse_type_sub_contravariant(&mut self, tyexpr: &ast::STypeExpr) -> Result<RcParsedType> {
        for v in self.join_allowed.values_mut() {
            *v = flip(v);
        }
        let res = self.parse_type_sub(tyexpr);
        for v in self.join_allowed.values_mut() {
            *v = flip(v);
        }

        res
    }

    fn parse_type_sub_invariant(&mut self, tyexpr: &ast::STypeExpr) -> Result<RcParsedType> {
        let temp = std::mem::take(&mut self.join_allowed);
        let res = self.parse_type_sub(tyexpr);
        self.join_allowed = temp;

        res
    }

    fn parse_type_sub(&mut self, tyexpr: &ast::STypeExpr) -> Result<RcParsedType> {
        use ast::TypeExpr::*;
        let mut deps = PolyAndRecDeps::default();
        let span = tyexpr.1;
        let head = match &tyexpr.0 {
            Bot => ParsedTypeHead::Bot,
            Case(cases) => {
                let mut m = HashMap::new();
                for &((tag, tag_span), ref wrapped_expr) in cases {
                    let sub = deps.add(self.parse_type_sub(wrapped_expr)?);
                    m.insert(tag, (tag_span, sub));
                }
                ParsedTypeHead::Case(m)
            }
            Func(lhs, rhs) => {
                let lhs = deps.add(self.parse_type_sub_contravariant(lhs)?);
                let rhs = deps.add(self.parse_type_sub(rhs)?);
                ParsedTypeHead::Func(lhs, rhs)
            }
            Record(fields) => {
                let mut m = HashMap::new();

                for &((name, name_span), ref type_decl) in fields {
                    use ast::FieldTypeDecl::*;

                    match type_decl {
                        Imm(ty) => {
                            let ty = deps.add(self.parse_type_sub(ty)?);
                            m.insert(name, (name_span, ty, None));
                        }
                        // Mutable field with read and write types the same
                        RWSame(ty) => {
                            let ty = deps.add(self.parse_type_sub_invariant(ty)?);
                            m.insert(name, (name_span, ty.clone(), Some(ty)));
                        }
                        RWPair(ty, ty2) => {
                            let ty = deps.add(self.parse_type_sub(ty)?);
                            let ty2 = deps.add(self.parse_type_sub_contravariant(ty2)?);
                            m.insert(name, (name_span, ty, Some(ty2)));
                        }
                    }
                }
                ParsedTypeHead::Record(m)
            }
            Hole => ParsedTypeHead::Hole(HoleSrc::Explicit(span)),
            Ident(s) => {
                if let Some(ty) = self.local_types.get(s) {
                    match ty {
                        &TypeVar::Rec(loc) => {
                            deps.rec.0.insert(loc);
                            ParsedTypeHead::RecVar(loc)
                        }
                        &TypeVar::Param(spec) => {
                            deps.poly.0.insert(spec.loc);
                            ParsedTypeHead::PolyVar(spec)
                        }
                    }
                } else if let Some(&ty) = self.global_types.get(s) {
                    ParsedTypeHead::Simple(ty)
                } else {
                    return Err(SyntaxError::new1("SyntaxError: Undefined type or type constructor", span));
                }
            }
            &Poly(ref params, ref def, kind) => {
                let loc = SourceLoc(span);
                let mark = self.local_types.unwind_point();
                self.join_allowed.insert(loc, match kind {
                    PolyKind::Universal => JoinKind::Union,
                    PolyKind::Existential => JoinKind::Intersect,
                });

                let mut parsed_params = HashMap::new();
                for param in params.iter().copied() {
                    parsed_params.insert(param.name.0, param.name.1);
                    self.local_types
                        .insert(param.alias.0, TypeVar::Param(VarSpec { loc, name: param.name.0 }));
                }

                let sub = deps.add(self.parse_type_sub(def)?);

                self.join_allowed.remove(&loc);
                self.local_types.unwind(mark);
                deps.poly.0.remove(&loc);

                let spec = Rc::new(PolyHeadData {
                    kind,
                    loc,
                    params: parsed_params.into_iter().collect(),
                });
                ParsedTypeHead::PolyHead(spec, sub)
            }
            &RecursiveDef(name, ref def) => {
                let loc = SourceLoc(span);

                let mark = self.local_types.unwind_point();
                self.local_types.insert(name, TypeVar::Rec(loc));
                let sub = deps.add(self.parse_type_sub(def)?);
                self.local_types.unwind(mark);

                use ParsedTypeHead::*;
                if !matches!(sub.2, Case(..) | Func(..) | Record(..) | PolyHead(..) | RecHead(..)) {
                    return Err(SyntaxError::new1(
                        "SyntaxError: Recursive types must be defined as a function, record, variant, or recursive type.",
                        sub.1,
                    ));
                }

                deps.rec.0.remove(&loc);
                ParsedTypeHead::RecHead(loc, sub)
            }
            Top => ParsedTypeHead::Top,
            &VarJoin(kind, ref children) => self.parse_union_or_intersect_type(&mut deps, kind, children)?,
        };
        Ok(Rc::new((deps, span, head)))
    }

    fn parse_type_or_hole_sub(&mut self, tyexpr: Option<&ast::STypeExpr>, span_before_hole: Span) -> Result<RcParsedType> {
        tyexpr.map(|tyexpr| self.parse_type_sub(tyexpr)).unwrap_or_else(|| {
            Ok(Rc::new((
                PolyAndRecDeps::default(),
                span_before_hole,
                ParsedTypeHead::Hole(HoleSrc::OptAscribe(span_before_hole)),
            )))
        })
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn parse_type(&mut self, tyexpr: &ast::STypeExpr) -> Result<ParsedTypeSig> {
        Ok(ParsedTypeSig(self.parse_type_sub(tyexpr)?))
    }

    pub fn parse_type_or_hole(&mut self, tyexpr: Option<&ast::STypeExpr>, span_before_hole: Span) -> Result<ParsedTypeSig> {
        Ok(ParsedTypeSig(self.parse_type_or_hole_sub(tyexpr, span_before_hole)?))
    }

    fn add_type_params(
        &mut self,
        loc: SourceLoc,
        ty_params: &[TypeParam],
        kind: ast::PolyKind,
        out: &mut ParsedBindings,
    ) -> Option<Rc<PolyHeadData>> {
        if ty_params.is_empty() {
            return None;
        }

        let mut parsed_params = HashMap::new();
        for param in ty_params.iter().copied() {
            let (name, name_span) = param.name;
            let (alias, alias_span) = param.alias;

            parsed_params.insert(name, name_span);
            self.local_types.insert(alias, TypeVar::Param(VarSpec { loc, name }));
            out.types.push((alias, loc, name));
        }

        let spec = Rc::new(PolyHeadData {
            kind,
            loc,
            params: parsed_params.into_iter().collect(),
        });
        out.poly_heads.push(spec.clone());
        Some(spec)
    }

    fn parse_let_pattern_sub(
        &mut self,
        pat: &ast::LetPattern,
        out: &mut ParsedBindings,
        no_typed_var_allowed: bool,
    ) -> Result<RcParsedType> {
        use ast::LetPattern::*;

        Ok(match pat {
            &Var((name, span), ref tyexpr) => {
                let ty = if let Some(tyexpr) = tyexpr.as_ref() {
                    self.parse_type_sub(tyexpr)?
                } else {
                    let head = if name.is_some() {
                        // If pattern does not allow unpathenthesized typed vars, it needs to be
                        // surrounded in pathenthesis when adding a type annotation.
                        let src = if no_typed_var_allowed {
                            HoleSrc::BareVarPattern(span)
                        } else {
                            HoleSrc::OptAscribe(span)
                        };
                        ParsedTypeHead::Hole(src)
                    } else {
                        ParsedTypeHead::Top
                    };
                    Rc::new((PolyAndRecDeps::default(), span, head))
                };
                if let Some(name) = name {
                    out.insert_var(name, span, ty.clone())?;
                }
                ty
            }

            &Case((tag, span), ref val_pat) => {
                let sub = self.parse_let_pattern_sub(val_pat, out, true)?;

                let deps = sub.0.clone();
                let mut m = HashMap::new();
                m.insert(tag, (span, sub));

                Rc::new((deps, span, ParsedTypeHead::Case(m)))
            }
            &Record(((ref ty_params, ref pairs), span)) => {
                let loc = SourceLoc(span);
                let mark = self.local_types.unwind_point();

                let poly_spec = self.add_type_params(loc, ty_params, ast::PolyKind::Existential, out);

                let mut field_names = HashMap::with_capacity(pairs.len());
                let mut fields = HashMap::new();
                let mut deps = PolyAndRecDeps::default();
                for &((name, name_span), ref sub_pattern) in pairs {
                    if let Some(old_span) = field_names.insert(name, name_span) {
                        return Err(SyntaxError::new2(
                            "SyntaxError: Repeated field pattern name",
                            name_span,
                            "Note: Field was already bound here",
                            old_span,
                        ));
                    }

                    let sub = deps.add(self.parse_let_pattern_sub(sub_pattern, out, false)?);
                    fields.insert(name, (name_span, sub, None));
                }
                self.local_types.unwind(mark);

                let mut new_type = Rc::new((deps.clone(), span, ParsedTypeHead::Record(fields)));
                if let Some(spec) = poly_spec {
                    deps.poly.remove(loc);
                    new_type = Rc::new((deps, span, ParsedTypeHead::PolyHead(spec, new_type)));
                }

                new_type
            }
        })
    }

    pub fn parse_let_pattern(&mut self, pat: &ast::LetPattern, no_typed_var_allowed: bool) -> Result<ParsedLetPattern> {
        let mut out = ParsedBindings::default();
        let ty = self.parse_let_pattern_sub(pat, &mut out, no_typed_var_allowed)?;
        Ok(ParsedLetPattern(ty, out))
    }

    pub fn parse_func_sig(
        &mut self,
        ty_params: &Option<Vec<TypeParam>>,
        arg_pat: &(ast::LetPattern, Span),
        ret_type: Option<&ast::STypeExpr>,
        span: Span,
    ) -> Result<ParsedFuncSig> {
        let (arg_pat, arg_pat_span) = (&arg_pat.0, arg_pat.1);

        let ty_params = ty_params.as_ref().map(|v| &v[..]).unwrap_or_default();
        let loc = SourceLoc(span);
        let mut out = ParsedBindings::default();

        let mark = self.local_types.unwind_point();
        let poly_spec = self.add_type_params(loc, ty_params, ast::PolyKind::Universal, &mut out);

        let mut deps = PolyAndRecDeps::default();
        let arg_bound = deps.add(self.parse_let_pattern_sub(arg_pat, &mut out, true)?);
        let ret_type = deps.add(self.parse_type_or_hole_sub(ret_type, arg_pat_span)?);

        self.local_types.unwind(mark);

        let mut func_type = Rc::new((deps, span, ParsedTypeHead::Func(arg_bound, ret_type.clone())));

        if let Some(spec) = poly_spec {
            func_type = Rc::new((PolyAndRecDeps::default(), span, ParsedTypeHead::PolyHead(spec, func_type)));
        }

        Ok(ParsedFuncSig {
            bindings: out,
            ret_type,
            func_type,
        })
    }
}
