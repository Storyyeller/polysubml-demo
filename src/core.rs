use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::error;
use std::fmt;
use std::rc::Rc;

use crate::ast::InstantiateSourceKind;
use crate::ast::PolyKind;
use crate::ast::StringId;
use crate::bound_pairs_set::BoundPairsSet;
use crate::instantiate::InstantionContext;
use crate::instantiate::Substitutions;
use crate::parse_types::PolyDeps;
use crate::parse_types::SourceLoc;
use crate::reachability;
use crate::reachability::EdgeDataTrait;
use crate::reachability::ExtNodeDataTrait;
use crate::reachability::Reachability;
use crate::reachability::TypeNodeInd;
use crate::spans::Span;
use crate::spans::SpannedError as TypeError;
use crate::type_errors::HoleSrc;
use crate::type_errors::PartialTypeError;
use crate::type_errors::immutable_field_err;
use crate::type_errors::missing_field_err;
use crate::type_errors::poisoned_poly_err;
use crate::type_errors::type_escape_error;
use crate::type_errors::type_mismatch_err;
use crate::type_errors::unhandled_variant_err;
use crate::unwindmap::sorted;

const NONE: TypeNodeInd = TypeNodeInd(usize::MAX);
pub const INVALID: TypeNodeInd = TypeNodeInd(usize::MAX - 1);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Value(pub TypeNodeInd);
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Use(pub TypeNodeInd);

/// Tracks which types were in scope when a given hole was created
/// This is an integer which is incremeneted whenever one or more
/// types are added to the bindings.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ScopeLvl(pub u32);

#[derive(Debug)]
pub struct TypeCtor {
    pub name: StringId,
    pub span: Option<Span>, // None for builtin type ctors
    pub scopelvl: ScopeLvl,
    debug: String,
}
impl TypeCtor {
    fn new(name: StringId, span: Option<Span>, scopelvl: ScopeLvl) -> Self {
        let debug = format!("{}@{:?}", name.into_inner(), span);
        Self {
            name,
            span,
            scopelvl,
            debug,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TypeCtorInd(pub usize);

#[derive(Debug)]
pub struct PolyHeadData {
    pub kind: PolyKind,
    pub loc: SourceLoc,
    pub params: Box<[(StringId, Span)]>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct VarSpec {
    pub loc: SourceLoc,
    pub name: StringId,
}

// Heads will be cloned during instantiation in order to work around the borrow checker
#[derive(Debug, Clone)]
pub enum VTypeHead {
    VUnion(Vec<Value>),
    VInstantiateExist {
        // Only mutated during instantiation process and during cleanup, but use RefCell for simplicity
        params: Rc<RefCell<HashMap<StringId, (Value, Use)>>>,
        target: Value,
        src_template: (Span, InstantiateSourceKind),
    },

    VTop,
    VFunc {
        arg: Use,
        ret: Value,
    },
    VObj {
        fields: HashMap<StringId, (Value, Option<Use>, Span)>,
    },
    VCase {
        case: (StringId, Value),
    },
    VAbstract {
        ty: TypeCtorInd,
    },

    VPolyHead(Rc<PolyHeadData>, Value, bool),
    VTypeVar(VarSpec),
    VDisjointIntersect(HashSet<VarSpec>, Option<Value>),
}

#[derive(Debug, Clone)]
pub enum UTypeHead {
    UIntersection(Vec<Use>),
    UInstantiateUni {
        // Only mutated during instantiation process and during cleanup, but use RefCell for simplicity
        params: Rc<RefCell<HashMap<StringId, (Value, Use)>>>,
        target: Use,
        src_template: (Span, InstantiateSourceKind),
    },

    UBot,
    UFunc {
        arg: Value,
        ret: Use,
    },
    UObj {
        fields: HashMap<StringId, (Use, Option<Value>, Span)>,
    },
    UCase {
        cases: HashMap<StringId, Use>,
        wildcard: Option<Use>,
    },
    UAbstract {
        ty: TypeCtorInd,
    },
    UPolyHead(Rc<PolyHeadData>, Use, bool),
    UTypeVar(VarSpec),
    UDisjointUnion(HashSet<VarSpec>, Option<Use>),
}
pub type VTypeNode = (VTypeHead, Span, PolyDeps);
pub type UTypeNode = (UTypeHead, Span, PolyDeps);

enum CheckHeadsResult {
    Done,
    Instantiate {
        poly: Rc<PolyHeadData>,
        substitution_params: Rc<RefCell<HashMap<StringId, (Value, Use)>>>,
        src_template: (Span, InstantiateSourceKind),
        reason: FlowReason,

        // If poly.kind is Universal, instantiate lhs then flow lhs' -> rhs
        // otherwise, instantiate rhs then flow lhs -> rhs'
        lhs_sub: Value,
        rhs_sub: Use,
    },
}

fn check_heads(
    strings: &mut lasso::Rodeo,
    type_ctors: &[TypeCtor],
    lhs_ind: Value,
    lhs: &VTypeNode,
    rhs_ind: Use,
    rhs: &UTypeNode,
    mut edge_context: TypeEdge,
    out: &mut Vec<(Value, Use, TypeEdge)>,
) -> Result<CheckHeadsResult, PartialTypeError> {
    use CheckHeadsResult::*;
    use UTypeHead::*;
    use VTypeHead::*;
    edge_context.reason = FlowReason::Check(lhs_ind, rhs_ind);

    // Remove unused context
    // edge_context.bound_pairs.filter(|&(a, b)| lhs.2.get(a) && rhs.2.get(b));
    edge_context.bound_pairs.filter_left(|a| lhs.2.get(a));
    edge_context.bound_pairs.filter_right(|a| rhs.2.get(a));

    // First handle (non-disjoint) unions and intersections
    if let &VUnion(ref types) = &lhs.0 {
        for lhs2 in types.iter().copied() {
            out.push((lhs2, rhs_ind, edge_context.clone()));
        }
        return Ok(Done);
    } else if let &UIntersection(ref types) = &rhs.0 {
        for rhs2 in types.iter().copied() {
            out.push((lhs_ind, rhs2, edge_context.clone()));
        }
        return Ok(Done);
    }

    // Now handle disjoint unions and intersections
    if let &VDisjointIntersect(ref vars1, def1) = &lhs.0 {
        match &rhs.0 {
            &UDisjointUnion(ref vars2, def2) => {
                if edge_context.bound_pairs.disjoint_union_vars_have_match(vars1, vars2) {
                    return Ok(Done);
                }
            }
            &UTypeVar(tv2) => {
                let mut vars2 = HashSet::new();
                vars2.insert(tv2);
                if edge_context.bound_pairs.disjoint_union_vars_have_match(vars1, &vars2) {
                    return Ok(Done);
                }
            }
            _ => {}
        };

        if let Some(lhs2) = def1 {
            out.push((lhs2, rhs_ind, edge_context));
            return Ok(Done);
        }
    } else if let &UDisjointUnion(ref vars2, def2) = &rhs.0 {
        // Case where lhs is DisjointIntersect was already handled above, so we only need to check for lone TypeVar
        if let &VTypeVar(tv1) = &lhs.0 {
            let mut vars1 = HashSet::new();
            vars1.insert(tv1);
            if edge_context.bound_pairs.disjoint_union_vars_have_match(&vars1, vars2) {
                return Ok(Done);
            }
        }

        if let Some(rhs2) = def2 {
            out.push((lhs_ind, rhs2, edge_context));
            return Ok(Done);
        }
    }

    // Now check to see if we need to instantiate polymorphic types
    // Important: Only do this after checking for unions and intersections
    if let &VInstantiateExist {
        target,
        ref params,
        src_template,
    } = &lhs.0
    {
        if let &UPolyHead(ref poly, rhs_sub, poison) = &rhs.0 {
            if poly.kind == PolyKind::Existential {
                if poison {
                    return Err(poisoned_poly_err(lhs.1));
                }
                return Ok(CheckHeadsResult::Instantiate {
                    poly: poly.clone(),
                    substitution_params: params.clone(),
                    src_template,
                    reason: edge_context.reason,
                    lhs_sub: target,
                    rhs_sub,
                });
            }
        }
        out.push((target, rhs_ind, edge_context));
        return Ok(Done);
    } else if let &UInstantiateUni {
        target,
        ref params,
        src_template,
    } = &rhs.0
    {
        if let &VPolyHead(ref poly, lhs_sub, poison) = &lhs.0 {
            if poly.kind == PolyKind::Universal {
                if poison {
                    return Err(poisoned_poly_err(rhs.1));
                }
                return Ok(CheckHeadsResult::Instantiate {
                    poly: poly.clone(),
                    substitution_params: params.clone(),
                    src_template,
                    reason: edge_context.reason,
                    lhs_sub,
                    rhs_sub: target,
                });
            }
        }
        out.push((lhs_ind, target, edge_context));
        return Ok(Done);
    }

    match (&lhs.0, &rhs.0) {
        // Check for polymorphic heads and update the edge context, then recurse
        (&VPolyHead(ref lhs_poly, lhs_t, _), &UPolyHead(ref rhs_poly, rhs_t, _)) => {
            edge_context.bound_pairs.push((lhs_poly.loc, rhs_poly.loc));
            out.push((lhs_t, rhs_t, edge_context));
        }
        (&VPolyHead(ref lhs_poly, lhs_t, _), _) => {
            out.push((lhs_t, rhs_ind, edge_context));
        }
        (_, &UPolyHead(ref rhs_poly, rhs_t, _)) => {
            out.push((lhs_ind, rhs_t, edge_context));
        }

        // Check for basic types - the type constructors on both sides have to match.
        (
            &VFunc {
                arg: arg1, ret: ret1, ..
            },
            &UFunc {
                arg: arg2, ret: ret2, ..
            },
        ) => {
            // flip the order since arguments are contravariant
            out.push((arg2, arg1, edge_context.flip()));
            out.push((ret1, ret2, edge_context));
        }
        (&VObj { fields: ref fields1 }, &UObj { fields: ref fields2 }) => {
            // Check if the accessed field is defined
            for (name, &(rhs_r, rhs_w, rhs_span)) in fields2.iter() {
                if let Some(&(lhs_r, lhs_w, lhs_span)) = fields1.get(name) {
                    out.push((lhs_r, rhs_r, edge_context.clone()));

                    // Check for mutability
                    if let Some(rhs_w) = rhs_w {
                        if let Some(lhs_w) = lhs_w {
                            // Contravariant
                            out.push((rhs_w, lhs_w, edge_context.flip()));
                        } else {
                            return Err(immutable_field_err(lhs_span, rhs_span, strings.resolve(&name)));
                        }
                    }
                } else {
                    return Err(missing_field_err(lhs.1, rhs_span, strings.resolve(&name)));
                }
            }
        }
        (
            &VCase { case: (name, lhs2) },
            &UCase {
                cases: ref cases2,
                wildcard,
            },
        ) => {
            // Check if the right case is handled
            if let Some(rhs2) = cases2.get(&name).copied() {
                out.push((lhs2, rhs2, edge_context));
            } else if let Some(rhs2) = wildcard {
                out.push((lhs_ind, rhs2, edge_context));
            } else {
                return Err(unhandled_variant_err(lhs, rhs, strings.resolve(&name)));
            }
        }

        (&VAbstract { ty: ty_ind1 }, &UAbstract { ty: ty_ind2 }) => {
            let ty_def1 = &type_ctors[ty_ind1.0];
            let ty_def2 = &type_ctors[ty_ind2.0];
            if ty_ind1 == ty_ind2 {
                if edge_context.scopelvl < ty_def1.scopelvl {
                    return Err(type_escape_error(strings, ty_def1, lhs, rhs, edge_context.scopelvl));
                }
            } else {
                return Err(type_mismatch_err(strings, type_ctors, lhs, rhs));
            }
        }

        (&VTypeVar(tv1), &UTypeVar(tv2)) => {
            if tv1.name != tv2.name || !edge_context.bound_pairs.get(tv1.loc, tv2.loc) {
                return Err(type_mismatch_err(strings, type_ctors, lhs, rhs));
            }
        }

        _ => {
            return Err(type_mismatch_err(strings, type_ctors, lhs, rhs));
        }
    };
    Ok(Done)
}

#[derive(Debug, Clone, Copy)]
pub struct InferenceVarData {
    pub scopelvl: ScopeLvl,
    pub src: HoleSrc,
}

#[derive(Debug)]
pub enum TypeNode {
    Var(InferenceVarData),
    Value(VTypeNode),
    Use(UTypeNode),

    // Invariant: No placeholders exist when flow() is called, so they are never present during type checking.
    Placeholder,
}
impl TypeNode {
    fn scopelvl(&self) -> ScopeLvl {
        use TypeNode::*;
        match self {
            Var(data) => data.scopelvl,
            _ => ScopeLvl(u32::MAX),
        }
    }
}
impl ExtNodeDataTrait for TypeNode {
    fn truncate(&mut self, i: TypeNodeInd) {
        if let TypeNode::Value((VTypeHead::VInstantiateExist { params, .. }, ..)) = self {
            params
                .borrow_mut()
                .retain(|_, (v, u)| (v.0 < i || v.0 == NONE) && (u.0 < i || u.0 == NONE));
        }
        if let TypeNode::Use((UTypeHead::UInstantiateUni { params, .. }, ..)) = self {
            params
                .borrow_mut()
                .retain(|_, (v, u)| (v.0 < i || v.0 == NONE) && (u.0 < i || u.0 == NONE));
        }
    }
}

/// Used to track the reason a flow edge was added so we can backtrack when printing errors
#[derive(Debug, Clone, Copy)]
pub enum FlowReason {
    Root(Span),
    Transitivity(TypeNodeInd),
    Check(Value, Use),
}

#[derive(Debug, Clone)]
pub struct TypeEdge {
    scopelvl: ScopeLvl,
    bound_pairs: BoundPairsSet,
    pub reason: FlowReason,
}
impl TypeEdge {
    fn flip(&self) -> Self {
        let mut new = self.clone();
        new.bound_pairs = new.bound_pairs.flip();
        new
    }
}
impl EdgeDataTrait<TypeNode> for TypeEdge {
    fn expand(mut self, hole: &TypeNode, ind: TypeNodeInd) -> Self {
        self.scopelvl = std::cmp::min(self.scopelvl, hole.scopelvl());
        self.reason = FlowReason::Transitivity(ind);
        self
    }

    fn update(&mut self, other: &Self) -> bool {
        let mut changed = false;
        if other.scopelvl < self.scopelvl {
            self.scopelvl = other.scopelvl;
            changed = true;
        }
        if self.bound_pairs.update_intersect(&other.bound_pairs) {
            changed = true;
        }

        changed
    }
}

pub struct TypeCheckerCore {
    // Only public for instantiation.rs
    pub r: reachability::Reachability<TypeNode, TypeEdge>,
    pub type_ctors: Vec<TypeCtor>,
    pub scopelvl: ScopeLvl,
    pub flowcount: u32,
    pub varcount: u32,
}
impl TypeCheckerCore {
    pub fn new() -> Self {
        Self {
            r: Reachability::new(),
            type_ctors: Vec::new(),
            scopelvl: ScopeLvl(0),
            flowcount: 0,
            varcount: 0,
        }
    }

    pub fn add_type_ctor(&mut self, ty: TypeCtor) -> TypeCtorInd {
        let i = self.type_ctors.len();
        self.type_ctors.push(ty);
        TypeCtorInd(i)
    }
    pub fn add_builtin_type(&mut self, name: StringId) -> TypeCtorInd {
        self.add_type_ctor(TypeCtor::new(name, None, ScopeLvl(0)))
    }
    pub fn add_abstract_type(&mut self, name: StringId, span: Span) -> TypeCtorInd {
        // println!("new abs ctor {} {}", name.into_inner(), self.type_ctors.len());
        self.add_type_ctor(TypeCtor::new(name, Some(span), self.scopelvl))
    }

    fn new_edge_context(&self, reason: FlowReason, scopelvl: ScopeLvl) -> TypeEdge {
        TypeEdge {
            scopelvl,
            bound_pairs: BoundPairsSet::default(),
            reason,
        }
    }

    pub fn flow(
        &mut self,
        strings: &mut lasso::Rodeo,
        lhs: Value,
        rhs: Use,
        expl_span: Span,
        scopelvl: ScopeLvl,
    ) -> Result<(), TypeError> {
        self.flowcount += 1;
        // println!("flow #{}: {}->{}", self.flowcount, lhs.0.0, rhs.0.0);

        let mut pending_edges = vec![(lhs, rhs, self.new_edge_context(FlowReason::Root(expl_span), scopelvl))];
        let mut type_pairs_to_check = Vec::new();
        while let Some((lhs, rhs, edge_context)) = pending_edges.pop() {
            // Check for top/bottom types
            if lhs.0 == NONE || rhs.0 == NONE {
                continue;
            }

            self.r.add_edge(lhs.0, rhs.0, edge_context, &mut type_pairs_to_check);

            // Check if adding that edge resulted in any new type pairs needing to be checked
            while let Some((lhs, rhs, edge_context)) = type_pairs_to_check.pop() {
                if let TypeNode::Value(lhs_head) = self.r.get(lhs).unwrap() {
                    if let TypeNode::Use(rhs_head) = self.r.get(rhs).unwrap() {
                        let lhs = Value(lhs);
                        let rhs = Use(rhs);

                        let res = check_heads(
                            strings,
                            &self.type_ctors,
                            lhs,
                            lhs_head,
                            rhs,
                            rhs_head,
                            edge_context,
                            &mut pending_edges,
                        );
                        let res = match res {
                            Ok(v) => v,
                            Err(mut e) => {
                                e.add_hole_int(self, strings, (lhs, rhs));
                                return e.into();
                            }
                        };

                        // Handle any followup operations that require mutation
                        // e.g. function instantation
                        self.flow_sub_mut(res, &mut pending_edges, scopelvl);
                    }
                }
            }
        }
        assert!(pending_edges.is_empty() && type_pairs_to_check.is_empty());
        Ok(())
    }

    fn flow_sub_mut(&mut self, res: CheckHeadsResult, out: &mut Vec<(Value, Use, TypeEdge)>, scopelvl: ScopeLvl) {
        match res {
            CheckHeadsResult::Done => {}
            CheckHeadsResult::Instantiate {
                poly,
                substitution_params,
                src_template,
                reason,
                lhs_sub,
                rhs_sub,
            } => {
                // Domain expansion - for type parameters not already specified, substitute them
                // with a new inference variable. The same inference variable will be used for
                // all instantiations of that parameter with the same instantiation node.
                let mut params_mut = substitution_params.borrow_mut();
                for (name, _) in poly.params.iter().copied() {
                    // println!("inserting var for {}", name.into_inner());
                    params_mut
                        .entry(name)
                        .or_insert_with(|| self.var(HoleSrc::Instantiation(src_template, name), scopelvl));
                }
                drop(params_mut);

                // Now do the actual instantiation
                let params = substitution_params.borrow();
                let mut ctx = InstantionContext::new(self, Substitutions::Type(&params), poly.loc);

                // Functions can only be instantiated when they have no free variables,
                // so using empty context is ok.
                match poly.kind {
                    PolyKind::Universal => {
                        let new = ctx.instantiate_val(lhs_sub);
                        // println!("instantiate {}->{}", lhs_sub.0.0, new.0.0);
                        out.push((new, rhs_sub, self.new_edge_context(reason, scopelvl)));
                    }
                    PolyKind::Existential => {
                        let new = ctx.instantiate_use(rhs_sub);
                        out.push((lhs_sub, new, self.new_edge_context(reason, scopelvl)));
                    }
                }
            }
        }
    }

    pub fn new_val(&mut self, val_type: VTypeHead, span: Span, deps: Option<PolyDeps>) -> Value {
        // println!("val[{}] = {:?}", self.r.len(), val_type);
        Value(self.r.add_node(TypeNode::Value((val_type, span, deps.unwrap_or_default()))))
    }

    pub fn new_use(&mut self, constraint: UTypeHead, span: Span, deps: Option<PolyDeps>) -> Use {
        // println!("use[{}] = {:?}", self.r.len(), constraint);
        Use(self.r.add_node(TypeNode::Use((constraint, span, deps.unwrap_or_default()))))
    }

    pub fn var(&mut self, src: HoleSrc, scopelvl: ScopeLvl) -> (Value, Use) {
        let data = InferenceVarData { scopelvl, src };
        let i = self.r.add_node(TypeNode::Var(data));
        self.varcount += 1;
        // println!("var #{}: {} {:?}", self.flowcount, i.0, data);
        (Value(i), Use(i))
    }

    pub const fn bot(&self) -> Value {
        Value(NONE)
    }
    pub const fn top_use(&self) -> Use {
        Use(NONE)
    }

    pub fn simple_val(&mut self, ty: TypeCtorInd, span: Span) -> Value {
        self.new_val(VTypeHead::VAbstract { ty }, span, None)
    }
    pub fn simple_use(&mut self, ty: TypeCtorInd, span: Span) -> Use {
        self.new_use(UTypeHead::UAbstract { ty }, span, None)
    }

    pub fn obj_use(&mut self, fields: Vec<(StringId, (Use, Option<Value>, Span))>, span: Span) -> Use {
        let fields = fields.into_iter().collect();
        self.new_use(UTypeHead::UObj { fields }, span, None)
    }

    pub fn case_use(&mut self, cases: Vec<(StringId, Use)>, wildcard: Option<Use>, span: Span) -> Use {
        let cases = cases.into_iter().collect();
        self.new_use(UTypeHead::UCase { cases, wildcard }, span, None)
    }

    pub fn val_placeholder(&mut self) -> Value {
        Value(self.r.add_node(TypeNode::Placeholder))
    }
    pub fn use_placeholder(&mut self) -> Use {
        Use(self.r.add_node(TypeNode::Placeholder))
    }
    pub fn set_val(&mut self, ph: Value, head: VTypeHead, span: Span, deps: Option<PolyDeps>) {
        // println!("set_val[{}] = {:?}", ph.0.0, head);
        let r = self.r.get_mut(ph.0).unwrap();
        if let TypeNode::Placeholder = *r {
            *r = TypeNode::Value((head, span, deps.unwrap_or_default()));
        } else {
            unreachable!();
        }
    }
    pub fn set_use(&mut self, ph: Use, head: UTypeHead, span: Span, deps: Option<PolyDeps>) {
        // println!("set_use[{}] = {:?}", ph.0.0, head);
        let r = self.r.get_mut(ph.0).unwrap();
        if let TypeNode::Placeholder = *r {
            *r = TypeNode::Use((head, span, deps.unwrap_or_default()));
        } else {
            unreachable!();
        }
    }

    pub fn custom(&mut self, ty: TypeCtorInd, span: Span) -> (Value, Use) {
        (
            self.new_val(VTypeHead::VAbstract { ty }, span, None),
            self.new_use(UTypeHead::UAbstract { ty }, span, None),
        )
    }

    ////////////////////////////////////////////////////////////////////////////////
    pub fn save(&mut self) {
        self.r.save();
    }
    pub fn revert(&mut self) {
        self.r.revert();
    }
    pub fn make_permanent(&mut self) {
        self.r.make_permanent();
    }

    pub fn num_type_nodes(&self) -> usize {
        self.r.len()
    }
}

pub struct SavePoint(TypeCheckerCore);
